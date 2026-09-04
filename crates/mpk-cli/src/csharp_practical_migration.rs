//! Private predecessor-producer migration into the C# practical successor family.
//!
//! This adapter is deliberately detached from CLI routing and installed bundle
//! discovery.  It accepts only an already validated revision-3 frontend result,
//! a separately validated revision-4 request, and the immutable captured input
//! bytes.  There is no format selector: a successful call always emits the v2
//! family and a failed predecessor result always emits one artifact-free v2
//! diagnostic.

use crate::csharp_practical_frontend_protocol::{
    PracticalDiagnosticFamily, CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE,
};
use crate::successor_frontend_protocol::AcceptedSuccessorFrontendEnvelope;
use mpk_vc::csharp_practical_registry::{
    SuccessorCompiledSemanticProfile, SuccessorSemanticContext, ValidatedSuccessorRequest,
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
    SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
};
use mpk_vc::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, PracticalJsonValue,
};
use mpk_vc::semantic_profile_registry::RegistryRevision;
use mpk_vc::source_manifest::{input_set_hash, InputEntry};
use mpk_vc::{
    hash_domain_separated_raw, parse_strict_json, sha256_raw_file_bytes, CapturedInput, HashDomain,
    InputKind, StrictJsonLimits,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const PREDECESSOR_MIGRATION_WORK_ITEM: &str = "CSHARP-03-T02-W08";
pub const SUCCESSOR_FRONTEND_REQUEST_SCHEMA: &str = "mpk.frontend.request.v2";
pub const SUCCESSOR_FRONTEND_SUCCESS_SCHEMA: &str = "mpk.frontend.success.v2";
pub const SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA: &str = "mpk.frontend.diagnostic.v2";
pub const SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA: &str = "mpk.frontend.source_artifacts.v2";
pub const SUCCESSOR_VIR_SCHEMA: &str = "mpk.vir.v2";
pub const SUCCESSOR_SOURCE_MAP_SCHEMA: &str = "mpk.source_map.v2";
pub const SUCCESSOR_FRONTEND_MANIFEST_SCHEMA: &str = "mpk.source_manifest.frontend.v2";
pub const SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA: &str = "mpk.csharp.semantic_bindings.v1";
pub const SUCCESSOR_CLOSED_INSTANCES_SCHEMA: &str = "mpk.csharp.closed_instances.v1";

const PREDECESSOR_FRONTEND_SCHEMA: &str = "mpk.frontend.cli.v1";
const PREDECESSOR_VIR_SCHEMA: &str = "mpk.vir.v1";
const PREDECESSOR_SOURCE_MAP_SCHEMA: &str = "mpk.source_map.v1";
const PREDECESSOR_SOURCE_MANIFEST_SCHEMA: &str = "mpk.source_manifest.v1";

const REQUEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-REQUEST-2.0");
const SUCCESS_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-SUCCESS-2.0");
const DIAGNOSTIC_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-DIAGNOSTIC-2.0");
const SOURCE_ARTIFACTS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-SOURCE-ARTIFACTS-2.0");
const CONTRACT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CONTRACT-2.0");
const VIR_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-2.0");
const SOURCE_MAP_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MAP-2.0");
const SOURCE_MANIFEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MANIFEST-2.0");
const SEMANTIC_BINDINGS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SEMANTIC-BINDING-SET-1.0");
const CLOSED_INSTANCES_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-CLOSED-INSTANCES-1.0");

const REQUEST_BYTES_MAX: usize = 131_072;
const SUCCESSOR_SCHEMA_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateSuccessorSchemaKind {
    FrontendDiagnostic,
    FrontendSourceArtifacts,
    FrontendSuccess,
    DiagnosticEntry,
    DiagnosticRequestLinkage,
}

/// Strictly validates one W08-owned root or nested schema shape.
///
/// This validator intentionally does not install a consumer route. W09 owns
/// complete typed consumption; W08 uses this boundary to reject malformed
/// generated transports and to execute the frozen schema vectors in the same
/// production module that emits predecessor migrations.
pub fn validate_private_successor_schema(
    kind: PrivateSuccessorSchemaKind,
    input: &[u8],
) -> Result<(), PredecessorMigrationError> {
    parse_strict_json(input, SUCCESSOR_SCHEMA_LIMITS)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    let value: Value = serde_json::from_slice(input)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    let valid = match kind {
        PrivateSuccessorSchemaKind::FrontendDiagnostic => diagnostic_shape(&value),
        PrivateSuccessorSchemaKind::FrontendSourceArtifacts => source_artifacts_shape(&value),
        PrivateSuccessorSchemaKind::FrontendSuccess => success_shape(&value),
        PrivateSuccessorSchemaKind::DiagnosticEntry => diagnostic_entry_shape(&value),
        PrivateSuccessorSchemaKind::DiagnosticRequestLinkage => linkage_shape(&value),
    };
    if valid {
        Ok(())
    } else {
        Err(failure(PredecessorMigrationCode::ArtifactShape))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PredecessorProducer {
    CSharpScalar,
    Go,
    Java,
    Rust,
}

impl PredecessorProducer {
    pub const ALL: [Self; 4] = [Self::CSharpScalar, Self::Go, Self::Java, Self::Rust];

    pub const fn semantic_profile(self) -> &'static str {
        match self {
            Self::CSharpScalar => "mpk.csharp.scalar.v0",
            Self::Go => "mpk.go.fixed.v0",
            Self::Java => "mpk.java.scalar.v0",
            Self::Rust => "mpk.rust.checked.v0",
        }
    }

    pub const fn source_language(self) -> &'static str {
        match self {
            Self::CSharpScalar => "csharp",
            Self::Go => "go",
            Self::Java => "java",
            Self::Rust => "rust",
        }
    }

    pub const fn report_stem(self) -> &'static str {
        match self {
            Self::CSharpScalar => "csharp-scalar",
            Self::Go => "go",
            Self::Java => "java",
            Self::Rust => "rust",
        }
    }

    fn from_profile(profile: SuccessorCompiledSemanticProfile) -> Option<Self> {
        Some(match profile {
            SuccessorCompiledSemanticProfile::CSharpScalarV0 => Self::CSharpScalar,
            SuccessorCompiledSemanticProfile::GoFixedV0 => Self::Go,
            SuccessorCompiledSemanticProfile::JavaScalarV0 => Self::Java,
            SuccessorCompiledSemanticProfile::RustCheckedV0 => Self::Rust,
            SuccessorCompiledSemanticProfile::CSharpPracticalV1 => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateCSharpProducerRoute {
    ScalarV0,
    PracticalV1,
}

/// Selects the one private C# implementation from a validated profile only.
///
/// `ambient_override_present` represents a detected unlisted selector; it is
/// accepted as evidence only to reject the invocation and is never interpreted.
pub fn select_private_csharp_producer(
    profile: SuccessorCompiledSemanticProfile,
    ambient_override_present: bool,
) -> Result<PrivateCSharpProducerRoute, PredecessorMigrationError> {
    if ambient_override_present {
        return Err(failure(PredecessorMigrationCode::AmbientOverride));
    }
    match profile {
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => {
            Ok(PrivateCSharpProducerRoute::ScalarV0)
        }
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => {
            Ok(PrivateCSharpProducerRoute::PracticalV1)
        }
        SuccessorCompiledSemanticProfile::GoFixedV0
        | SuccessorCompiledSemanticProfile::JavaScalarV0
        | SuccessorCompiledSemanticProfile::RustCheckedV0 => {
            Err(failure(PredecessorMigrationCode::UnsupportedProfile))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedLimitDisposition {
    Rejected,
    DiagnosticBudget,
    OutputLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetainedPredecessorLimit {
    pub id: &'static str,
    pub inclusive_maximum: u64,
    pub source_constant: &'static str,
    pub disposition: RetainedLimitDisposition,
}

macro_rules! retained_limit {
    ($id:literal, $maximum:literal, $constant:literal, $disposition:ident) => {
        RetainedPredecessorLimit {
            id: $id,
            inclusive_maximum: $maximum,
            source_constant: $constant,
            disposition: RetainedLimitDisposition::$disposition,
        }
    };
}

pub const RETAINED_PREDECESSOR_LIMITS: [RetainedPredecessorLimit; 32] = [
    retained_limit!("source_files", 256, "SourceFilesMaximum", Rejected),
    retained_limit!(
        "source_file_bytes",
        1_048_576,
        "SourceFileBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "source_total_bytes",
        16_777_216,
        "SourceTotalBytesMaximum",
        Rejected
    ),
    retained_limit!("contract_files", 128, "ContractFilesMaximum", Rejected),
    retained_limit!(
        "contract_file_bytes",
        1_048_576,
        "ContractFileBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "contract_total_bytes",
        8_388_608,
        "ContractTotalBytesMaximum",
        Rejected
    ),
    retained_limit!("snapshot_entries", 512, "SnapshotEntriesMaximum", Rejected),
    retained_limit!(
        "snapshot_total_bytes",
        33_554_432,
        "SnapshotTotalBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "normalized_path_bytes",
        1_024,
        "NormalizedPathBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "canonical_method_id_bytes",
        1_024,
        "CanonicalMethodIdBytesMaximum",
        Rejected
    ),
    retained_limit!("selected_methods", 32, "SelectedMethodsMaximum", Rejected),
    retained_limit!("method_closure", 128, "MethodClosureMaximum", Rejected),
    retained_limit!("syntax_nodes", 250_000, "SyntaxNodesMaximum", Rejected),
    retained_limit!(
        "operations_per_method",
        100_000,
        "OperationsPerMethodMaximum",
        Rejected
    ),
    retained_limit!(
        "operations_per_closure",
        250_000,
        "OperationsPerClosureMaximum",
        Rejected
    ),
    retained_limit!(
        "cfg_blocks_per_method",
        1_024,
        "CfgBlocksPerMethodMaximum",
        Rejected
    ),
    retained_limit!(
        "cfg_blocks_per_closure",
        8_192,
        "CfgBlocksPerClosureMaximum",
        Rejected
    ),
    retained_limit!("contract_clauses", 64, "ContractClausesMaximum", Rejected),
    retained_limit!(
        "contract_nodes_per_method",
        1_024,
        "ContractNodesPerMethodMaximum",
        Rejected
    ),
    retained_limit!(
        "contract_nodes_per_closure",
        8_192,
        "ContractNodesPerClosureMaximum",
        Rejected
    ),
    retained_limit!("contract_depth", 32, "ContractDepthMaximum", Rejected),
    retained_limit!(
        "normalized_issues",
        1_024,
        "NormalizedIssuesMaximum",
        DiagnosticBudget
    ),
    retained_limit!(
        "diagnostic_message_bytes_each",
        4_096,
        "DiagnosticMessageBytesEachMaximum",
        DiagnosticBudget
    ),
    retained_limit!(
        "diagnostic_message_bytes_total",
        2_097_152,
        "DiagnosticMessageBytesTotalMaximum",
        DiagnosticBudget
    ),
    retained_limit!(
        "frontend_argument_bytes",
        131_072,
        "FrontendArgumentBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "private_runtime_stdout",
        268_435_456,
        "PrivateRuntimeStdoutMaximum",
        OutputLimit
    ),
    retained_limit!(
        "private_runtime_stderr",
        2_097_152,
        "PrivateRuntimeStderrMaximum",
        OutputLimit
    ),
    retained_limit!(
        "vir_canonical_bytes",
        201_326_592,
        "VirCanonicalBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "source_map_canonical_bytes",
        33_554_432,
        "SourceMapCanonicalBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "source_manifest_canonical_bytes",
        4_194_304,
        "SourceManifestCanonicalBytesMaximum",
        Rejected
    ),
    retained_limit!(
        "frontend_stdout",
        268_435_456,
        "FrontendStdoutMaximum",
        OutputLimit
    ),
    retained_limit!(
        "frontend_stderr",
        2_097_152,
        "FrontendStderrMaximum",
        OutputLimit
    ),
];

pub fn retained_predecessor_limit(
    id: &str,
) -> Result<RetainedPredecessorLimit, PredecessorMigrationError> {
    RETAINED_PREDECESSOR_LIMITS
        .iter()
        .copied()
        .find(|limit| limit.id == id)
        .ok_or_else(|| failure(PredecessorMigrationCode::UnknownLimit))
}

pub fn validate_retained_predecessor_limit(
    id: &str,
    candidate: u64,
) -> Result<(), PredecessorMigrationError> {
    let limit = retained_predecessor_limit(id)?;
    if candidate <= limit.inclusive_maximum {
        Ok(())
    } else {
        Err(failure(PredecessorMigrationCode::LimitExceeded))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateSuccessorArtifact {
    schema: &'static str,
    sha256: String,
    canonical_bytes: Vec<u8>,
}

impl PrivateSuccessorArtifact {
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    fn reference(&self) -> PracticalJsonValue {
        PracticalJsonValue::object(vec![
            ("schema", PracticalJsonValue::string(self.schema)),
            ("sha256", PracticalJsonValue::string(&self.sha256)),
            (
                "canonical_bytes",
                PracticalJsonValue::U64(self.canonical_bytes.len() as u64),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivatePredecessorArtifacts {
    vir: PrivateSuccessorArtifact,
    source_map: PrivateSuccessorArtifact,
    source_manifest: PrivateSuccessorArtifact,
    semantic_bindings: PrivateSuccessorArtifact,
    closed_instances: PrivateSuccessorArtifact,
    source_artifacts: PrivateSuccessorArtifact,
}

impl PrivatePredecessorArtifacts {
    pub fn vir(&self) -> &PrivateSuccessorArtifact {
        &self.vir
    }

    pub fn source_map(&self) -> &PrivateSuccessorArtifact {
        &self.source_map
    }

    pub fn source_manifest(&self) -> &PrivateSuccessorArtifact {
        &self.source_manifest
    }

    pub fn semantic_bindings(&self) -> &PrivateSuccessorArtifact {
        &self.semantic_bindings
    }

    pub fn closed_instances(&self) -> &PrivateSuccessorArtifact {
        &self.closed_instances
    }

    pub fn source_artifacts(&self) -> &PrivateSuccessorArtifact {
        &self.source_artifacts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredecessorEquivalenceReceipt {
    source_behavior_sha256: String,
    obligation_sha256: String,
    verdict_sha256: String,
    axiom_count: u64,
    practical_foundation_instances: u64,
}

impl PredecessorEquivalenceReceipt {
    pub fn source_behavior_sha256(&self) -> &str {
        &self.source_behavior_sha256
    }

    pub fn obligation_sha256(&self) -> &str {
        &self.obligation_sha256
    }

    pub fn verdict_sha256(&self) -> &str {
        &self.verdict_sha256
    }

    pub const fn axiom_count(&self) -> u64 {
        self.axiom_count
    }

    pub const fn practical_foundation_instances(&self) -> u64 {
        self.practical_foundation_instances
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivatePredecessorMigration {
    producer: PredecessorProducer,
    request: PrivateSuccessorArtifact,
    artifacts: Option<PrivatePredecessorArtifacts>,
    frontend_result: PrivateSuccessorArtifact,
    frontend_stdout: Vec<u8>,
    equivalence: PredecessorEquivalenceReceipt,
}

impl PrivatePredecessorMigration {
    pub const fn producer(&self) -> PredecessorProducer {
        self.producer
    }

    pub fn request(&self) -> &PrivateSuccessorArtifact {
        &self.request
    }

    pub fn artifacts(&self) -> Option<&PrivatePredecessorArtifacts> {
        self.artifacts.as_ref()
    }

    pub fn frontend_result(&self) -> &PrivateSuccessorArtifact {
        &self.frontend_result
    }

    pub fn frontend_stdout(&self) -> &[u8] {
        &self.frontend_stdout
    }

    pub fn equivalence(&self) -> &PredecessorEquivalenceReceipt {
        &self.equivalence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredecessorMigrationCode {
    AmbientOverride,
    UnsupportedProfile,
    IdentityMismatch,
    InputMismatch,
    ArtifactShape,
    Hash,
    UnknownLimit,
    LimitExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredecessorMigrationError {
    code: PredecessorMigrationCode,
}

impl PredecessorMigrationError {
    pub const fn code(&self) -> PredecessorMigrationCode {
        self.code
    }

    pub const fn as_str(&self) -> &'static str {
        match self.code {
            PredecessorMigrationCode::AmbientOverride => "CSHARP_PRACTICAL_MIGRATION_AMBIENT",
            PredecessorMigrationCode::UnsupportedProfile => "CSHARP_PRACTICAL_MIGRATION_PROFILE",
            PredecessorMigrationCode::IdentityMismatch => "CSHARP_PRACTICAL_MIGRATION_IDENTITY",
            PredecessorMigrationCode::InputMismatch => "CSHARP_PRACTICAL_MIGRATION_INPUT",
            PredecessorMigrationCode::ArtifactShape => "CSHARP_PRACTICAL_MIGRATION_ARTIFACT",
            PredecessorMigrationCode::Hash => "CSHARP_PRACTICAL_MIGRATION_HASH",
            PredecessorMigrationCode::UnknownLimit => "CSHARP_PRACTICAL_MIGRATION_LIMIT_ID",
            PredecessorMigrationCode::LimitExceeded => "CSHARP_PRACTICAL_MIGRATION_LIMIT",
        }
    }
}

impl fmt::Display for PredecessorMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Error for PredecessorMigrationError {}

const fn failure(code: PredecessorMigrationCode) -> PredecessorMigrationError {
    PredecessorMigrationError { code }
}

fn exact_fields(value: &Value, fields: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
    })
}

fn lower_sha256(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn artifact_ref_shape(value: &Value, schema: &str) -> bool {
    exact_fields(value, &["schema", "sha256", "canonical_bytes"])
        && value["schema"] == schema
        && lower_sha256(&value["sha256"])
        && value["canonical_bytes"].as_u64().is_some()
}

fn source_location_shape(value: &Value) -> bool {
    if value.is_null() {
        return true;
    }
    if !exact_fields(value, &["source_file_ordinal", "start_byte", "end_byte"]) {
        return false;
    }
    let Some(source_file_ordinal) = value["source_file_ordinal"].as_u64() else {
        return false;
    };
    let Some(start_byte) = value["start_byte"].as_u64() else {
        return false;
    };
    let Some(end_byte) = value["end_byte"].as_u64() else {
        return false;
    };
    source_file_ordinal <= u64::from(u16::MAX)
        && end_byte <= u64::from(u32::MAX)
        && start_byte < end_byte
}

fn diagnostic_entry_shape(value: &Value) -> bool {
    exact_fields(value, &["code", "message", "location"])
        && value["code"]
            .as_str()
            .and_then(PracticalDiagnosticFamily::parse)
            .is_some()
        && value["message"] == CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE
        && source_location_shape(&value["location"])
}

fn linkage_shape(value: &Value) -> bool {
    match value.get("state").and_then(Value::as_str) {
        Some("unvalidated") => exact_fields(value, &["state"]),
        Some("validated") => {
            exact_fields(value, &["state", "request_sha256", "semantic_context"])
                && lower_sha256(&value["request_sha256"])
                && value["semantic_context"].is_object()
        }
        _ => false,
    }
}

fn source_artifacts_shape(value: &Value) -> bool {
    exact_fields(
        value,
        &[
            "schema",
            "semantic_context",
            "selection_sha256",
            "vir",
            "source_map",
            "source_manifest",
            "semantic_bindings",
            "closed_instances",
            "foundation_descriptor",
            "boundary_contracts",
            "transition_contracts",
            "artifacts_sha256",
        ],
    ) && value["schema"] == SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA
        && value["semantic_context"].is_object()
        && lower_sha256(&value["selection_sha256"])
        && artifact_ref_shape(&value["vir"], SUCCESSOR_VIR_SCHEMA)
        && artifact_ref_shape(&value["source_map"], SUCCESSOR_SOURCE_MAP_SCHEMA)
        && artifact_ref_shape(
            &value["source_manifest"],
            SUCCESSOR_FRONTEND_MANIFEST_SCHEMA,
        )
        && artifact_ref_shape(
            &value["semantic_bindings"],
            SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA,
        )
        && artifact_ref_shape(
            &value["closed_instances"],
            SUCCESSOR_CLOSED_INSTANCES_SCHEMA,
        )
        && exact_fields(
            &value["foundation_descriptor"],
            &["schema", "id", "content_sha256"],
        )
        && value["foundation_descriptor"]["schema"] == FOUNDATION_DESCRIPTOR_SCHEMA
        && value["foundation_descriptor"]["id"] == FOUNDATION_DESCRIPTOR_ID
        && value["foundation_descriptor"]["content_sha256"] == FOUNDATION_DESCRIPTOR_CONTENT_SHA256
        && artifact_ref_array_shape(&value["boundary_contracts"], "mpk.csharp.boundary.v1")
        && artifact_ref_array_shape(&value["transition_contracts"], "mpk.csharp.transition.v1")
        && lower_sha256(&value["artifacts_sha256"])
}

fn artifact_ref_array_shape(value: &Value, schema: &str) -> bool {
    value.as_array().is_some_and(|entries| {
        entries
            .iter()
            .all(|entry| artifact_ref_shape(entry, schema))
    })
}

fn success_shape(value: &Value) -> bool {
    exact_fields(
        value,
        &[
            "schema",
            "request_sha256",
            "semantic_context",
            "artifacts",
            "success_sha256",
        ],
    ) && value["schema"] == SUCCESSOR_FRONTEND_SUCCESS_SCHEMA
        && lower_sha256(&value["request_sha256"])
        && value["semantic_context"].is_object()
        && source_artifacts_shape(&value["artifacts"])
        && value["semantic_context"] == value["artifacts"]["semantic_context"]
        && lower_sha256(&value["success_sha256"])
}

fn diagnostic_shape(value: &Value) -> bool {
    let Some(phase) = value["phase"]
        .as_u64()
        .and_then(|phase| u8::try_from(phase).ok())
    else {
        return false;
    };
    exact_fields(
        value,
        &[
            "schema",
            "raw_request_sha256",
            "raw_request_size_bytes",
            "request_linkage",
            "status",
            "phase",
            "diagnostics",
            "diagnostic_sha256",
        ],
    ) && value["schema"] == SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA
        && lower_sha256(&value["raw_request_sha256"])
        && value["raw_request_size_bytes"]
            .as_u64()
            .is_some_and(|size| u32::try_from(size).is_ok())
        && linkage_shape(&value["request_linkage"])
        && value["status"] == "rejected"
        && phase <= 8
        && value["diagnostics"].as_array().is_some_and(|entries| {
            !entries.is_empty()
                && entries.len() <= 1_024
                && entries.iter().all(|entry| {
                    diagnostic_entry_shape(entry)
                        && entry["code"]
                            .as_str()
                            .and_then(PracticalDiagnosticFamily::parse)
                            .is_some_and(|family| family.phase() == phase)
                })
        })
        && lower_sha256(&value["diagnostic_sha256"])
}

/// Regenerates one validated predecessor result into the sole v2 artifact family.
pub fn migrate_predecessor_frontend(
    successor_request: &ValidatedSuccessorRequest,
    predecessor: &AcceptedSuccessorFrontendEnvelope,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<PrivatePredecessorMigration, PredecessorMigrationError> {
    let producer = PredecessorProducer::from_profile(successor_request.compiled_profile())
        .ok_or_else(|| failure(PredecessorMigrationCode::UnsupportedProfile))?;
    validate_identity(producer, successor_request, predecessor)?;
    enforce_input_limits(captured_inputs)?;

    let semantic_context = successor_context_value(successor_request.semantic_context())?;
    let selection = practical_from_json(successor_request.selection())?;
    let selection_bytes = canonical_practical_json_bytes(&selection)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    let selection_sha256 = sha256_raw_file_bytes(&selection_bytes).to_hex();
    let request = build_frontend_request(
        successor_request,
        &semantic_context,
        &selection,
        captured_inputs,
    )?;

    let predecessor_value = parse_predecessor_envelope(predecessor)?;
    let status = predecessor.status();
    let phase = predecessor.phase();
    let verdict_sha256 = fingerprint(&PracticalJsonValue::object(vec![
        ("status", PracticalJsonValue::string(status)),
        ("phase", PracticalJsonValue::string(phase)),
    ]))?;

    if let Some(active) = predecessor.artifacts() {
        let manifest: Value = serde_json::from_slice(active.source_manifest().canonical_bytes())
            .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
        validate_captured_inputs(&manifest, captured_inputs)?;
        let artifacts = migrate_success_artifacts(
            producer,
            &semantic_context,
            &selection_sha256,
            active.vir().canonical_bytes(),
            active.source_map().canonical_bytes(),
            active.source_manifest().canonical_bytes(),
        )?;
        let success = finalized_artifact(
            SUCCESSOR_FRONTEND_SUCCESS_SCHEMA,
            "success_sha256",
            SUCCESS_HASH_DOMAIN,
            vec![
                (
                    "schema",
                    PracticalJsonValue::string(SUCCESSOR_FRONTEND_SUCCESS_SCHEMA),
                ),
                (
                    "request_sha256",
                    PracticalJsonValue::string(request.sha256()),
                ),
                ("semantic_context", semantic_context.clone()),
                ("artifacts", artifact_value(&artifacts.source_artifacts)?),
            ],
        )?;
        let old_units = predecessor_value
            .get("ir")
            .and_then(|value| value.get("value"))
            .and_then(|value| value.get("units"))
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
        let migrated_vir = parse_artifact_value(&artifacts.vir)?;
        let new_units = migrated_vir
            .get("units")
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
        let source_behavior_sha256 = semantic_units_fingerprint(old_units)?;
        if source_behavior_sha256 != semantic_units_fingerprint(new_units)? {
            return Err(failure(PredecessorMigrationCode::ArtifactShape));
        }
        let obligation_sha256 = obligations_fingerprint(old_units)?;
        if obligation_sha256 != obligations_fingerprint(new_units)? {
            return Err(failure(PredecessorMigrationCode::ArtifactShape));
        }
        Ok(finish_migration(
            producer,
            request,
            Some(artifacts),
            success,
            PredecessorEquivalenceReceipt {
                source_behavior_sha256,
                obligation_sha256,
                verdict_sha256,
                axiom_count: 0,
                practical_foundation_instances: 0,
            },
        )?)
    } else {
        let diagnostic = build_diagnostic(&request, &semantic_context, status, phase)?;
        Ok(finish_migration(
            producer,
            request,
            None,
            diagnostic,
            PredecessorEquivalenceReceipt {
                source_behavior_sha256: fingerprint(&PracticalJsonValue::object(vec![
                    ("status", PracticalJsonValue::string(status)),
                    ("selection", selection),
                ]))?,
                obligation_sha256: fingerprint(&PracticalJsonValue::Array(Vec::new()))?,
                verdict_sha256,
                axiom_count: 0,
                practical_foundation_instances: 0,
            },
        )?)
    }
}

fn finish_migration(
    producer: PredecessorProducer,
    request: PrivateSuccessorArtifact,
    artifacts: Option<PrivatePredecessorArtifacts>,
    frontend_result: PrivateSuccessorArtifact,
    equivalence: PredecessorEquivalenceReceipt,
) -> Result<PrivatePredecessorMigration, PredecessorMigrationError> {
    let schema_kind = match frontend_result.schema {
        SUCCESSOR_FRONTEND_SUCCESS_SCHEMA => PrivateSuccessorSchemaKind::FrontendSuccess,
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA => PrivateSuccessorSchemaKind::FrontendDiagnostic,
        _ => return Err(failure(PredecessorMigrationCode::ArtifactShape)),
    };
    validate_private_successor_schema(schema_kind, &frontend_result.canonical_bytes)?;
    if let Some(generated) = &artifacts {
        validate_private_successor_schema(
            PrivateSuccessorSchemaKind::FrontendSourceArtifacts,
            &generated.source_artifacts.canonical_bytes,
        )?;
    }
    let mut frontend_stdout = frontend_result.canonical_bytes.clone();
    frontend_stdout.push(b'\n');
    validate_retained_predecessor_limit("frontend_stdout", frontend_stdout.len() as u64)?;
    Ok(PrivatePredecessorMigration {
        producer,
        request,
        artifacts,
        frontend_result,
        frontend_stdout,
        equivalence,
    })
}

fn validate_identity(
    producer: PredecessorProducer,
    successor_request: &ValidatedSuccessorRequest,
    predecessor: &AcceptedSuccessorFrontendEnvelope,
) -> Result<(), PredecessorMigrationError> {
    let old_context = predecessor.semantic_context();
    let new_context = successor_request.semantic_context();
    let installed_predecessor_registry = RegistryRevision::Revision3.identity();
    let old_selection = serde_json::to_value(predecessor.selection())
        .map_err(|_| failure(PredecessorMigrationCode::IdentityMismatch))?;
    if old_context.profile_registry() != &installed_predecessor_registry
        || producer.source_language() != old_context.source_language()
        || producer.semantic_profile() != old_context.semantic_profile()
        || old_context.source_language() != new_context.source_language()
        || old_context.semantic_profile() != new_context.semantic_profile()
        || old_context.semantic_parameters().schema() != new_context.semantic_parameters().schema()
        || old_context.semantic_parameters().value() != new_context.semantic_parameters().value()
        || old_selection != *successor_request.selection()
        || new_context.foundation_descriptor().schema() != FOUNDATION_DESCRIPTOR_SCHEMA
        || new_context.foundation_descriptor().id() != FOUNDATION_DESCRIPTOR_ID
        || new_context.foundation_descriptor().content_sha256()
            != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    {
        return Err(failure(PredecessorMigrationCode::IdentityMismatch));
    }
    Ok(())
}

fn build_frontend_request(
    successor_request: &ValidatedSuccessorRequest,
    semantic_context: &PracticalJsonValue,
    selection: &PracticalJsonValue,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<PrivateSuccessorArtifact, PredecessorMigrationError> {
    let semantic_request = PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(SUCCESSOR_VALIDATED_REQUEST_SCHEMA),
        ),
        ("semantic_context", semantic_context.clone()),
        ("selection", selection.clone()),
        (
            "request_sha256",
            PracticalJsonValue::string(successor_request.request_sha256()),
        ),
    ]);
    let (source_snapshot, sidecars) = request_inventory(captured_inputs)?;
    let artifact = finalized_artifact(
        SUCCESSOR_FRONTEND_REQUEST_SCHEMA,
        "request_sha256",
        REQUEST_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_FRONTEND_REQUEST_SCHEMA),
            ),
            ("semantic_request", semantic_request),
            ("source_snapshot", source_snapshot),
            ("sidecars", sidecars),
        ],
    )?;
    if artifact.canonical_bytes.len() > REQUEST_BYTES_MAX {
        return Err(failure(PredecessorMigrationCode::LimitExceeded));
    }
    Ok(artifact)
}

fn request_inventory(
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(PracticalJsonValue, PracticalJsonValue), PredecessorMigrationError> {
    let mut source_rows = Vec::new();
    let mut sidecar_rows = Vec::new();
    let mut source_entries = Vec::new();
    let mut sidecar_entries = Vec::new();
    for input in captured_inputs {
        let digest = sha256_raw_file_bytes(input.bytes).to_hex();
        let entry = InputEntry {
            kind: input.kind,
            normalized_path: input.normalized_path.to_owned(),
            size_bytes: i64::try_from(input.bytes.len())
                .map_err(|_| failure(PredecessorMigrationCode::LimitExceeded))?,
            sha256: digest.clone(),
        };
        if input.kind == InputKind::Contract {
            let sidecar: Value = serde_json::from_slice(input.bytes)
                .map_err(|_| failure(PredecessorMigrationCode::InputMismatch))?;
            let schema = sidecar
                .get("schema")
                .and_then(Value::as_str)
                .ok_or_else(|| failure(PredecessorMigrationCode::InputMismatch))?;
            sidecar_rows.push(PracticalJsonValue::object(vec![
                ("schema", PracticalJsonValue::string(schema)),
                ("path", PracticalJsonValue::string(input.normalized_path)),
                ("raw_sha256", PracticalJsonValue::string(digest)),
            ]));
            sidecar_entries.push(entry);
        } else {
            source_rows.push(PracticalJsonValue::object(vec![
                ("path", PracticalJsonValue::string(input.normalized_path)),
                ("raw_sha256", PracticalJsonValue::string(digest)),
                (
                    "size_bytes",
                    PracticalJsonValue::U64(input.bytes.len() as u64),
                ),
            ]));
            source_entries.push(entry);
        }
    }
    sort_unique_values(&mut source_rows)?;
    sort_unique_values(&mut sidecar_rows)?;
    let source_hash = input_set_hash(&source_entries)
        .map_err(|_| failure(PredecessorMigrationCode::Hash))?
        .as_str()
        .to_owned();
    let sidecar_hash = input_set_hash(&sidecar_entries)
        .map_err(|_| failure(PredecessorMigrationCode::Hash))?
        .as_str()
        .to_owned();
    Ok((
        PracticalJsonValue::object(vec![
            ("entries", PracticalJsonValue::Array(source_rows)),
            ("snapshot_sha256", PracticalJsonValue::string(source_hash)),
        ]),
        PracticalJsonValue::object(vec![
            ("entries", PracticalJsonValue::Array(sidecar_rows)),
            ("set_sha256", PracticalJsonValue::string(sidecar_hash)),
        ]),
    ))
}

fn migrate_success_artifacts(
    producer: PredecessorProducer,
    semantic_context: &PracticalJsonValue,
    selection_sha256: &str,
    old_vir_bytes: &[u8],
    old_source_map_bytes: &[u8],
    old_manifest_bytes: &[u8],
) -> Result<PrivatePredecessorArtifacts, PredecessorMigrationError> {
    validate_retained_predecessor_limit("vir_canonical_bytes", old_vir_bytes.len() as u64)?;
    validate_retained_predecessor_limit(
        "source_map_canonical_bytes",
        old_source_map_bytes.len() as u64,
    )?;
    validate_retained_predecessor_limit(
        "source_manifest_canonical_bytes",
        old_manifest_bytes.len() as u64,
    )?;
    let context_json = json_from_practical(semantic_context)?;
    let mut old_vir: Value = serde_json::from_slice(old_vir_bytes)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    if old_vir.get("schema").and_then(Value::as_str) != Some(PREDECESSOR_VIR_SCHEMA) {
        return Err(failure(PredecessorMigrationCode::ArtifactShape));
    }
    rebind_contract_contexts(&mut old_vir, &context_json)?;
    let units = old_vir
        .get("units")
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
    let vir = finalized_artifact(
        SUCCESSOR_VIR_SCHEMA,
        "vir_sha256",
        VIR_HASH_DOMAIN,
        vec![
            ("schema", PracticalJsonValue::string(SUCCESSOR_VIR_SCHEMA)),
            ("semantic_context", semantic_context.clone()),
            ("units", practical_from_json(units)?),
        ],
    )?;
    validate_retained_predecessor_limit("vir_canonical_bytes", vir.canonical_bytes.len() as u64)?;

    let old_map: Value = serde_json::from_slice(old_source_map_bytes)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    if old_map.get("schema").and_then(Value::as_str) != Some(PREDECESSOR_SOURCE_MAP_SCHEMA)
        || old_map.get("entries").and_then(Value::as_array).is_none()
    {
        return Err(failure(PredecessorMigrationCode::ArtifactShape));
    }
    let old_manifest: Value = serde_json::from_slice(old_manifest_bytes)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    if old_manifest.get("schema").and_then(Value::as_str)
        != Some(PREDECESSOR_SOURCE_MANIFEST_SCHEMA)
    {
        return Err(failure(PredecessorMigrationCode::ArtifactShape));
    }
    let input_set_sha256 = old_manifest
        .get("input_set_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
    let source_map = finalized_artifact(
        SUCCESSOR_SOURCE_MAP_SCHEMA,
        "source_map_sha256",
        SOURCE_MAP_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_SOURCE_MAP_SCHEMA),
            ),
            ("semantic_context", semantic_context.clone()),
            (
                "selection_sha256",
                PracticalJsonValue::string(selection_sha256),
            ),
            (
                "source_snapshot_sha256",
                PracticalJsonValue::string(input_set_sha256),
            ),
            ("vir", vir.reference()),
            ("entries", practical_from_json(&old_map["entries"])?),
        ],
    )?;
    validate_retained_predecessor_limit(
        "source_map_canonical_bytes",
        source_map.canonical_bytes.len() as u64,
    )?;

    let semantic_bindings = finalized_artifact(
        SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA,
        "binding_set_sha256",
        SEMANTIC_BINDINGS_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA),
            ),
            ("semantic_context", semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(format!("predecessor.{}", producer.report_stem())),
            ),
            ("bindings", PracticalJsonValue::Array(Vec::new())),
        ],
    )?;
    let closed_instances = finalized_artifact(
        SUCCESSOR_CLOSED_INSTANCES_SCHEMA,
        "closed_set_sha256",
        CLOSED_INSTANCES_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_CLOSED_INSTANCES_SCHEMA),
            ),
            (
                "semantic_profile",
                PracticalJsonValue::string(producer.semantic_profile()),
            ),
            (
                "foundation_id",
                PracticalJsonValue::string(FOUNDATION_DESCRIPTOR_ID),
            ),
            (
                "foundation_sha256",
                PracticalJsonValue::string(FOUNDATION_DESCRIPTOR_CONTENT_SHA256),
            ),
            ("instances", PracticalJsonValue::Array(Vec::new())),
        ],
    )?;
    let source_manifest = build_source_manifest(
        semantic_context,
        selection_sha256,
        &old_manifest,
        &semantic_bindings,
        &closed_instances,
        &vir,
        &source_map,
    )?;
    validate_retained_predecessor_limit(
        "source_manifest_canonical_bytes",
        source_manifest.canonical_bytes.len() as u64,
    )?;
    let source_artifacts = finalized_artifact(
        SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA,
        "artifacts_sha256",
        SOURCE_ARTIFACTS_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA),
            ),
            ("semantic_context", semantic_context.clone()),
            (
                "selection_sha256",
                PracticalJsonValue::string(selection_sha256),
            ),
            ("vir", vir.reference()),
            ("source_map", source_map.reference()),
            ("source_manifest", source_manifest.reference()),
            ("semantic_bindings", semantic_bindings.reference()),
            ("closed_instances", closed_instances.reference()),
            ("foundation_descriptor", foundation_descriptor_value()),
            ("boundary_contracts", PracticalJsonValue::Array(Vec::new())),
            (
                "transition_contracts",
                PracticalJsonValue::Array(Vec::new()),
            ),
        ],
    )?;
    Ok(PrivatePredecessorArtifacts {
        vir,
        source_map,
        source_manifest,
        semantic_bindings,
        closed_instances,
        source_artifacts,
    })
}

fn build_source_manifest(
    semantic_context: &PracticalJsonValue,
    selection_sha256: &str,
    old: &Value,
    semantic_bindings: &PrivateSuccessorArtifact,
    closed_instances: &PrivateSuccessorArtifact,
    vir: &PrivateSuccessorArtifact,
    source_map: &PrivateSuccessorArtifact,
) -> Result<PrivateSuccessorArtifact, PredecessorMigrationError> {
    let input_set_sha256 = old
        .get("input_set_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
    let copied = |name: &str| {
        old.get(name)
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))
            .and_then(practical_from_json)
    };
    finalized_artifact(
        SUCCESSOR_FRONTEND_MANIFEST_SCHEMA,
        "manifest_sha256",
        SOURCE_MANIFEST_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_FRONTEND_MANIFEST_SCHEMA),
            ),
            ("semantic_context", semantic_context.clone()),
            (
                "selection_sha256",
                PracticalJsonValue::string(selection_sha256),
            ),
            ("inputs", copied("inputs")?),
            (
                "input_set_sha256",
                PracticalJsonValue::string(input_set_sha256),
            ),
            ("limit_profile", copied("limit_profile")?),
            ("release_registry", copied("release_registry")?),
            ("toolchain", copied("toolchain")?),
            ("frontend", copied("frontend")?),
            ("units", copied("units")?),
            ("target", copied("target")?),
            ("foundation_descriptor", foundation_descriptor_value()),
            ("semantic_bindings", semantic_bindings.reference()),
            ("closed_instances", closed_instances.reference()),
            ("vir", vir.reference()),
            ("source_map", source_map.reference()),
        ],
    )
}

fn rebind_contract_contexts(
    vir: &mut Value,
    semantic_context: &Value,
) -> Result<(), PredecessorMigrationError> {
    let units = vir
        .get_mut("units")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
    for unit in units {
        let functions = unit
            .get_mut("functions")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
        for function in functions {
            let contract = function
                .get_mut("contracts")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
            contract.insert("semantic_context".to_owned(), semantic_context.clone());
            contract.remove("contract_hash");
            let canonical = serde_json::to_vec(&Value::Object(contract.clone()))
                .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
            let hash = hash_domain_separated_raw(CONTRACT_HASH_DOMAIN, &canonical)
                .map_err(|_| failure(PredecessorMigrationCode::Hash))?
                .to_hex();
            contract.insert("contract_hash".to_owned(), Value::String(hash));
        }
    }
    Ok(())
}

fn build_diagnostic(
    request: &PrivateSuccessorArtifact,
    semantic_context: &PracticalJsonValue,
    status: &str,
    phase: &str,
) -> Result<PrivateSuccessorArtifact, PredecessorMigrationError> {
    let diagnostic_code = diagnostic_family(status, phase);
    finalized_artifact(
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA,
        "diagnostic_sha256",
        DIAGNOSTIC_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA),
            ),
            (
                "raw_request_sha256",
                PracticalJsonValue::string(
                    sha256_raw_file_bytes(request.canonical_bytes()).to_hex(),
                ),
            ),
            (
                "raw_request_size_bytes",
                PracticalJsonValue::U64(request.canonical_bytes().len() as u64),
            ),
            (
                "request_linkage",
                PracticalJsonValue::object(vec![
                    ("state", PracticalJsonValue::string("validated")),
                    (
                        "request_sha256",
                        PracticalJsonValue::string(request.sha256()),
                    ),
                    ("semantic_context", semantic_context.clone()),
                ]),
            ),
            ("status", PracticalJsonValue::string("rejected")),
            (
                "phase",
                PracticalJsonValue::U64(u64::from(diagnostic_phase(phase))),
            ),
            (
                "diagnostics",
                PracticalJsonValue::Array(vec![PracticalJsonValue::object(vec![
                    ("code", PracticalJsonValue::string(diagnostic_code)),
                    (
                        "message",
                        PracticalJsonValue::string(CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE),
                    ),
                    ("location", PracticalJsonValue::Null),
                ])]),
            ),
        ],
    )
}

