//! Private consumer closure for `CSHARP-03-T02-W09`.
//!
//! The entry point in this module is intentionally absent from the command
//! dispatcher. It accepts a revision-4 registry, a caller-injected release
//! descriptor, a W08 migration, and retained Certificate v0 bytes. Successful
//! input is re-imported by every private successor consumer and summarized in
//! one deterministic receipt. Rejected frontend input stops before VC,
//! certificate, policy, AI, or API artifacts are constructed.

use crate::csharp_practical_migration::{
    validate_private_successor_schema, PredecessorProducer, PrivatePredecessorMigration,
    PrivateSuccessorArtifact, PrivateSuccessorSchemaKind, SUCCESSOR_CLOSED_INSTANCES_SCHEMA,
    SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA, SUCCESSOR_FRONTEND_MANIFEST_SCHEMA,
    SUCCESSOR_FRONTEND_REQUEST_SCHEMA, SUCCESSOR_FRONTEND_SUCCESS_SCHEMA,
    SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA, SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA,
    SUCCESSOR_SOURCE_MAP_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use crate::program_certificate::validate_private_successor_program_consumer;
use crate::successor_ai_explain::{
    build_private_successor_ai_artifacts, validate_private_successor_ai_artifacts,
    PrivateAiArtifacts, PRIVATE_AI_EXPLANATION_SCHEMA, PRIVATE_AI_REQUEST_SCHEMA,
};
use crate::successor_policy::{
    build_private_successor_policy, validate_private_successor_policy, PrivatePolicyArtifacts,
    PrivatePolicySource, PRIVATE_POLICY_EVIDENCE_SCHEMA, PRIVATE_POLICY_RECEIPT_SCHEMA,
    PRIVATE_POLICY_REPRODUCTION_SCHEMA, PRIVATE_POLICY_SCAN_SCHEMA,
};
use mpk_api::successor_api::{
    build_private_successor_api_exchange, validate_private_successor_api_exchange,
    PrivateSuccessorApiArtifactRef, PrivateSuccessorApiExchange,
    PRIVATE_SUCCESSOR_API_REQUEST_SCHEMA, PRIVATE_SUCCESSOR_API_RESPONSE_SCHEMA,
    PRIVATE_SUCCESSOR_API_SESSION_SCHEMA,
};
use mpk_vc::csharp_practical_consumer::{
    build_private_predecessor_verification_artifacts,
    validate_private_predecessor_verification_artifacts,
    validate_private_predecessor_verification_transports, PrivatePredecessorVerificationArtifacts,
    PrivatePredecessorVerificationSource, PrivateVerificationArtifactRef,
    PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA, PRIVATE_PREDECESSOR_SKELETON_SCHEMA,
    PRIVATE_PREDECESSOR_VC_SCHEMA,
};
use mpk_vc::csharp_practical_registry::{
    validate_successor_registry_document, validate_successor_semantic_request,
    SuccessorRegistryDocumentKind, ValidatedSuccessorRegistry,
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
    SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
};
use mpk_vc::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, parse_canonical_practical_json, PracticalArtifactKind,
    PracticalJsonValue,
};
use mpk_vc::{
    canonical_json_bytes, hash_domain_separated_raw, parse_strict_json, sha256_raw_file_bytes,
    HashDomain, StrictJsonLimits, StrictJsonValue,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const PRIVATE_CONSUMER_WORK_ITEM: &str = "CSHARP-03-T02-W09";
pub const PRIVATE_CONSUMER_RECEIPT_SCHEMA: &str =
    "mpk.csharp_practical.t02_w09.consumer_migration_receipt.v1";

pub const PRIVATE_CONSUMER_FAMILIES: [&str; 17] = [
    "semantic_registry",
    "semantic_context",
    "semantic_parameters",
    "selection",
    "profile_contract",
    "source_artifact",
    "foundation",
    "vir",
    "frontend_protocol",
    "source_map",
    "source_manifest",
    "vc_skeleton",
    "release",
    "policy_evidence",
    "program_assembly",
    "ai",
    "api",
];

pub const PRIVATE_CONSUMER_INVENTORY_EDGES: [&str; 18] = [
    "context.api_linkage",
    "artifacts.protocol_consumer",
    "foundation.registry",
    "foundation.program_base_bytes",
    "frontend.installed_runner",
    "vc.policy_consumer",
    "release.model_and_hash",
    "release.installed_resolver",
    "policy.generator_and_importer",
    "program.assembler",
    "program.profile_binding",
    "ai.request_and_report",
    "ai.cli_route",
    "api.router_and_sessions",
    "api.owner_test",
    "release.cli_contract_reexport",
    "release.frontend_sandbox",
    "policy.cli_routes",
];

const TRANSPORT_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);
const REQUEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-REQUEST-2.0");
const SUCCESS_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-SUCCESS-2.0");
const DIAGNOSTIC_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-FRONTEND-DIAGNOSTIC-2.0");
const SOURCE_ARTIFACTS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-SOURCE-ARTIFACTS-2.0");
const VIR_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-2.0");
const SOURCE_MAP_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MAP-2.0");
const SOURCE_MANIFEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MANIFEST-2.0");
const SEMANTIC_BINDINGS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SEMANTIC-BINDING-SET-1.0");
const CLOSED_INSTANCES_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-CLOSED-INSTANCES-1.0");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateIdentityFamily {
    pub family: String,
    pub successor_identities: Vec<String>,
    pub successor_hash_domains: Vec<String>,
}

impl PrivateIdentityFamily {
    pub fn new(
        family: impl Into<String>,
        successor_identities: impl IntoIterator<Item = impl Into<String>>,
        successor_hash_domains: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            family: family.into(),
            successor_identities: successor_identities.into_iter().map(Into::into).collect(),
            successor_hash_domains: successor_hash_domains.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrivateObservedArtifactIdentity<'a> {
    pub family: &'a str,
    pub artifact_schema: &'a str,
    pub semantic_context_schema: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateIdentityCode {
    Inventory,
    DuplicateSuccessorIdentity,
    DuplicateSuccessorHashDomain,
    MixedArtifactFamily,
}

impl PrivateIdentityCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inventory => "identity_inventory",
            Self::DuplicateSuccessorIdentity => "duplicate_successor_identity",
            Self::DuplicateSuccessorHashDomain => "duplicate_successor_hash_domain",
            Self::MixedArtifactFamily => "mixed_artifact_family",
        }
    }
}