fn diagnostic_phase(phase: &str) -> u8 {
    match phase {
        "capture" => 0,
        "release" => 1,
        "source" | "metadata" | "typecheck" => 2,
        "subset" => 3,
        "lowering" | "emission" => 8,
        _ => 0,
    }
}

fn diagnostic_family(_status: &str, phase: &str) -> &'static str {
    match phase {
        "release" => "CSHARP_PRACTICAL_DEPENDENCY",
        "source" | "metadata" | "typecheck" => "CSHARP_PRACTICAL_TYPE",
        "subset" => "CSHARP_PRACTICAL_GENERIC",
        "lowering" | "emission" => "CSHARP_PRACTICAL_LOWERING",
        _ => "CSHARP_PRACTICAL_PROTOCOL",
    }
}

fn parse_predecessor_envelope(
    predecessor: &AcceptedSuccessorFrontendEnvelope,
) -> Result<Value, PredecessorMigrationError> {
    let transport = predecessor.canonical_bytes();
    let document = transport
        .strip_suffix(b"\n")
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
    let value: Value = serde_json::from_slice(document)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    if value.get("schema").and_then(Value::as_str) != Some(PREDECESSOR_FRONTEND_SCHEMA) {
        return Err(failure(PredecessorMigrationCode::ArtifactShape));
    }
    Ok(value)
}