/// Returns the immutable W09 projection of the 17 frozen identity families.
///
/// Each family includes the complete frozen successor identity/domain set.
/// Listing a later-owned release identity here does not materialize it; it
/// prevents an implementation from reusing the name in another family.
pub fn private_successor_identity_inventory() -> Vec<PrivateIdentityFamily> {
    vec![
        identity(
            "semantic_registry",
            &[
                "mpk.csharp.practical.v1",
                "mpk.semantic_profile.entry.v2",
                "mpk.semantic_profile.registry.limits.v2",
                "mpk.semantic_profile.registry.v2",
            ],
            &[
                "MPK-SEMANTIC-PROFILE-ENTRY-2.0",
                "MPK-SEMANTIC-PROFILE-REGISTRY-2.0",
            ],
        ),
        identity(
            "semantic_context",
            &[
                "mpk.semantic_context.v2",
                "mpk.validated_semantic_request.v2",
            ],
            &[
                "MPK-SEMANTIC-CONTEXT-2.0",
                "MPK-VALIDATED-SEMANTIC-REQUEST-2.0",
            ],
        ),
        identity(
            "semantic_parameters",
            &["mpk.semantic_parameters.csharp_practical.v1"],
            &["MPK-CSHARP-PRACTICAL-PARAMETERS-1.0"],
        ),
        identity(
            "selection",
            &["mpk.selection.csharp_members.v1"],
            &["MPK-CSHARP-SELECTION-1.0"],
        ),
        identity(
            "profile_contract",
            &[
                "mpk.csharp.boundary.v1",
                "mpk.csharp.boundary_input.v1",
                "mpk.csharp.boundary_output.v1",
                "mpk.csharp.canonical_json.v1",
                "mpk.csharp.contract.v1",
                "mpk.csharp.contract_expression.v1",
                "mpk.csharp.limits.v1",
                "mpk.csharp.operations.v1",
                "mpk.csharp.parse_format.v1",
                "mpk.csharp.required_checks.v1",
                "mpk.csharp.semantic_bindings.v1",
                "mpk.csharp.transition.v1",
                "mpk.csharp.type_contract.v1",
                "mpk.profile.ai.csharp_practical.v1",
                "mpk.profile.ai.csharp_scalar.v1",
                "mpk.profile.ai.go_fixed.v1",
                "mpk.profile.ai.java_scalar.v1",
                "mpk.profile.ai.rust_checked.v1",
                "mpk.profile.evidence.csharp_practical.v1",
                "mpk.profile.evidence.csharp_scalar.v1",
                "mpk.profile.evidence.go_fixed.v1",
                "mpk.profile.evidence.java_scalar.v1",
                "mpk.profile.evidence.rust_checked.v1",
                "mpk.profile.frontend.csharp_practical.v1",
                "mpk.profile.frontend.csharp_scalar.v1",
                "mpk.profile.frontend.go_fixed.v1",
                "mpk.profile.frontend.java_scalar.v1",
                "mpk.profile.frontend.rust_checked.v1",
                "mpk.profile.manifest.csharp_practical.v1",
                "mpk.profile.manifest.csharp_scalar.v1",
                "mpk.profile.manifest.go_fixed.v1",
                "mpk.profile.manifest.java_scalar.v1",
                "mpk.profile.manifest.rust_checked.v1",
                "mpk.profile.policy.csharp_practical.v1",
                "mpk.profile.policy.csharp_scalar.v1",
                "mpk.profile.policy.go_fixed.v1",
                "mpk.profile.policy.java_scalar.v1",
                "mpk.profile.policy.rust_checked.v1",
                "mpk.profile.release.csharp_practical.v1",
                "mpk.profile.release.csharp_scalar.v1",
                "mpk.profile.release.go_fixed.v1",
                "mpk.profile.release.java_scalar.v1",
                "mpk.profile.release.rust_checked.v1",
                "mpk.profile.source_map.csharp_practical.v1",
                "mpk.profile.source_map.csharp_scalar.v1",
                "mpk.profile.source_map.go_fixed.v1",
                "mpk.profile.source_map.java_scalar.v1",
                "mpk.profile.source_map.rust_checked.v1",
                "mpk.profile.vc.csharp_practical.v1",
                "mpk.profile.vc.csharp_scalar.v1",
                "mpk.profile.vc.go_fixed.v1",
                "mpk.profile.vc.java_scalar.v1",
                "mpk.profile.vc.rust_checked.v1",
                "mpk.profile.vir.csharp_practical.v1",
                "mpk.profile.vir.csharp_scalar.v1",
                "mpk.profile.vir.go_fixed.v1",
                "mpk.profile.vir.java_scalar.v1",
                "mpk.profile.vir.rust_checked.v1",
            ],
            &[
                "MPK-COMPILED-PROFILE-CONTRACT-1.0",
                "MPK-CONTRACT-2.0",
                "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
                "MPK-CSHARP-BOUNDARY-INPUT-1.0",
                "MPK-CSHARP-BOUNDARY-OUTPUT-1.0",
                "MPK-CSHARP-CANONICAL-VALUE-1.0",
                "MPK-CSHARP-LIMITS-1.0",
                "MPK-CSHARP-METHOD-CONTRACT-1.0",
                "MPK-CSHARP-OPERATIONS-1.0",
                "MPK-CSHARP-REQUIRED-CHECKS-1.0",
                "MPK-CSHARP-SEMANTIC-BINDING-SET-1.0",
                "MPK-CSHARP-TRANSITION-CONTRACT-1.0",
                "MPK-CSHARP-TYPE-CONTRACT-1.0",
            ],
        ),
        identity(
            "source_artifact",
            &[SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA],
            &["MPK-FRONTEND-SOURCE-ARTIFACTS-2.0"],
        ),
        identity(
            "foundation",
            &[
                "mpk.csharp.closed_instances.v1",
                "mpk.csharp.foundation_definitions.v1",
                "mpk.csharp.foundation_descriptor.v1",
                "mpk.csharp.foundation_expansion.v1",
                "mpk.csharp.practical.foundation.v1",
                "mpk.csharp.semantic_binding.v1",
            ],
            &[
                "MPK-CSHARP-CLOSED-INSTANCES-1.0",
                "MPK-CSHARP-DECLARATION-1.0",
                "MPK-CSHARP-DECLARATION-PROVENANCE-1.0",
                "MPK-CSHARP-FOUNDATION-MEMBER-1.0",
                "MPK-CSHARP-PRACTICAL-FOUNDATION-1.0",
                "MPK-CSHARP-SEMANTIC-BINDING-1.0",
                "MPK-CSHARP-SEMANTIC-INSTANCE-1.0",
            ],
        ),
        identity("vir", &[SUCCESSOR_VIR_SCHEMA], &["MPK-VIR-2.0"]),
        identity(
            "frontend_protocol",
            &[
                "mpk.frontend.cli.v2",
                SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA,
                SUCCESSOR_FRONTEND_REQUEST_SCHEMA,
                SUCCESSOR_FRONTEND_SUCCESS_SCHEMA,
            ],
            &[
                "MPK-FRONTEND-DIAGNOSTIC-2.0",
                "MPK-FRONTEND-REQUEST-2.0",
                "MPK-FRONTEND-SUCCESS-2.0",
            ],
        ),
        identity(
            "source_map",
            &[SUCCESSOR_SOURCE_MAP_SCHEMA],
            &["MPK-SOURCE-MAP-2.0"],
        ),
        identity(
            "source_manifest",
            &[
                "mpk.source_manifest.certificate.v2",
                SUCCESSOR_FRONTEND_MANIFEST_SCHEMA,
            ],
            &["MPK-SOURCE-MANIFEST-2.0"],
        ),
        identity(
            "vc_skeleton",
            &[
                PRIVATE_PREDECESSOR_SKELETON_SCHEMA,
                PRIVATE_PREDECESSOR_VC_SCHEMA,
            ],
            &["MPK-VC-3.0"],
        ),
        identity(
            "release",
            &[
                "mpk.release.bundle_candidate.v2",
                "mpk.release.bundle_inventory.v1",
                "mpk.release.bundle_registry.v2",
                "mpk.release.evidence.v2",
                "mpk.release.frontend_bundle.v2",
                "mpk.release.receipt.v2",
                "mpk.release.registry.v2",
                "mpk.release.toolchain_bundle.v2",
            ],
            &[
                "MPK-BUNDLE-CONTENT-1.0",
                "MPK-BUNDLE-REGISTRY-2.0",
                "MPK-RELEASE-RECEIPT-2.0",
                "MPK-RELEASE-REGISTRY-2.0",
            ],
        ),
        identity(
            "policy_evidence",
            &[
                PRIVATE_POLICY_EVIDENCE_SCHEMA,
                PRIVATE_POLICY_RECEIPT_SCHEMA,
                PRIVATE_POLICY_REPRODUCTION_SCHEMA,
                PRIVATE_POLICY_SCAN_SCHEMA,
            ],
            &[
                "MPK-POLICY-EVIDENCE-3.0",
                "MPK-POLICY-RECEIPT-3.0",
                "MPK-POLICY-REPRODUCTION-3.0",
            ],
        ),
        identity(
            "program_assembly",
            &[PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA],
            &["MPK-PROGRAM-ASSEMBLY-2.0"],
        ),
        identity(
            "ai",
            &[PRIVATE_AI_REQUEST_SCHEMA, PRIVATE_AI_EXPLANATION_SCHEMA],
            &[],
        ),
        identity(
            "api",
            &[
                PRIVATE_SUCCESSOR_API_REQUEST_SCHEMA,
                PRIVATE_SUCCESSOR_API_RESPONSE_SCHEMA,
                PRIVATE_SUCCESSOR_API_SESSION_SCHEMA,
                "mpk.ai.api.v3",
            ],
            &[],
        ),
    ]
}

/// Executes the four frozen W09 identity-vector semantics in production code.
pub fn validate_private_successor_identity_inventory(
    families: &[PrivateIdentityFamily],
    observed: Option<PrivateObservedArtifactIdentity<'_>>,
) -> Result<(), PrivateIdentityCode> {
    if families.len() != PRIVATE_CONSUMER_FAMILIES.len() {
        return Err(PrivateIdentityCode::Inventory);
    }
    let expected = PRIVATE_CONSUMER_FAMILIES
        .into_iter()
        .collect::<BTreeSet<_>>();
    let actual = families
        .iter()
        .map(|family| family.family.as_str())
        .collect::<BTreeSet<_>>();
    if actual != expected
        || families.iter().any(|family| {
            family.successor_identities.is_empty()
                || family
                    .successor_identities
                    .iter()
                    .any(|value| value.is_empty())
                || family
                    .successor_hash_domains
                    .iter()
                    .any(|value| value.is_empty())
        })
    {
        return Err(PrivateIdentityCode::Inventory);
    }
    let mut identities = BTreeSet::new();
    for value in families
        .iter()
        .flat_map(|family| family.successor_identities.iter())
    {
        if !identities.insert(value.as_str()) {
            return Err(PrivateIdentityCode::DuplicateSuccessorIdentity);
        }
    }
    let mut domains = BTreeSet::new();
    for value in families
        .iter()
        .flat_map(|family| family.successor_hash_domains.iter())
    {
        if !domains.insert(value.as_str()) {
            return Err(PrivateIdentityCode::DuplicateSuccessorHashDomain);
        }
    }
    if families != private_successor_identity_inventory() {
        return Err(PrivateIdentityCode::Inventory);
    }
    if let Some(observed) = observed {
        let valid = observed.semantic_context_schema == SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA
            && families.iter().any(|family| {
                family.family == observed.family
                    && family
                        .successor_identities
                        .iter()
                        .any(|identity| identity == observed.artifact_schema)
            });
        if !valid {
            return Err(PrivateIdentityCode::MixedArtifactFamily);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateConsumerMigrationCode {
    Identity,
    Artifact,
    Linkage,
    Release,
    Verification,
    Certificate,
    Policy,
    Ai,
    Api,
    Cli,
    Fixture,
    Receipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateConsumerMigrationError {
    code: PrivateConsumerMigrationCode,
    detail: &'static str,
}

impl PrivateConsumerMigrationError {
    pub const fn code(&self) -> PrivateConsumerMigrationCode {
        self.code
    }

    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

impl fmt::Display for PrivateConsumerMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "private consumer migration {:?}: {}",
            self.code, self.detail
        )
    }
}

impl Error for PrivateConsumerMigrationError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivateConsumerMigrationOutcome {
    Accepted(Box<PrivateAcceptedConsumerMigration>),
    Rejected(PrivateRejectedConsumerMigration),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateAcceptedConsumerMigration {
    receipt: Vec<u8>,
    receipt_sha256: String,
    verification: PrivatePredecessorVerificationArtifacts,
    certificate_sha256: String,
    policy: PrivatePolicyArtifacts,
    ai: PrivateAiArtifacts,
    api: PrivateSuccessorApiExchange,
}

impl PrivateAcceptedConsumerMigration {
    pub fn receipt(&self) -> &[u8] {
        &self.receipt
    }

    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }

    pub fn verification(&self) -> &PrivatePredecessorVerificationArtifacts {
        &self.verification
    }

    pub fn certificate_sha256(&self) -> &str {
        &self.certificate_sha256
    }

    pub fn policy_documents(&self) -> [&[u8]; 4] {
        [
            self.policy.scan(),
            self.policy.evidence(),
            self.policy.reproduction(),
            self.policy.receipt(),
        ]
    }

    pub fn ai_documents(&self) -> [&[u8]; 2] {
        [self.ai.request(), self.ai.explanation()]
    }

    pub fn api_documents(&self) -> [&[u8]; 3] {
        [self.api.request(), self.api.session(), self.api.response()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateRejectedConsumerMigration {
    request_sha256: String,
    diagnostic_sha256: String,
    receipt: Vec<u8>,
    receipt_sha256: String,
}

impl PrivateRejectedConsumerMigration {
    pub fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub fn diagnostic_sha256(&self) -> &str {
        &self.diagnostic_sha256
    }

    pub fn receipt(&self) -> &[u8] {
        &self.receipt
    }

    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiptArtifactRef {
    schema: String,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiptEquivalence {
    source_behavior_sha256: String,
    obligation_sha256: String,
    verdict_sha256: String,
    axiom_count: u64,
    practical_foundation_instances: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiptGuarantees {
    public_route_added: bool,
    release_candidate_materialized: bool,
    dual_format_fallback: bool,
    certificate_v0_retained: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptedReceipt {
    schema: String,
    work_item: String,
    status: String,
    producer: String,
    semantic_profile: String,
    consumer_families: Vec<String>,
    inventory_edges: Vec<String>,
    release_registry: ReceiptArtifactRef,
    frontend_result: ReceiptArtifactRef,
    source_artifacts: ReceiptArtifactRef,
    source_ir: ReceiptArtifactRef,
    source_map: ReceiptArtifactRef,
    source_manifest: ReceiptArtifactRef,
    source_vc: ReceiptArtifactRef,
    source_skeleton: ReceiptArtifactRef,
    program_assembly: ReceiptArtifactRef,
    certificate_sha256: String,
    policy_scan: ReceiptArtifactRef,
    policy_evidence: ReceiptArtifactRef,
    policy_reproduction: ReceiptArtifactRef,
    policy_receipt: ReceiptArtifactRef,
    ai_request: ReceiptArtifactRef,
    ai_explanation: ReceiptArtifactRef,
    api_request: ReceiptArtifactRef,
    api_session: ReceiptArtifactRef,
    api_response: ReceiptArtifactRef,
    equivalence: ReceiptEquivalence,
    guarantees: ReceiptGuarantees,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RejectedReceipt {
    schema: String,
    work_item: String,
    status: String,
    producer: String,
    semantic_profile: String,
    request: ReceiptArtifactRef,
    diagnostic: ReceiptArtifactRef,
    downstream_artifact_count: u64,
    guarantees: ReceiptGuarantees,
}

/// Closes all W09-owned private consumer edges without installing a route.
pub fn close_private_successor_consumers(
    registry: &ValidatedSuccessorRegistry,
    release_transport: &[u8],
    migration: &PrivatePredecessorMigration,
    certificate_bytes: &[u8],
) -> Result<PrivateConsumerMigrationOutcome, PrivateConsumerMigrationError> {
    validate_private_successor_identity_inventory(&private_successor_identity_inventory(), None)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Identity, "identity inventory"))?;
    let release = crate::frontend_registry::validate_private_csharp_practical_release(
        release_transport,
        registry,
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Release, "release registry"))?;
    let request = validate_artifact(migration.request())?;
    let semantic_request = request
        .get("semantic_request")
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "semantic request"))?;
    let semantic_request_bytes = canonical_successor_value(semantic_request)?;
    let validated_request = validate_successor_semantic_request(registry, &semantic_request_bytes)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Identity, "semantic request"))?;
    if validated_request.semantic_context().semantic_profile()
        != migration.producer().semantic_profile()
        || !release
            .tuple_profiles()
            .iter()
            .any(|profile| profile == validated_request.semantic_context().semantic_profile())
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Identity,
            "producer or release tuple",
        ));
    }
    let semantic_context = semantic_request
        .get("semantic_context")
        .cloned()
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "semantic context"))?;
    let frontend =
        crate::successor_frontend_runner::consume_private_successor_frontend_result(migration)
            .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "frontend result"))?;
    if frontend.result_sha256() != migration.frontend_result().sha256() {
        return Err(failure(
            PrivateConsumerMigrationCode::Linkage,
            "frontend result receipt",
        ));
    }

    if migration.frontend_result().schema() == SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA {
        validate_diagnostic_linkage(migration, &request, &semantic_context)?;
        let receipt = RejectedReceipt {
            schema: PRIVATE_CONSUMER_RECEIPT_SCHEMA.to_owned(),
            work_item: PRIVATE_CONSUMER_WORK_ITEM.to_owned(),
            status: "rejected_artifact_free".to_owned(),
            producer: producer_name(migration.producer()).to_owned(),
            semantic_profile: migration.producer().semantic_profile().to_owned(),
            request: receipt_ref(migration.request()),
            diagnostic: receipt_ref(migration.frontend_result()),
            downstream_artifact_count: 0,
            guarantees: guarantees(),
        };
        let receipt = encode_receipt(&receipt)?;
        return Ok(PrivateConsumerMigrationOutcome::Rejected(
            PrivateRejectedConsumerMigration {
                request_sha256: migration.request().sha256().to_owned(),
                diagnostic_sha256: migration.frontend_result().sha256().to_owned(),
                receipt_sha256: sha256_raw_file_bytes(&receipt).to_hex(),
                receipt,
            },
        ));
    }

    let artifacts = migration
        .artifacts()
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Linkage, "success artifacts"))?;
    if frontend.artifacts_sha256() != Some(artifacts.source_artifacts().sha256()) {
        return Err(failure(
            PrivateConsumerMigrationCode::Linkage,
            "frontend artifact receipt",
        ));
    }
    let graph = validate_success_artifact_graph(migration, &semantic_context)?;
    crate::successor_frontend_protocol::consume_private_successor_source_artifacts(
        artifacts.source_artifacts(),
    )
    .map_err(|_| {
        failure(
            PrivateConsumerMigrationCode::Artifact,
            "source artifact consumer",
        )
    })?;

    let verification_source = PrivatePredecessorVerificationSource {
        semantic_context: &semantic_context,
        source_ir: PrivateVerificationArtifactRef {
            schema: artifacts.vir().schema(),
            sha256: artifacts.vir().sha256(),
            canonical_bytes: artifacts.vir().canonical_bytes().len() as u64,
        },
        source_manifest: PrivateVerificationArtifactRef {
            schema: artifacts.source_manifest().schema(),
            sha256: artifacts.source_manifest().sha256(),
            canonical_bytes: artifacts.source_manifest().canonical_bytes().len() as u64,
        },
        obligation_sha256: migration.equivalence().obligation_sha256(),
        verdict_sha256: migration.equivalence().verdict_sha256(),
        axiom_count: migration.equivalence().axiom_count(),
    };
    let verification = build_private_predecessor_verification_artifacts(verification_source)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Verification, "VC assembly"))?;
    validate_private_predecessor_verification_artifacts(verification_source, &verification)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Verification, "VC re-import"))?;
    validate_private_predecessor_verification_transports(
        verification_source,
        verification.vc().canonical_bytes(),
        verification.skeleton().canonical_bytes(),
        verification.assembly().canonical_bytes(),
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Verification, "VC transport"))?;
    let certificate = validate_private_successor_program_consumer(
        verification.assembly().canonical_bytes(),
        verification.assembly().sha256(),
        certificate_bytes,
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Certificate, "Certificate v0"))?;
    if certificate.axiom_count() != migration.equivalence().axiom_count()
        || certificate.foundation_content_sha256() != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Certificate,
            "foundation or axiom linkage",
        ));
    }

    let policy = build_private_successor_policy(PrivatePolicySource {
        semantic_context: &semantic_context,
        source_ir_sha256: artifacts.vir().sha256(),
        source_map_sha256: artifacts.source_map().sha256(),
        source_manifest_sha256: artifacts.source_manifest().sha256(),
        source_artifacts_sha256: artifacts.source_artifacts().sha256(),
        source_vc_sha256: verification.vc().sha256(),
        source_skeleton_sha256: verification.skeleton().sha256(),
        program_assembly_sha256: verification.assembly().sha256(),
        certificate_sha256: certificate.certificate_sha256(),
        release_registry_sha256: release.registry_sha256(),
    })
    .map_err(|_| failure(PrivateConsumerMigrationCode::Policy, "policy artifacts"))?;
    let ai = build_private_successor_ai_artifacts(
        &semantic_context,
        policy.evidence_sha256(),
        policy.receipt_sha256(),
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Ai, "AI artifacts"))?;
    let api = build_private_successor_api_exchange(
        &semantic_context,
        PrivateSuccessorApiArtifactRef {
            schema: artifacts.vir().schema(),
            sha256: artifacts.vir().sha256(),
        },
        PrivateSuccessorApiArtifactRef {
            schema: verification.vc().schema(),
            sha256: verification.vc().sha256(),
        },
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Api, "API exchange"))?;
    let cli = crate::successor_cli::prepare_private_successor_cli_consumer(
        migration.producer().semantic_profile(),
        frontend.result_schema(),
        PRIVATE_POLICY_SCAN_SCHEMA,
        PRIVATE_POLICY_EVIDENCE_SCHEMA,
        PRIVATE_AI_REQUEST_SCHEMA,
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Cli, "CLI consumer"))?;
    if cli.semantic_profile() != migration.producer().semantic_profile()
        || cli.consumed_schemas().len() != 4
    {
        return Err(failure(PrivateConsumerMigrationCode::Cli, "CLI receipt"));
    }
    let member_paths = release_member_paths(release_transport)?;
    let member_refs = member_paths.iter().map(String::as_str).collect::<Vec<_>>();
    let fixture = crate::frontend_sandbox::prepare_private_consumer_fixture(&member_refs)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Fixture, "release fixture"))?;
    if fixture.members().len() != release.member_count() {
        return Err(failure(
            PrivateConsumerMigrationCode::Fixture,
            "release member accounting",
        ));
    }

    let receipt = AcceptedReceipt {
        schema: PRIVATE_CONSUMER_RECEIPT_SCHEMA.to_owned(),
        work_item: PRIVATE_CONSUMER_WORK_ITEM.to_owned(),
        status: "accepted_private_consumer_closure".to_owned(),
        producer: producer_name(migration.producer()).to_owned(),
        semantic_profile: migration.producer().semantic_profile().to_owned(),
        consumer_families: PRIVATE_CONSUMER_FAMILIES
            .into_iter()
            .map(str::to_owned)
            .collect(),
        inventory_edges: PRIVATE_CONSUMER_INVENTORY_EDGES
            .into_iter()
            .map(str::to_owned)
            .collect(),
        release_registry: ReceiptArtifactRef {
            schema: "mpk.release.bundle_registry.v2".to_owned(),
            sha256: release.registry_sha256().to_owned(),
        },
        frontend_result: receipt_ref(migration.frontend_result()),
        source_artifacts: receipt_ref(artifacts.source_artifacts()),
        source_ir: receipt_ref(artifacts.vir()),
        source_map: receipt_ref(artifacts.source_map()),
        source_manifest: receipt_ref(artifacts.source_manifest()),
        source_vc: verification_ref(verification.vc()),
        source_skeleton: verification_ref(verification.skeleton()),
        program_assembly: verification_ref(verification.assembly()),
        certificate_sha256: certificate.certificate_sha256().to_owned(),
        policy_scan: bytes_ref(PRIVATE_POLICY_SCAN_SCHEMA, policy.scan_sha256()),
        policy_evidence: bytes_ref(PRIVATE_POLICY_EVIDENCE_SCHEMA, policy.evidence_sha256()),
        policy_reproduction: bytes_ref(
            PRIVATE_POLICY_REPRODUCTION_SCHEMA,
            policy.reproduction_sha256(),
        ),
        policy_receipt: bytes_ref(PRIVATE_POLICY_RECEIPT_SCHEMA, policy.receipt_sha256()),
        ai_request: bytes_ref(PRIVATE_AI_REQUEST_SCHEMA, ai.request_sha256()),
        ai_explanation: bytes_ref(PRIVATE_AI_EXPLANATION_SCHEMA, ai.explanation_sha256()),
        api_request: bytes_ref(PRIVATE_SUCCESSOR_API_REQUEST_SCHEMA, api.request_sha256()),
        api_session: bytes_ref(PRIVATE_SUCCESSOR_API_SESSION_SCHEMA, api.session_sha256()),
        api_response: bytes_ref(PRIVATE_SUCCESSOR_API_RESPONSE_SCHEMA, api.response_sha256()),
        equivalence: ReceiptEquivalence {
            source_behavior_sha256: migration.equivalence().source_behavior_sha256().to_owned(),
            obligation_sha256: migration.equivalence().obligation_sha256().to_owned(),
            verdict_sha256: migration.equivalence().verdict_sha256().to_owned(),
            axiom_count: migration.equivalence().axiom_count(),
            practical_foundation_instances: migration
                .equivalence()
                .practical_foundation_instances(),
        },
        guarantees: guarantees(),
    };
    let receipt = encode_receipt(&receipt)?;
    validate_accepted_receipt(&receipt, &graph)?;
    Ok(PrivateConsumerMigrationOutcome::Accepted(Box::new(
        PrivateAcceptedConsumerMigration {
            receipt_sha256: sha256_raw_file_bytes(&receipt).to_hex(),
            receipt,
            verification,
            certificate_sha256: certificate.certificate_sha256().to_owned(),
            policy,
            ai,
            api,
        },
    )))
}