fn validate_captured_inputs(
    manifest: &Value,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), PredecessorMigrationError> {
    let expected = manifest
        .get("inputs")
        .and_then(Value::as_array)
        .ok_or_else(|| failure(PredecessorMigrationCode::InputMismatch))?;
    if expected.len() != captured_inputs.len() {
        return Err(failure(PredecessorMigrationCode::InputMismatch));
    }
    let actual = captured_inputs
        .iter()
        .map(|input| {
            serde_json::json!({
                "kind": input_kind_name(input.kind),
                "normalized_path": input.normalized_path,
                "sha256": sha256_raw_file_bytes(input.bytes).to_hex(),
                "size_bytes": input.bytes.len(),
            })
        })
        .collect::<Vec<_>>();
    if *expected != actual {
        return Err(failure(PredecessorMigrationCode::InputMismatch));
    }
    let entries = captured_inputs
        .iter()
        .map(|input| InputEntry {
            kind: input.kind,
            normalized_path: input.normalized_path.to_owned(),
            size_bytes: input.bytes.len() as i64,
            sha256: sha256_raw_file_bytes(input.bytes).to_hex(),
        })
        .collect::<Vec<_>>();
    let digest = input_set_hash(&entries)
        .map_err(|_| failure(PredecessorMigrationCode::Hash))?
        .as_str()
        .to_owned();
    if manifest.get("input_set_hash").and_then(Value::as_str) != Some(digest.as_str()) {
        return Err(failure(PredecessorMigrationCode::InputMismatch));
    }
    Ok(())
}