/// Strict validator exposed for artifact mutation tests. Only W08 successor
/// artifact schemas are recognized; there is deliberately no old-format arm.
pub fn validate_private_successor_artifact_transport(
    schema: &str,
    expected_sha256: &str,
    input: &[u8],
) -> Result<(), PrivateConsumerMigrationError> {
    let spec = artifact_spec(schema)
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "unknown schema"))?;
    validate_artifact_parts(schema, expected_sha256, input, spec).map(|_| ())
}

pub fn validate_private_policy_documents(
    scan: &[u8],
    evidence: &[u8],
    reproduction: &[u8],
    receipt: &[u8],
) -> Result<(), PrivateConsumerMigrationError> {
    validate_private_successor_policy(scan, evidence, reproduction, receipt)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Policy, "policy re-import"))
}

pub fn validate_private_ai_documents(
    request: &[u8],
    explanation: &[u8],
) -> Result<(), PrivateConsumerMigrationError> {
    validate_private_successor_ai_artifacts(request, explanation)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Ai, "AI re-import"))
}

pub fn validate_private_api_documents(
    request: &[u8],
    session: &[u8],
    response: &[u8],
) -> Result<(), PrivateConsumerMigrationError> {
    validate_private_successor_api_exchange(request, session, response)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Api, "API re-import"))
}