fn enforce_input_limits(
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), PredecessorMigrationError> {
    let mut paths = BTreeSet::new();
    let mut source_files = 0_u64;
    let mut source_bytes = 0_u64;
    let mut contract_files = 0_u64;
    let mut contract_bytes = 0_u64;
    validate_retained_predecessor_limit("snapshot_entries", captured_inputs.len() as u64)?;
    let total = captured_inputs.iter().try_fold(0_u64, |sum, input| {
        sum.checked_add(input.bytes.len() as u64)
            .ok_or_else(|| failure(PredecessorMigrationCode::LimitExceeded))
    })?;
    validate_retained_predecessor_limit("snapshot_total_bytes", total)?;
    for input in captured_inputs {
        let size = input.bytes.len() as u64;
        validate_retained_predecessor_limit(
            "normalized_path_bytes",
            input.normalized_path.len() as u64,
        )?;
        if input.normalized_path.is_empty()
            || input.normalized_path.starts_with('/')
            || input
                .normalized_path
                .split('/')
                .any(|part| part.is_empty() || part == "..")
            || !paths.insert(input.normalized_path)
        {
            return Err(failure(PredecessorMigrationCode::InputMismatch));
        }
        match input.kind {
            InputKind::Contract => {
                contract_files += 1;
                contract_bytes = contract_bytes
                    .checked_add(size)
                    .ok_or_else(|| failure(PredecessorMigrationCode::LimitExceeded))?;
                validate_retained_predecessor_limit("contract_file_bytes", size)?;
            }
            InputKind::Source => {
                source_files += 1;
                source_bytes = source_bytes
                    .checked_add(size)
                    .ok_or_else(|| failure(PredecessorMigrationCode::LimitExceeded))?;
                validate_retained_predecessor_limit("source_file_bytes", size)?;
            }
            InputKind::BuildManifest | InputKind::Lockfile => {}
        }
    }
    validate_retained_predecessor_limit("source_files", source_files)?;
    validate_retained_predecessor_limit("source_total_bytes", source_bytes)?;
    validate_retained_predecessor_limit("contract_files", contract_files)?;
    validate_retained_predecessor_limit("contract_total_bytes", contract_bytes)?;
    Ok(())
}