struct ValidatedGraph {
    semantic_context: Value,
}

fn validate_success_artifact_graph(
    migration: &PrivatePredecessorMigration,
    semantic_context: &Value,
) -> Result<ValidatedGraph, PrivateConsumerMigrationError> {
    let artifacts = migration
        .artifacts()
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Linkage, "artifact graph"))?;
    let vir = validate_artifact(artifacts.vir())?;
    let source_map = validate_artifact(artifacts.source_map())?;
    let source_manifest = validate_artifact(artifacts.source_manifest())?;
    let semantic_bindings = validate_artifact(artifacts.semantic_bindings())?;
    let closed_instances = validate_artifact(artifacts.closed_instances())?;
    let source_artifacts = validate_artifact(artifacts.source_artifacts())?;
    let success = validate_artifact(migration.frontend_result())?;
    let semantic_request_bytes =
        canonical_artifact_field(migration.request().canonical_bytes(), "semantic_request")?;
    let selection_bytes = canonical_artifact_field(&semantic_request_bytes, "selection")?;
    let selection_sha256 = sha256_raw_file_bytes(&selection_bytes).to_hex();
    let contexts = [
        &vir["semantic_context"],
        &source_map["semantic_context"],
        &source_manifest["semantic_context"],
        &semantic_bindings["semantic_context"],
        &source_artifacts["semantic_context"],
        &success["semantic_context"],
    ];
    if contexts
        .into_iter()
        .any(|context| context != semantic_context)
        || source_map["selection_sha256"] != selection_sha256
        || source_manifest["selection_sha256"] != selection_sha256
        || source_artifacts["selection_sha256"] != selection_sha256
        || closed_instances["semantic_profile"] != migration.producer().semantic_profile()
        || closed_instances["foundation_id"] != FOUNDATION_DESCRIPTOR_ID
        || closed_instances["foundation_sha256"] != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
        || closed_instances["instances"].as_array().map(Vec::len)
            != Some(migration.equivalence().practical_foundation_instances() as usize)
        || semantic_bindings["compilation_id"]
            != format!("predecessor.{}", migration.producer().report_stem())
        || !semantic_bindings["bindings"]
            .as_array()
            .is_some_and(Vec::is_empty)
        || !source_artifacts["boundary_contracts"]
            .as_array()
            .is_some_and(Vec::is_empty)
        || !source_artifacts["transition_contracts"]
            .as_array()
            .is_some_and(Vec::is_empty)
        || !foundation_matches(&source_manifest["foundation_descriptor"])
        || !foundation_matches(&source_artifacts["foundation_descriptor"])
        || success["request_sha256"] != migration.request().sha256()
        || success["artifacts"] != source_artifacts
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Linkage,
            "context, selection, foundation, or result",
        ));
    }
    for (actual, expected) in [
        (&source_map["vir"], artifacts.vir()),
        (
            &source_manifest["semantic_bindings"],
            artifacts.semantic_bindings(),
        ),
        (
            &source_manifest["closed_instances"],
            artifacts.closed_instances(),
        ),
        (&source_manifest["vir"], artifacts.vir()),
        (&source_manifest["source_map"], artifacts.source_map()),
        (&source_artifacts["vir"], artifacts.vir()),
        (&source_artifacts["source_map"], artifacts.source_map()),
        (
            &source_artifacts["source_manifest"],
            artifacts.source_manifest(),
        ),
        (
            &source_artifacts["semantic_bindings"],
            artifacts.semantic_bindings(),
        ),
        (
            &source_artifacts["closed_instances"],
            artifacts.closed_instances(),
        ),
    ] {
        if !reference_matches(actual, expected) {
            return Err(failure(
                PrivateConsumerMigrationCode::Linkage,
                "artifact reference",
            ));
        }
    }
    Ok(ValidatedGraph {
        semantic_context: semantic_context.clone(),
    })
}