fn semantic_units_fingerprint(units: &Value) -> Result<String, PredecessorMigrationError> {
    let mut projected = units.clone();
    for unit in projected
        .as_array_mut()
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?
    {
        for function in unit
            .get_mut("functions")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?
        {
            let contract = function
                .get_mut("contracts")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
            contract.remove("semantic_context");
            contract.remove("contract_hash");
        }
    }
    fingerprint(&practical_from_json(&projected)?)
}

fn obligations_fingerprint(units: &Value) -> Result<String, PredecessorMigrationError> {
    let mut obligations = Vec::new();
    for unit in units
        .as_array()
        .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?
    {
        for function in unit
            .get("functions")
            .and_then(Value::as_array)
            .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?
        {
            let contract = function
                .get("contracts")
                .and_then(Value::as_object)
                .ok_or_else(|| failure(PredecessorMigrationCode::ArtifactShape))?;
            let projected = serde_json::json!({
                "ensures": contract.get("ensures"),
                "function_id": contract.get("function_id"),
                "loops": contract.get("loops"),
                "modifies": contract.get("modifies"),
                "panic": contract.get("panic"),
                "requires": contract.get("requires"),
                "termination": contract.get("termination"),
                "unit_id": contract.get("unit_id"),
            });
            obligations.push(practical_from_json(&projected)?);
        }
    }
    fingerprint(&PracticalJsonValue::Array(obligations))
}