fn validate_diagnostic_linkage(
    migration: &PrivatePredecessorMigration,
    _request: &Value,
    semantic_context: &Value,
) -> Result<(), PrivateConsumerMigrationError> {
    if migration.artifacts().is_some() {
        return Err(failure(
            PrivateConsumerMigrationCode::Linkage,
            "diagnostic artifacts",
        ));
    }
    let diagnostic = validate_artifact(migration.frontend_result())?;
    let linkage = &diagnostic["request_linkage"];
    if diagnostic["raw_request_sha256"]
        != sha256_raw_file_bytes(migration.request().canonical_bytes()).to_hex()
        || diagnostic["raw_request_size_bytes"] != migration.request().canonical_bytes().len()
        || linkage["state"] != "validated"
        || linkage["request_sha256"] != migration.request().sha256()
        || linkage["semantic_context"] != *semantic_context
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Linkage,
            "diagnostic request",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ArtifactSpec {
    hash_field: &'static str,
    hash_domain: HashDomain,
}

fn artifact_spec(schema: &str) -> Option<ArtifactSpec> {
    Some(match schema {
        SUCCESSOR_FRONTEND_REQUEST_SCHEMA => ArtifactSpec {
            hash_field: "request_sha256",
            hash_domain: REQUEST_HASH_DOMAIN,
        },
        SUCCESSOR_FRONTEND_SUCCESS_SCHEMA => ArtifactSpec {
            hash_field: "success_sha256",
            hash_domain: SUCCESS_HASH_DOMAIN,
        },
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA => ArtifactSpec {
            hash_field: "diagnostic_sha256",
            hash_domain: DIAGNOSTIC_HASH_DOMAIN,
        },
        SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA => ArtifactSpec {
            hash_field: "artifacts_sha256",
            hash_domain: SOURCE_ARTIFACTS_HASH_DOMAIN,
        },
        SUCCESSOR_VIR_SCHEMA => ArtifactSpec {
            hash_field: "vir_sha256",
            hash_domain: VIR_HASH_DOMAIN,
        },
        SUCCESSOR_SOURCE_MAP_SCHEMA => ArtifactSpec {
            hash_field: "source_map_sha256",
            hash_domain: SOURCE_MAP_HASH_DOMAIN,
        },
        SUCCESSOR_FRONTEND_MANIFEST_SCHEMA => ArtifactSpec {
            hash_field: "manifest_sha256",
            hash_domain: SOURCE_MANIFEST_HASH_DOMAIN,
        },
        SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA => ArtifactSpec {
            hash_field: "binding_set_sha256",
            hash_domain: SEMANTIC_BINDINGS_HASH_DOMAIN,
        },
        SUCCESSOR_CLOSED_INSTANCES_SCHEMA => ArtifactSpec {
            hash_field: "closed_set_sha256",
            hash_domain: CLOSED_INSTANCES_HASH_DOMAIN,
        },
        _ => return None,
    })
}

fn validate_artifact(
    artifact: &PrivateSuccessorArtifact,
) -> Result<Value, PrivateConsumerMigrationError> {
    let spec = artifact_spec(artifact.schema())
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "artifact schema"))?;
    validate_artifact_parts(
        artifact.schema(),
        artifact.sha256(),
        artifact.canonical_bytes(),
        spec,
    )
}

fn validate_artifact_parts(
    schema: &str,
    expected_sha256: &str,
    input: &[u8],
    spec: ArtifactSpec,
) -> Result<Value, PrivateConsumerMigrationError> {
    parse_strict_json(input, TRANSPORT_LIMITS)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "strict JSON"))?;
    let practical =
        parse_canonical_practical_json(PracticalArtifactKind::SourceArtifacts, input)
            .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "canonical JSON"))?;
    let value: Value = serde_json::from_slice(input)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "JSON shape"))?;
    let fields = practical
        .as_object()
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "artifact object"))?;
    let expected_fields = artifact_fields(schema)
        .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "artifact fields"))?;
    if !fields
        .iter()
        .map(|(name, _)| name.as_str())
        .eq(expected_fields.iter().copied())
        || practical.get("schema").and_then(PracticalJsonValue::as_str) != Some(schema)
        || !valid_sha256(expected_sha256)
        || practical
            .get(spec.hash_field)
            .and_then(PracticalJsonValue::as_str)
            != Some(expected_sha256)
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Artifact,
            "canonical identity",
        ));
    }
    let preimage = canonical_practical_json_bytes(&PracticalJsonValue::Object(
        fields[..fields.len() - 1].to_vec(),
    ))
    .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "hash preimage"))?;
    let actual = hash_domain_separated_raw(spec.hash_domain, &preimage)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "hash domain"))?
        .to_hex();
    if actual != expected_sha256 {
        return Err(failure(
            PrivateConsumerMigrationCode::Artifact,
            "artifact hash",
        ));
    }
    if !artifact_value_shape(schema, &value) {
        return Err(failure(
            PrivateConsumerMigrationCode::Artifact,
            "artifact shape",
        ));
    }
    if let Some(kind) = match schema {
        SUCCESSOR_FRONTEND_SUCCESS_SCHEMA => Some(PrivateSuccessorSchemaKind::FrontendSuccess),
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA => {
            Some(PrivateSuccessorSchemaKind::FrontendDiagnostic)
        }
        SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA => {
            Some(PrivateSuccessorSchemaKind::FrontendSourceArtifacts)
        }
        _ => None,
    } {
        validate_private_successor_schema(kind, input).map_err(|_| {
            failure(
                PrivateConsumerMigrationCode::Artifact,
                "nested artifact shape",
            )
        })?;
    }
    Ok(value)
}

fn artifact_fields(schema: &str) -> Option<&'static [&'static str]> {
    Some(match schema {
        SUCCESSOR_FRONTEND_REQUEST_SCHEMA => &[
            "schema",
            "semantic_request",
            "source_snapshot",
            "sidecars",
            "request_sha256",
        ],
        SUCCESSOR_FRONTEND_SUCCESS_SCHEMA => &[
            "schema",
            "request_sha256",
            "semantic_context",
            "artifacts",
            "success_sha256",
        ],
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA => &[
            "schema",
            "raw_request_sha256",
            "raw_request_size_bytes",
            "request_linkage",
            "status",
            "phase",
            "diagnostics",
            "diagnostic_sha256",
        ],
        SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA => &[
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
        SUCCESSOR_VIR_SCHEMA => &["schema", "semantic_context", "units", "vir_sha256"],
        SUCCESSOR_SOURCE_MAP_SCHEMA => &[
            "schema",
            "semantic_context",
            "selection_sha256",
            "source_snapshot_sha256",
            "vir",
            "entries",
            "source_map_sha256",
        ],
        SUCCESSOR_FRONTEND_MANIFEST_SCHEMA => &[
            "schema",
            "semantic_context",
            "selection_sha256",
            "inputs",
            "input_set_sha256",
            "limit_profile",
            "release_registry",
            "toolchain",
            "frontend",
            "units",
            "target",
            "foundation_descriptor",
            "semantic_bindings",
            "closed_instances",
            "vir",
            "source_map",
            "manifest_sha256",
        ],
        SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA => &[
            "schema",
            "semantic_context",
            "compilation_id",
            "bindings",
            "binding_set_sha256",
        ],
        SUCCESSOR_CLOSED_INSTANCES_SCHEMA => &[
            "schema",
            "semantic_profile",
            "foundation_id",
            "foundation_sha256",
            "instances",
            "closed_set_sha256",
        ],
        _ => return None,
    })
}

fn artifact_value_shape(schema: &str, value: &Value) -> bool {
    match schema {
        SUCCESSOR_FRONTEND_REQUEST_SCHEMA => {
            successor_registry_document_shape(
                &value["semantic_request"],
                SuccessorRegistryDocumentKind::ValidatedRequest,
            ) && source_snapshot_shape(&value["source_snapshot"])
                && sidecar_set_shape(&value["sidecars"])
        }
        SUCCESSOR_FRONTEND_SUCCESS_SCHEMA => {
            lower_sha256_value(&value["request_sha256"])
                && successor_context_shape(&value["semantic_context"])
                && value["artifacts"].is_object()
        }
        SUCCESSOR_FRONTEND_DIAGNOSTIC_SCHEMA => value["diagnostics"].is_array(),
        SUCCESSOR_SOURCE_ARTIFACTS_SCHEMA => {
            successor_context_shape(&value["semantic_context"])
                && lower_sha256_value(&value["selection_sha256"])
                && artifact_reference_shape(&value["vir"], SUCCESSOR_VIR_SCHEMA)
                && artifact_reference_shape(&value["source_map"], SUCCESSOR_SOURCE_MAP_SCHEMA)
                && artifact_reference_shape(
                    &value["source_manifest"],
                    SUCCESSOR_FRONTEND_MANIFEST_SCHEMA,
                )
                && artifact_reference_shape(
                    &value["semantic_bindings"],
                    SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA,
                )
                && artifact_reference_shape(
                    &value["closed_instances"],
                    SUCCESSOR_CLOSED_INSTANCES_SCHEMA,
                )
                && foundation_matches(&value["foundation_descriptor"])
                && artifact_reference_array_shape(
                    &value["boundary_contracts"],
                    "mpk.csharp.boundary.v1",
                )
                && artifact_reference_array_shape(
                    &value["transition_contracts"],
                    "mpk.csharp.transition.v1",
                )
        }
        SUCCESSOR_VIR_SCHEMA => {
            successor_context_shape(&value["semantic_context"]) && value["units"].is_array()
        }
        SUCCESSOR_SOURCE_MAP_SCHEMA => {
            successor_context_shape(&value["semantic_context"])
                && lower_sha256_value(&value["selection_sha256"])
                && lower_sha256_value(&value["source_snapshot_sha256"])
                && artifact_reference_shape(&value["vir"], SUCCESSOR_VIR_SCHEMA)
                && value["entries"].is_array()
        }
        SUCCESSOR_FRONTEND_MANIFEST_SCHEMA => {
            successor_context_shape(&value["semantic_context"])
                && lower_sha256_value(&value["selection_sha256"])
                && value["inputs"].is_array()
                && lower_sha256_value(&value["input_set_sha256"])
                && value["limit_profile"].is_string()
                && value["release_registry"].is_object()
                && value["toolchain"].is_object()
                && value["frontend"].is_object()
                && value["units"].is_array()
                && value["target"].is_object()
                && foundation_matches(&value["foundation_descriptor"])
                && artifact_reference_shape(
                    &value["semantic_bindings"],
                    SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA,
                )
                && artifact_reference_shape(
                    &value["closed_instances"],
                    SUCCESSOR_CLOSED_INSTANCES_SCHEMA,
                )
                && artifact_reference_shape(&value["vir"], SUCCESSOR_VIR_SCHEMA)
                && artifact_reference_shape(&value["source_map"], SUCCESSOR_SOURCE_MAP_SCHEMA)
        }
        SUCCESSOR_SEMANTIC_BINDINGS_SCHEMA => {
            successor_context_shape(&value["semantic_context"])
                && value["compilation_id"]
                    .as_str()
                    .is_some_and(|id| !id.is_empty() && id.len() <= 1_024)
                && value["bindings"].is_array()
        }
        SUCCESSOR_CLOSED_INSTANCES_SCHEMA => {
            value["semantic_profile"]
                .as_str()
                .is_some_and(valid_canonical_id)
                && value["foundation_id"] == FOUNDATION_DESCRIPTOR_ID
                && value["foundation_sha256"] == FOUNDATION_DESCRIPTOR_CONTENT_SHA256
                && value["instances"].is_array()
        }
        _ => false,
    }
}

fn successor_registry_document_shape(value: &Value, kind: SuccessorRegistryDocumentKind) -> bool {
    serde_json::to_vec(value)
        .is_ok_and(|transport| validate_successor_registry_document(kind, &transport).is_ok())
}

fn successor_context_shape(value: &Value) -> bool {
    successor_registry_document_shape(value, SuccessorRegistryDocumentKind::SemanticContext)
}

fn source_snapshot_shape(value: &Value) -> bool {
    exact_value_fields(value, &["entries", "snapshot_sha256"])
        && lower_sha256_value(&value["snapshot_sha256"])
        && value["entries"].as_array().is_some_and(|entries| {
            strictly_ordered_paths(entries, |entry| {
                if exact_value_fields(entry, &["path", "raw_sha256", "size_bytes"])
                    && normalized_relative_path(&entry["path"])
                    && lower_sha256_value(&entry["raw_sha256"])
                    && entry["size_bytes"]
                        .as_u64()
                        .is_some_and(|size| u32::try_from(size).is_ok())
                {
                    entry["path"].as_str()
                } else {
                    None
                }
            })
        })
}

fn sidecar_set_shape(value: &Value) -> bool {
    exact_value_fields(value, &["entries", "set_sha256"])
        && lower_sha256_value(&value["set_sha256"])
        && value["entries"]
            .as_array()
            .is_some_and(|entries| strictly_ordered_sidecars(entries))
}

fn strictly_ordered_sidecars(entries: &[Value]) -> bool {
    let mut previous: Option<(&str, &str, &str)> = None;
    let mut paths = BTreeSet::new();
    for entry in entries {
        let Some(schema) = entry["schema"].as_str() else {
            return false;
        };
        let Some(path) = entry["path"].as_str() else {
            return false;
        };
        let Some(raw_sha256) = entry["raw_sha256"].as_str() else {
            return false;
        };
        if !exact_value_fields(entry, &["schema", "path", "raw_sha256"])
            || !matches!(
                schema,
                "mpk.csharp.type_contract.v1"
                    | "mpk.csharp.contract.v1"
                    | "mpk.csharp.semantic_binding.v1"
                    | "mpk.csharp.boundary.v1"
                    | "mpk.csharp.transition.v1"
            )
            || !normalized_relative_path(&entry["path"])
            || !valid_sha256(raw_sha256)
            || !paths.insert(path)
        {
            return false;
        }
        let current = (schema, path, raw_sha256);
        if previous.is_some_and(|value| value >= current) {
            return false;
        }
        previous = Some(current);
    }
    true
}

fn strictly_ordered_paths<'a>(
    entries: &'a [Value],
    path: impl Fn(&'a Value) -> Option<&'a str>,
) -> bool {
    let mut previous = None;
    for entry in entries {
        let Some(current) = path(entry) else {
            return false;
        };
        if previous.is_some_and(|value: &str| value.as_bytes() >= current.as_bytes()) {
            return false;
        }
        previous = Some(current);
    }
    true
}

fn artifact_reference_array_shape(value: &Value, schema: &str) -> bool {
    value.as_array().is_some_and(|entries| {
        entries
            .iter()
            .all(|entry| artifact_reference_shape(entry, schema))
    })
}

fn artifact_reference_shape(value: &Value, schema: &str) -> bool {
    exact_value_fields(value, &["schema", "sha256", "canonical_bytes"])
        && value["schema"] == schema
        && lower_sha256_value(&value["sha256"])
        && value["canonical_bytes"]
            .as_u64()
            .is_some_and(|size| size > 0 && size <= 268_435_456)
}

fn exact_value_fields(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|fields| {
        fields.len() == expected.len() && expected.iter().all(|name| fields.contains_key(*name))
    })
}