fn fingerprint(value: &PracticalJsonValue) -> Result<String, PredecessorMigrationError> {
    let bytes = canonical_practical_json_bytes(value)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    Ok(sha256_raw_file_bytes(&bytes).to_hex())
}

fn finalized_artifact(
    schema: &'static str,
    hash_field: &'static str,
    domain: HashDomain,
    fields: Vec<(&'static str, PracticalJsonValue)>,
) -> Result<PrivateSuccessorArtifact, PredecessorMigrationError> {
    let preimage = PracticalJsonValue::object(fields.clone());
    let preimage_bytes = canonical_practical_json_bytes(&preimage)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    let sha256 = hash_domain_separated_raw(domain, &preimage_bytes)
        .map_err(|_| failure(PredecessorMigrationCode::Hash))?
        .to_hex();
    let mut completed = fields;
    completed.push((hash_field, PracticalJsonValue::string(&sha256)));
    let canonical_bytes = canonical_practical_json_bytes(&PracticalJsonValue::object(completed))
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    Ok(PrivateSuccessorArtifact {
        schema,
        sha256,
        canonical_bytes,
    })
}

fn artifact_value(
    artifact: &PrivateSuccessorArtifact,
) -> Result<PracticalJsonValue, PredecessorMigrationError> {
    let value: Value = serde_json::from_slice(artifact.canonical_bytes())
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    practical_from_json(&value)
}

fn parse_artifact_value(
    artifact: &PrivateSuccessorArtifact,
) -> Result<Value, PredecessorMigrationError> {
    serde_json::from_slice(artifact.canonical_bytes())
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))
}