fn lower_sha256_value(value: &Value) -> bool {
    value.as_str().is_some_and(valid_sha256)
}

fn normalized_relative_path(value: &Value) -> bool {
    value.as_str().is_some_and(|path| {
        if path.is_empty()
            || path.len() > 1_024
            || !path.is_ascii()
            || path.starts_with('/')
            || path.ends_with('/')
            || path.contains(['\\', ':', '\0'])
            || path.to_ascii_lowercase().starts_with("file:")
        {
            return false;
        }
        path.split('/').all(|component| {
            !component.is_empty()
                && component.len() <= 255
                && !matches!(component, "." | "..")
                && !component.ends_with('.')
                && component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
                && !windows_device_name(component)
        })
    })
}

fn windows_device_name(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

fn valid_canonical_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.is_ascii()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/')
        })
}

fn canonical_artifact_field(
    input: &[u8],
    field: &str,
) -> Result<Vec<u8>, PrivateConsumerMigrationError> {
    let value = parse_canonical_practical_json(PracticalArtifactKind::SourceArtifacts, input)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "canonical field"))?;
    canonical_practical_json_bytes(
        value
            .get(field)
            .ok_or_else(|| failure(PrivateConsumerMigrationCode::Artifact, "missing field"))?,
    )
    .map_err(|_| failure(PrivateConsumerMigrationCode::Artifact, "canonical field"))
}

fn canonical_successor_value(value: &Value) -> Result<Vec<u8>, PrivateConsumerMigrationError> {
    fn convert(value: &Value) -> Result<StrictJsonValue, PrivateConsumerMigrationError> {
        Ok(match value {
            Value::Null => StrictJsonValue::Null,
            Value::Bool(value) => StrictJsonValue::Bool(*value),
            Value::Number(value) => StrictJsonValue::Integer(value.as_i64().ok_or_else(|| {
                failure(PrivateConsumerMigrationCode::Artifact, "successor integer")
            })?),
            Value::String(value) => StrictJsonValue::String(value.clone()),
            Value::Array(values) => {
                StrictJsonValue::Array(values.iter().map(convert).collect::<Result<Vec<_>, _>>()?)
            }
            Value::Object(fields) => StrictJsonValue::Object(
                fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), convert(value)?)))
                    .collect::<Result<Vec<_>, PrivateConsumerMigrationError>>()?,
            ),
        })
    }
    canonical_json_bytes(&convert(value)?).map_err(|_| {
        failure(
            PrivateConsumerMigrationCode::Artifact,
            "successor canonical JSON",
        )
    })
}

fn validate_accepted_receipt(
    input: &[u8],
    graph: &ValidatedGraph,
) -> Result<(), PrivateConsumerMigrationError> {
    parse_strict_json(input, TRANSPORT_LIMITS)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Receipt, "strict JSON"))?;
    let receipt: AcceptedReceipt = serde_json::from_slice(input)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Receipt, "shape"))?;
    if serde_json::to_vec(&receipt)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Receipt, "canonical"))?
        != input
        || receipt.schema != PRIVATE_CONSUMER_RECEIPT_SCHEMA
        || receipt.work_item != PRIVATE_CONSUMER_WORK_ITEM
        || receipt.status != "accepted_private_consumer_closure"
        || receipt.consumer_families
            != PRIVATE_CONSUMER_FAMILIES
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        || receipt.inventory_edges
            != PRIVATE_CONSUMER_INVENTORY_EDGES
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        || graph.semantic_context.get("schema").and_then(Value::as_str)
            != Some(SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA)
        || receipt.guarantees != guarantees()
    {
        return Err(failure(
            PrivateConsumerMigrationCode::Receipt,
            "receipt linkage",
        ));
    }
    Ok(())
}

fn release_member_paths(input: &[u8]) -> Result<Vec<String>, PrivateConsumerMigrationError> {
    let value: Value = serde_json::from_slice(input)
        .map_err(|_| failure(PrivateConsumerMigrationCode::Fixture, "release JSON"))?;
    let mut paths = Vec::new();
    for field in ["frontend_bundles", "toolchain_bundles"] {
        let bundles = value[field]
            .as_array()
            .ok_or_else(|| failure(PrivateConsumerMigrationCode::Fixture, "release bundles"))?;
        for bundle in bundles {
            let bundle_id = bundle["bundle_id"].as_str().ok_or_else(|| {
                failure(PrivateConsumerMigrationCode::Fixture, "release bundle ID")
            })?;
            let members = bundle["inventory"]["members"].as_array().ok_or_else(|| {
                failure(PrivateConsumerMigrationCode::Fixture, "release inventory")
            })?;
            for member in members {
                let member_path = member["path"].as_str().ok_or_else(|| {
                    failure(PrivateConsumerMigrationCode::Fixture, "release member")
                })?;
                paths.push(format!("{field}/{bundle_id}/{member_path}"));
            }
        }
    }
    paths.sort();
    if paths.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(failure(
            PrivateConsumerMigrationCode::Fixture,
            "duplicate release member path",
        ));
    }
    Ok(paths)
}

fn foundation_matches(value: &Value) -> bool {
    value.get("schema").and_then(Value::as_str) == Some(FOUNDATION_DESCRIPTOR_SCHEMA)
        && value.get("id").and_then(Value::as_str) == Some(FOUNDATION_DESCRIPTOR_ID)
        && value.get("content_sha256").and_then(Value::as_str)
            == Some(FOUNDATION_DESCRIPTOR_CONTENT_SHA256)
}

fn reference_matches(value: &Value, artifact: &PrivateSuccessorArtifact) -> bool {
    value.get("schema").and_then(Value::as_str) == Some(artifact.schema())
        && value.get("sha256").and_then(Value::as_str) == Some(artifact.sha256())
        && value.get("canonical_bytes").and_then(Value::as_u64)
            == Some(artifact.canonical_bytes().len() as u64)
        && value.as_object().is_some_and(|object| object.len() == 3)
}

fn receipt_ref(artifact: &PrivateSuccessorArtifact) -> ReceiptArtifactRef {
    bytes_ref(artifact.schema(), artifact.sha256())
}

fn verification_ref(
    artifact: &mpk_vc::csharp_practical_consumer::PrivateVerificationArtifact,
) -> ReceiptArtifactRef {
    bytes_ref(artifact.schema(), artifact.sha256())
}

fn bytes_ref(schema: &str, sha256: &str) -> ReceiptArtifactRef {
    ReceiptArtifactRef {
        schema: schema.to_owned(),
        sha256: sha256.to_owned(),
    }
}

fn encode_receipt<T: Serialize>(value: &T) -> Result<Vec<u8>, PrivateConsumerMigrationError> {
    serde_json::to_vec(value).map_err(|_| failure(PrivateConsumerMigrationCode::Receipt, "encode"))
}

fn guarantees() -> ReceiptGuarantees {
    ReceiptGuarantees {
        public_route_added: false,
        release_candidate_materialized: false,
        dual_format_fallback: false,
        certificate_v0_retained: true,
    }
}

fn producer_name(producer: PredecessorProducer) -> &'static str {
    match producer {
        PredecessorProducer::CSharpScalar => "csharp_scalar",
        PredecessorProducer::Go => "go",
        PredecessorProducer::Java => "java",
        PredecessorProducer::Rust => "rust",
    }
}

fn identity(family: &str, identities: &[&str], hash_domains: &[&str]) -> PrivateIdentityFamily {
    PrivateIdentityFamily::new(
        family,
        identities.iter().copied(),
        hash_domains.iter().copied(),
    )
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn failure(
    code: PrivateConsumerMigrationCode,
    detail: &'static str,
) -> PrivateConsumerMigrationError {
    PrivateConsumerMigrationError { code, detail }
}