fn foundation_descriptor_value() -> PracticalJsonValue {
    PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(FOUNDATION_DESCRIPTOR_SCHEMA),
        ),
        ("id", PracticalJsonValue::string(FOUNDATION_DESCRIPTOR_ID)),
        (
            "content_sha256",
            PracticalJsonValue::string(FOUNDATION_DESCRIPTOR_CONTENT_SHA256),
        ),
    ])
}

fn successor_context_value(
    context: &SuccessorSemanticContext,
) -> Result<PracticalJsonValue, PredecessorMigrationError> {
    let registry = context.profile_registry();
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
                (
                    "value",
                    practical_from_json(context.semantic_parameters().value())?,
                ),
            ]),
        ),
        ("foundation_descriptor", foundation_descriptor_value()),
    ]))
}

fn practical_from_json(value: &Value) -> Result<PracticalJsonValue, PredecessorMigrationError> {
    Ok(match value {
        Value::Null => PracticalJsonValue::Null,
        Value::Bool(value) => PracticalJsonValue::Bool(*value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                PracticalJsonValue::I64(value)
            } else if let Some(value) = value.as_u64() {
                PracticalJsonValue::U64(value)
            } else {
                return Err(failure(PredecessorMigrationCode::ArtifactShape));
            }
        }
        Value::String(value) => PracticalJsonValue::string(value),
        Value::Array(values) => PracticalJsonValue::Array(
            values
                .iter()
                .map(practical_from_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(entries) => PracticalJsonValue::Object(
            entries
                .iter()
                .map(|(name, value)| Ok((name.clone(), practical_from_json(value)?)))
                .collect::<Result<Vec<_>, PredecessorMigrationError>>()?,
        ),
    })
}

fn json_from_practical(value: &PracticalJsonValue) -> Result<Value, PredecessorMigrationError> {
    let bytes = canonical_practical_json_bytes(value)
        .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))?;
    serde_json::from_slice(&bytes).map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))
}

fn sort_unique_values(
    values: &mut Vec<PracticalJsonValue>,
) -> Result<(), PredecessorMigrationError> {
    let mut keyed = values
        .drain(..)
        .map(|value| {
            canonical_practical_json_bytes(&value)
                .map(|key| (key, value))
                .map_err(|_| failure(PredecessorMigrationCode::ArtifactShape))
        })
        .collect::<Result<Vec<_>, _>>()?;
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    if keyed.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(failure(PredecessorMigrationCode::InputMismatch));
    }
    values.extend(keyed.into_iter().map(|(_, value)| value));
    Ok(())
}

const fn input_kind_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Source => "source",
        InputKind::Contract => "contract",
        InputKind::BuildManifest => "build_manifest",
        InputKind::Lockfile => "lockfile",
    }
}
