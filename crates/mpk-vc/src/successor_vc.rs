//! Active verification-condition and skeleton integration.
//!
//! This module is reached through successor integration carrying an already
//! validated semantic registry, VIR, and frontend manifest. It does not alter
//! the legacy `mpk.vc.v1` parser. The successor envelope is new; the checked
//! Bool/BV encoding, weakest-
//! precondition engine, grouping rules, theorem declarations, and limits are
//! deliberately shared with VC v1.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, serialize_json_bounded,
    BoundedJsonSerializeError, StrictJsonLimits,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};
use crate::program_encode::ProgramExprContext;
use crate::program_wp::{
    generate_program_vcs_from_profiled_projection, ProfiledProgramVcModule, ProgramVcMemberKind,
};
use crate::safety_check::{
    required_safety_checks, CompiledRequiredCheckProfile, SafetyEvidenceRoute, VirSafetyOperation,
};
use crate::semantic_profile::{
    GoFixedParameters, SemanticParameters, SemanticProfile, SourceLanguage,
};
use crate::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_semantic_context_linkage, CompiledProfileEnvelope,
    CompiledSemanticProfile, ProfileContractField, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use crate::successor_source_artifacts::{
    SuccessorSourceManifestStage, ValidatedSuccessorSourceManifest, ValidatedSuccessorVir,
    SUCCESSOR_SOURCE_MANIFEST_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use crate::type_encode::encode_vir_type;
use crate::vc::{
    VcBinder, VcDocument, VcFunction, VcMember, VcSourceContext, VcSourceFunction, VcTerm,
    VcTypeTerm, VERIFICATION_LIMIT_PROFILE,
};
use crate::vc_canonical::{
    canonical_vc_json, generate_vc_v1_from_context, import_vc_v1_json, vc_hash, ValidatedVcDocument,
};
use crate::vc_skeleton::{emit_validated_vc_skeleton_v1, GroupedTheoremDeclaration};
use crate::verification_limits::{
    VC_CANONICAL_JSON_BYTES_MAX, VC_CANONICAL_SKELETON_JSON_BYTES_MAX,
};
use crate::vir::{
    LowercaseSha256, VirContract, VirFunction, VirInstruction, VirModule, VirSafetyCheck, VirType,
    VirUnit, VIR_SCHEMA_VERSION,
};
use crate::vir_canonical::{contract_hash, vir_hash};
use crate::vir_validate::validate_vir;

pub const SUCCESSOR_VC_SCHEMA: &str = "mpk.vc.v2";
pub const SUCCESSOR_VC_SKELETON_SCHEMA: &str = "mpk.vc.cert_skeleton.v2";
pub const SUCCESSOR_VC_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VC-2.0");

const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SUCCESSOR_VC_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576);

/// Complete, explicitly injected source boundary for successor VC work.
#[derive(Clone, Copy)]
pub struct SuccessorVcSource<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub vir: &'a ValidatedSuccessorVir,
    pub manifest: &'a ValidatedSuccessorSourceManifest,
    pub profile_contract: &'a Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVcDocument {
    schema: String,
    semantic_context: SemanticContext,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    source_manifest_schema: String,
    source_manifest_hash: LowercaseSha256,
    input_set_hash: LowercaseSha256,
    profile_contract: CompiledProfileEnvelope,
    verification_limit_profile: String,
    functions: Vec<VcFunction>,
    vc_hash: LowercaseSha256,
}

impl SuccessorVcDocument {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn source_ir_schema(&self) -> &str {
        &self.source_ir_schema
    }

    pub fn source_ir_hash(&self) -> &LowercaseSha256 {
        &self.source_ir_hash
    }

    pub fn source_manifest_schema(&self) -> &str {
        &self.source_manifest_schema
    }

    pub fn source_manifest_hash(&self) -> &LowercaseSha256 {
        &self.source_manifest_hash
    }

    pub fn input_set_hash(&self) -> &LowercaseSha256 {
        &self.input_set_hash
    }

    pub fn profile_contract(&self) -> &CompiledProfileEnvelope {
        &self.profile_contract
    }

    pub fn verification_limit_profile(&self) -> &str {
        &self.verification_limit_profile
    }

    pub fn functions(&self) -> &[VcFunction] {
        &self.functions
    }

    pub fn vc_hash(&self) -> &LowercaseSha256 {
        &self.vc_hash
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVcDocument {
    schema: String,
    semantic_context: Value,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    source_manifest_schema: String,
    source_manifest_hash: LowercaseSha256,
    input_set_hash: LowercaseSha256,
    profile_contract: Value,
    verification_limit_profile: String,
    functions: Vec<VcFunction>,
    vc_hash: LowercaseSha256,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorRequiredCheckAudit {
    function_id: String,
    member_id: String,
    check: VirSafetyCheck,
    evidence_route: SafetyEvidenceRoute,
}

impl SuccessorRequiredCheckAudit {
    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    pub fn member_id(&self) -> &str {
        &self.member_id
    }

    pub fn check(&self) -> &VirSafetyCheck {
        &self.check
    }

    pub const fn evidence_route(&self) -> SafetyEvidenceRoute {
        self.evidence_route
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorVc {
    document: SuccessorVcDocument,
    canonical_bytes: Vec<u8>,
    required_checks: Vec<SuccessorRequiredCheckAudit>,
}

impl ValidatedSuccessorVc {
    pub fn document(&self) -> &SuccessorVcDocument {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.document.vc_hash
    }

    pub fn required_checks(&self) -> &[SuccessorRequiredCheckAudit] {
        &self.required_checks
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVcCertificateSkeleton {
    schema: String,
    semantic_context: SemanticContext,
    source_vc_schema: String,
    source_vc_hash: LowercaseSha256,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    source_manifest_schema: String,
    source_manifest_hash: LowercaseSha256,
    input_set_hash: LowercaseSha256,
    profile_contract: CompiledProfileEnvelope,
    verification_limit_profile: String,
    theorem_declarations: Vec<GroupedTheoremDeclaration>,
}

impl SuccessorVcCertificateSkeleton {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn source_vc_schema(&self) -> &str {
        &self.source_vc_schema
    }

    pub fn source_vc_hash(&self) -> &LowercaseSha256 {
        &self.source_vc_hash
    }

    pub fn source_ir_schema(&self) -> &str {
        &self.source_ir_schema
    }

    pub fn source_ir_hash(&self) -> &LowercaseSha256 {
        &self.source_ir_hash
    }

    pub fn source_manifest_schema(&self) -> &str {
        &self.source_manifest_schema
    }

    pub fn source_manifest_hash(&self) -> &LowercaseSha256 {
        &self.source_manifest_hash
    }

    pub fn input_set_hash(&self) -> &LowercaseSha256 {
        &self.input_set_hash
    }

    pub fn profile_contract(&self) -> &CompiledProfileEnvelope {
        &self.profile_contract
    }

    pub fn verification_limit_profile(&self) -> &str {
        &self.verification_limit_profile
    }

    pub fn theorem_declarations(&self) -> &[GroupedTheoremDeclaration] {
        &self.theorem_declarations
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVcCertificateSkeleton {
    schema: String,
    semantic_context: Value,
    source_vc_schema: String,
    source_vc_hash: LowercaseSha256,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    source_manifest_schema: String,
    source_manifest_hash: LowercaseSha256,
    input_set_hash: LowercaseSha256,
    profile_contract: Value,
    verification_limit_profile: String,
    theorem_declarations: Vec<GroupedTheoremDeclaration>,
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorVcSkeleton {
    skeleton: SuccessorVcCertificateSkeleton,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorVcSkeleton {
    pub fn skeleton(&self) -> &SuccessorVcCertificateSkeleton {
        &self.skeleton
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorVcValidationPhase {
    Transport,
    Shape,
    SemanticContext,
    ProfileContract,
    SourceLinkage,
    RequiredChecks,
    Members,
    Declarations,
    CanonicalSize,
    CanonicalTransport,
    Hash,
}

impl SuccessorVcValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::SemanticContext => "semantic_context",
            Self::ProfileContract => "profile_contract",
            Self::SourceLinkage => "source_linkage",
            Self::RequiredChecks => "required_checks",
            Self::Members => "members",
            Self::Declarations => "declarations",
            Self::CanonicalSize => "canonical_size",
            Self::CanonicalTransport => "canonical_transport",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorVcErrorCode {
    Json,
    Shape,
    SemanticContext,
    ProfileContract,
    SourceLinkage,
    RequiredChecks,
    Members,
    Declarations,
    Limit,
    CanonicalTransport,
    Hash,
}

impl SuccessorVcErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "SUCCESSOR_VC_JSON",
            Self::Shape => "SUCCESSOR_VC_SHAPE",
            Self::SemanticContext => "SUCCESSOR_VC_SEMANTIC_CONTEXT",
            Self::ProfileContract => "SUCCESSOR_VC_PROFILE_CONTRACT",
            Self::SourceLinkage => "SUCCESSOR_VC_SOURCE_LINKAGE",
            Self::RequiredChecks => "SUCCESSOR_VC_REQUIRED_CHECKS",
            Self::Members => "SUCCESSOR_VC_MEMBERS",
            Self::Declarations => "SUCCESSOR_VC_DECLARATIONS",
            Self::Limit => "SUCCESSOR_VC_LIMIT",
            Self::CanonicalTransport => "SUCCESSOR_VC_CANONICAL_TRANSPORT",
            Self::Hash => "SUCCESSOR_VC_HASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorVcError {
    phase: SuccessorVcValidationPhase,
    code: SuccessorVcErrorCode,
    detail: String,
}

impl SuccessorVcError {
    pub const fn phase(&self) -> SuccessorVcValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorVcErrorCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for SuccessorVcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} during {}: {}",
            self.code.as_str(),
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for SuccessorVcError {}

struct PreparedSource {
    profile_contract: CompiledProfileEnvelope,
    projection: VirModule,
    source_context: VcSourceContext,
    functions: Vec<VcFunction>,
    required_checks: Vec<SuccessorRequiredCheckAudit>,
}

/// Generates canonical `mpk.vc.v2` bytes from the complete validated staging
/// source. No bytes are returned unless regeneration and successor import both
/// succeed.
pub fn generate_successor_vc(
    source: SuccessorVcSource<'_>,
) -> Result<ValidatedSuccessorVc, SuccessorVcError> {
    let prepared = prepare_source(source)?;
    let document = expected_document(source, &prepared)?;
    let canonical = canonical_successor_vc_json(&document)?;
    import_successor_vc_json(&canonical, source)
}

/// Imports canonical successor VC bytes and regenerates every source-derived
/// function, safety member, group, and dependency before accepting them.
pub fn import_successor_vc_json(
    input: &[u8],
    source: SuccessorVcSource<'_>,
) -> Result<ValidatedSuccessorVc, SuccessorVcError> {
    let (canonical, wire) = parse_wire::<WireSuccessorVcDocument>(
        input,
        VC_CANONICAL_JSON_BYTES_MAX,
        SuccessorVcValidationPhase::Transport,
    )?;
    if wire.schema != SUCCESSOR_VC_SCHEMA {
        return Err(failure(
            SuccessorVcValidationPhase::Shape,
            SuccessorVcErrorCode::Shape,
            "successor VC schema differs",
        ));
    }
    let semantic_context = crate::semantic_profile_registry::validate_registry_semantic_context(
        source.registry,
        &wire.semantic_context,
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::SemanticContext,
            SuccessorVcErrorCode::SemanticContext,
            error.to_string(),
        )
    })?;
    let profile_contract = validate_compiled_profile_envelope(
        source.registry,
        &wire.profile_contract,
        ProfileContractField::Vc,
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::ProfileContract,
            SuccessorVcErrorCode::ProfileContract,
            error.to_string(),
        )
    })?;
    let document = SuccessorVcDocument {
        schema: wire.schema,
        semantic_context,
        source_ir_schema: wire.source_ir_schema,
        source_ir_hash: wire.source_ir_hash,
        source_manifest_schema: wire.source_manifest_schema,
        source_manifest_hash: wire.source_manifest_hash,
        input_set_hash: wire.input_set_hash,
        profile_contract,
        verification_limit_profile: wire.verification_limit_profile,
        functions: wire.functions,
        vc_hash: wire.vc_hash,
    };

    let prepared = prepare_source(source)?;
    validate_source_fields(&document, source, &prepared)?;
    let expected = expected_document(source, &prepared)?;
    if document.functions != expected.functions {
        return Err(failure(
            SuccessorVcValidationPhase::Members,
            SuccessorVcErrorCode::Members,
            "VC functions, members, groups, or dependencies differ from regeneration",
        ));
    }
    if input != canonical {
        return Err(failure(
            SuccessorVcValidationPhase::CanonicalTransport,
            SuccessorVcErrorCode::CanonicalTransport,
            "successor VC transport is not byte-identical JCS",
        ));
    }
    let recomputed = successor_vc_hash(&document)?;
    if recomputed != document.vc_hash {
        return Err(failure(
            SuccessorVcValidationPhase::Hash,
            SuccessorVcErrorCode::Hash,
            "vc_hash does not match the MPK-VC-2.0 preimage",
        ));
    }
    Ok(ValidatedSuccessorVc {
        document,
        canonical_bytes: canonical,
        required_checks: prepared.required_checks,
    })
}

pub fn successor_vc_hash(
    document: &SuccessorVcDocument,
) -> Result<LowercaseSha256, SuccessorVcError> {
    let mut value = serde_json::to_value(document).map_err(|error| {
        failure(
            SuccessorVcValidationPhase::Hash,
            SuccessorVcErrorCode::Hash,
            error.to_string(),
        )
    })?;
    value
        .as_object_mut()
        .and_then(|object| object.remove("vc_hash"))
        .ok_or_else(|| {
            failure(
                SuccessorVcValidationPhase::Shape,
                SuccessorVcErrorCode::Shape,
                "successor VC hash payload has no vc_hash",
            )
        })?;
    let payload = canonical_value(&value, VC_CANONICAL_JSON_BYTES_MAX)?;
    let digest =
        hash_domain_separated_raw(SUCCESSOR_VC_HASH_DOMAIN, &payload).map_err(|error| {
            failure(
                SuccessorVcValidationPhase::Hash,
                SuccessorVcErrorCode::Hash,
                error.to_string(),
            )
        })?;
    LowercaseSha256::new(digest.to_hex()).map_err(|error| {
        failure(
            SuccessorVcValidationPhase::Hash,
            SuccessorVcErrorCode::Hash,
            error.to_string(),
        )
    })
}

/// Emits the exact successor grouped declaration skeleton from a validated
/// successor VC. The declarations are reconstructed through the checked v1
/// grouping implementation, never copied from caller-provided skeleton data.
pub fn emit_successor_vc_skeleton(
    vc: &ValidatedSuccessorVc,
    source: SuccessorVcSource<'_>,
) -> Result<ValidatedSuccessorVcSkeleton, SuccessorVcError> {
    let prepared = prepare_source(source)?;
    validate_source_fields(vc.document(), source, &prepared)?;
    let expected = expected_document(source, &prepared)?;
    if vc.document() != &expected {
        return Err(failure(
            SuccessorVcValidationPhase::Members,
            SuccessorVcErrorCode::Members,
            "validated successor VC differs from source regeneration",
        ));
    }
    let skeleton = expected_skeleton(vc.document(), &prepared)?;
    let canonical = canonical_successor_skeleton_json(&skeleton)?;
    import_successor_vc_skeleton_json(&canonical, vc.canonical_bytes(), source)
}

/// Imports a successor skeleton against complete canonical successor VC bytes
/// and reconstructs every declaration from that VC.
pub fn import_successor_vc_skeleton_json(
    input: &[u8],
    source_vc_bytes: &[u8],
    source: SuccessorVcSource<'_>,
) -> Result<ValidatedSuccessorVcSkeleton, SuccessorVcError> {
    let vc = import_successor_vc_json(source_vc_bytes, source)?;
    let (canonical, wire) = parse_wire::<WireSuccessorVcCertificateSkeleton>(
        input,
        VC_CANONICAL_SKELETON_JSON_BYTES_MAX,
        SuccessorVcValidationPhase::Transport,
    )?;
    if wire.schema != SUCCESSOR_VC_SKELETON_SCHEMA {
        return Err(failure(
            SuccessorVcValidationPhase::Shape,
            SuccessorVcErrorCode::Shape,
            "successor skeleton schema differs",
        ));
    }
    let semantic_context = crate::semantic_profile_registry::validate_registry_semantic_context(
        source.registry,
        &wire.semantic_context,
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::SemanticContext,
            SuccessorVcErrorCode::SemanticContext,
            error.to_string(),
        )
    })?;
    let profile_contract = validate_compiled_profile_envelope(
        source.registry,
        &wire.profile_contract,
        ProfileContractField::Vc,
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::ProfileContract,
            SuccessorVcErrorCode::ProfileContract,
            error.to_string(),
        )
    })?;
    let skeleton = SuccessorVcCertificateSkeleton {
        schema: wire.schema,
        semantic_context,
        source_vc_schema: wire.source_vc_schema,
        source_vc_hash: wire.source_vc_hash,
        source_ir_schema: wire.source_ir_schema,
        source_ir_hash: wire.source_ir_hash,
        source_manifest_schema: wire.source_manifest_schema,
        source_manifest_hash: wire.source_manifest_hash,
        input_set_hash: wire.input_set_hash,
        profile_contract,
        verification_limit_profile: wire.verification_limit_profile,
        theorem_declarations: wire.theorem_declarations,
    };
    let prepared = prepare_source(source)?;
    let expected = expected_skeleton(vc.document(), &prepared)?;
    if skeleton != expected {
        return Err(failure(
            SuccessorVcValidationPhase::Declarations,
            SuccessorVcErrorCode::Declarations,
            "skeleton linkage or theorem declarations differ from reconstruction",
        ));
    }
    if input != canonical {
        return Err(failure(
            SuccessorVcValidationPhase::CanonicalTransport,
            SuccessorVcErrorCode::CanonicalTransport,
            "successor skeleton transport is not byte-identical JCS",
        ));
    }
    Ok(ValidatedSuccessorVcSkeleton {
        skeleton,
        canonical_bytes: canonical,
    })
}

fn expected_document(
    source: SuccessorVcSource<'_>,
    prepared: &PreparedSource,
) -> Result<SuccessorVcDocument, SuccessorVcError> {
    let manifest = source.manifest.manifest();
    let mut document = SuccessorVcDocument {
        schema: SUCCESSOR_VC_SCHEMA.to_owned(),
        semantic_context: source.vir.module().semantic_context().clone(),
        source_ir_schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
        source_ir_hash: source.vir.hash().clone(),
        source_manifest_schema: SUCCESSOR_SOURCE_MANIFEST_SCHEMA.to_owned(),
        source_manifest_hash: source.manifest.hash().clone(),
        input_set_hash: manifest.input_set_hash().clone(),
        profile_contract: prepared.profile_contract.clone(),
        verification_limit_profile: VERIFICATION_LIMIT_PROFILE.to_owned(),
        functions: prepared.functions.clone(),
        vc_hash: LowercaseSha256::new(ZERO_SHA256.to_owned()).expect("zero digest"),
    };
    document.vc_hash = successor_vc_hash(&document)?;
    canonical_successor_vc_json(&document)?;
    Ok(document)
}

fn expected_skeleton(
    vc: &SuccessorVcDocument,
    prepared: &PreparedSource,
) -> Result<SuccessorVcCertificateSkeleton, SuccessorVcError> {
    let active = active_proxy(vc, prepared)?;
    let active_skeleton = emit_validated_vc_skeleton_v1(&active).map_err(|error| {
        failure(
            SuccessorVcValidationPhase::Declarations,
            SuccessorVcErrorCode::Declarations,
            error.to_string(),
        )
    })?;
    Ok(SuccessorVcCertificateSkeleton {
        schema: SUCCESSOR_VC_SKELETON_SCHEMA.to_owned(),
        semantic_context: vc.semantic_context.clone(),
        source_vc_schema: SUCCESSOR_VC_SCHEMA.to_owned(),
        source_vc_hash: vc.vc_hash.clone(),
        source_ir_schema: vc.source_ir_schema.clone(),
        source_ir_hash: vc.source_ir_hash.clone(),
        source_manifest_schema: vc.source_manifest_schema.clone(),
        source_manifest_hash: vc.source_manifest_hash.clone(),
        input_set_hash: vc.input_set_hash.clone(),
        profile_contract: vc.profile_contract.clone(),
        verification_limit_profile: vc.verification_limit_profile.clone(),
        theorem_declarations: active_skeleton.skeleton().theorem_declarations.clone(),
    })
}

fn active_proxy(
    successor: &SuccessorVcDocument,
    prepared: &PreparedSource,
) -> Result<ValidatedVcDocument, SuccessorVcError> {
    let mut document = VcDocument {
        schema: crate::VC_SCHEMA_VERSION.to_owned(),
        source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
        source_ir_hash: prepared.projection.vir_hash.as_str().to_owned(),
        input_set_hash: successor.input_set_hash.as_str().to_owned(),
        semantic_profile: prepared.projection.semantic_profile,
        semantic_parameters: prepared.projection.semantic_parameters.clone(),
        verification_limit_profile: successor.verification_limit_profile.clone(),
        functions: successor.functions.clone(),
        vc_hash: ZERO_SHA256.to_owned(),
    };
    document.vc_hash = vc_hash(&document)
        .map_err(map_active_vc_error)?
        .as_str()
        .to_owned();
    let bytes = canonical_vc_json(&document).map_err(map_active_vc_error)?;
    import_vc_v1_json(&bytes, &prepared.source_context).map_err(map_active_vc_error)
}

fn validate_source_fields(
    document: &SuccessorVcDocument,
    source: SuccessorVcSource<'_>,
    prepared: &PreparedSource,
) -> Result<(), SuccessorVcError> {
    let vir_context = source.vir.module().semantic_context();
    let manifest = source.manifest.manifest();
    validate_semantic_context_linkage(&document.semantic_context, vir_context)
        .and_then(|_| {
            validate_semantic_context_linkage(
                &document.semantic_context,
                manifest.semantic_context(),
            )
        })
        .map_err(|error| {
            failure(
                SuccessorVcValidationPhase::SourceLinkage,
                SuccessorVcErrorCode::SourceLinkage,
                error.to_string(),
            )
        })?;
    if document.source_ir_schema != SUCCESSOR_VIR_SCHEMA
        || document.source_ir_hash != *source.vir.hash()
        || document.source_manifest_schema != SUCCESSOR_SOURCE_MANIFEST_SCHEMA
        || document.source_manifest_hash != *source.manifest.hash()
        || document.input_set_hash != *manifest.input_set_hash()
        || document.verification_limit_profile != VERIFICATION_LIMIT_PROFILE
        || document.profile_contract != prepared.profile_contract
        || document.profile_contract.profile_entry_sha256()
            != document.semantic_context.profile_entry_sha256()
    {
        return Err(failure(
            SuccessorVcValidationPhase::SourceLinkage,
            SuccessorVcErrorCode::SourceLinkage,
            "successor VC repeated source identity differs",
        ));
    }
    Ok(())
}

fn prepare_source(source: SuccessorVcSource<'_>) -> Result<PreparedSource, SuccessorVcError> {
    if source.manifest.stage() != SuccessorSourceManifestStage::Frontend {
        return Err(failure(
            SuccessorVcValidationPhase::SourceLinkage,
            SuccessorVcErrorCode::SourceLinkage,
            "successor VC generation requires a frontend-stage manifest",
        ));
    }
    validate_semantic_context_linkage(
        source.vir.module().semantic_context(),
        source.manifest.manifest().semantic_context(),
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::SourceLinkage,
            SuccessorVcErrorCode::SourceLinkage,
            error.to_string(),
        )
    })?;
    let context = source.vir.module().semantic_context();
    let profile = CompiledSemanticProfile::from_identity(
        context.source_language(),
        context.semantic_profile(),
    )
    .ok_or_else(|| {
        failure(
            SuccessorVcValidationPhase::SemanticContext,
            SuccessorVcErrorCode::SemanticContext,
            "semantic context does not select a compiled profile",
        )
    })?;
    let entry = source
        .registry
        .lookup(context.source_language(), context.semantic_profile())
        .ok_or_else(|| {
            failure(
                SuccessorVcValidationPhase::SemanticContext,
                SuccessorVcErrorCode::SemanticContext,
                "semantic profile entry is absent",
            )
        })?;
    if entry.compiled_profile() != profile || entry.entry_sha256() != context.profile_entry_sha256()
    {
        return Err(failure(
            SuccessorVcValidationPhase::SemanticContext,
            SuccessorVcErrorCode::SemanticContext,
            "semantic profile entry linkage differs",
        ));
    }
    let profile_contract = validate_compiled_profile_envelope(
        source.registry,
        source.profile_contract,
        ProfileContractField::Vc,
    )
    .map_err(|error| {
        failure(
            SuccessorVcValidationPhase::ProfileContract,
            SuccessorVcErrorCode::ProfileContract,
            error.to_string(),
        )
    })?;
    if profile_contract.profile_entry_sha256() != context.profile_entry_sha256() {
        return Err(failure(
            SuccessorVcValidationPhase::ProfileContract,
            SuccessorVcErrorCode::ProfileContract,
            "VC profile contract belongs to another semantic entry",
        ));
    }

    let projection = project_successor_vir(source.vir, profile)?;
    validate_projection(&projection, profile)?;
    let check_profile = compiled_check_profile(profile_contract.value(), profile)?;
    let generated = generate_program_vcs_from_profiled_projection(&projection, check_profile)
        .map_err(|error| {
            failure(
                SuccessorVcValidationPhase::RequiredChecks,
                SuccessorVcErrorCode::RequiredChecks,
                error.to_string(),
            )
        })?;
    let (source_context, functions, required_checks) =
        build_source_projection(source, &projection, &generated)?;
    Ok(PreparedSource {
        profile_contract,
        projection,
        source_context,
        functions,
        required_checks,
    })
}

fn project_successor_vir(
    source: &ValidatedSuccessorVir,
    profile: CompiledSemanticProfile,
) -> Result<VirModule, SuccessorVcError> {
    let (semantic_profile, semantic_parameters, source_language) =
        projection_semantics(source, profile)?;
    let mut units = Vec::with_capacity(source.module().units().len());
    let mut active_contract_hashes = BTreeMap::new();
    for unit in source.module().units() {
        let mut functions = Vec::with_capacity(unit.functions().len());
        for function in unit.functions() {
            let mut contract = VirContract {
                unit_id: function.contracts().unit_id().to_owned(),
                function_id: function.contracts().function_id().to_owned(),
                semantic_profile,
                semantic_parameters: semantic_parameters.clone(),
                requires: function.contracts().requires().to_vec(),
                ensures: function.contracts().ensures().to_vec(),
                modifies: function.contracts().modifies().to_vec(),
                panic: function.contracts().panic(),
                termination: function.contracts().termination(),
                loops: function.contracts().loops().to_vec(),
                contract_hash: LowercaseSha256::new(ZERO_SHA256.to_owned()).expect("zero digest"),
            };
            contract.contract_hash =
                contract_hash(&contract).map_err(|error| source_failure(error.to_string()))?;
            active_contract_hashes.insert(function.id().to_owned(), contract.contract_hash.clone());
            functions.push(VirFunction {
                id: function.id().to_owned(),
                unit_id: function.unit_id().to_owned(),
                name: function.name().to_owned(),
                params: function.params().to_vec(),
                results: function.results().to_vec(),
                locals: function.locals().to_vec(),
                blocks: function.blocks().to_vec(),
                contracts: contract,
                features_used: function.features_used().to_vec(),
            });
        }
        units.push(VirUnit {
            id: unit.id().to_owned(),
            name: unit.name().to_owned(),
            type_decls: unit.type_decls().to_vec(),
            const_decls: unit.const_decls().to_vec(),
            functions,
        });
    }
    for function in units.iter_mut().flat_map(|unit| &mut unit.functions) {
        for instruction in function
            .blocks
            .iter_mut()
            .flat_map(|block| &mut block.instructions)
        {
            if let VirInstruction::CallStatic {
                function,
                contract_hash,
                ..
            } = instruction
            {
                *contract_hash = active_contract_hashes
                    .get(function)
                    .cloned()
                    .ok_or_else(|| source_failure("static callee is absent"))?;
            }
        }
    }
    let mut module = VirModule {
        schema: VIR_SCHEMA_VERSION.to_owned(),
        source_language,
        semantic_profile,
        semantic_parameters,
        units,
        vir_hash: LowercaseSha256::new(ZERO_SHA256.to_owned()).expect("zero digest"),
    };
    module.vir_hash = vir_hash(&module).map_err(|error| source_failure(error.to_string()))?;
    Ok(module)
}

fn projection_semantics(
    source: &ValidatedSuccessorVir,
    profile: CompiledSemanticProfile,
) -> Result<(SemanticProfile, SemanticParameters, SourceLanguage), SuccessorVcError> {
    let parameters = source
        .module()
        .semantic_context()
        .semantic_parameters()
        .value()
        .clone();
    match profile {
        CompiledSemanticProfile::GoFixedV0 => {
            let parameters = serde_json::from_value::<GoFixedParameters>(parameters)
                .map_err(|error| source_failure(error.to_string()))?;
            Ok((
                SemanticProfile::GoFixedV0,
                SemanticParameters::GoFixed(parameters),
                SourceLanguage::Go,
            ))
        }
        CompiledSemanticProfile::RustCheckedV0 => {
            let parameters = serde_json::from_value(parameters)
                .map_err(|error| source_failure(error.to_string()))?;
            Ok((
                SemanticProfile::RustCheckedV0,
                parameters,
                SourceLanguage::Rust,
            ))
        }
        CompiledSemanticProfile::CSharpScalarV0 | CompiledSemanticProfile::JavaScalarV0 => Ok((
            SemanticProfile::GoFixedV0,
            SemanticParameters::GoFixed(GoFixedParameters {
                target_id: "linux/amd64".to_owned(),
                pointer_width: crate::PointerWidth::Bits64,
            }),
            SourceLanguage::Go,
        )),
    }
}

fn validate_projection(
    projection: &VirModule,
    profile: CompiledSemanticProfile,
) -> Result<(), SuccessorVcError> {
    if !matches!(
        profile,
        CompiledSemanticProfile::CSharpScalarV0 | CompiledSemanticProfile::JavaScalarV0
    ) {
        return validate_vir(projection).map_err(|error| source_failure(error.to_string()));
    }

    // C# and Java have their own method IDs and required-check rules. For the
    // common structural/type validator only, use private Go-shaped IDs and
    // check lists. The original Java lists are checked at VIR import; the WP
    // pass independently regenerates both languages' original check lists.
    let mut structural = scalar_structural_projection(projection, profile.source_language())?;
    let context_projection = structural.clone();
    for (unit_index, unit) in structural.units.iter_mut().enumerate() {
        for (function_index, function) in unit.functions.iter_mut().enumerate() {
            let context = ProgramExprContext::for_validated_function(
                &context_projection,
                &context_projection.units[unit_index],
                &context_projection.units[unit_index].functions[function_index],
            )
            .map_err(|error| source_failure(error.to_string()))?;
            for instruction in function
                .blocks
                .iter_mut()
                .flat_map(|block| &mut block.instructions)
            {
                let (operation, operands) = safety_operation(&context, instruction)?;
                *instruction_checks_mut(instruction) =
                    required_safety_checks(SemanticProfile::GoFixedV0, operation, &operands)
                        .map_err(|error| source_failure(error.to_string()))?;
            }
        }
    }
    structural.vir_hash =
        vir_hash(&structural).map_err(|error| source_failure(error.to_string()))?;
    validate_vir(&structural).map_err(|error| source_failure(error.to_string()))
}

// Used by Java's VIR importer before returning a validated artifact, without
// generating VCs, certificates or any public compatibility representation.
pub(crate) fn validate_java_vir_structure(
    source: &ValidatedSuccessorVir,
) -> Result<(), SuccessorVcError> {
    let projection = project_successor_vir(source, CompiledSemanticProfile::JavaScalarV0)?;
    validate_projection(&projection, CompiledSemanticProfile::JavaScalarV0)
}

fn scalar_structural_projection(
    projection: &VirModule,
    language: &str,
) -> Result<VirModule, SuccessorVcError> {
    if projection.units.len() != 1
        || !projection.units[0].type_decls.is_empty()
        || !projection.units[0].const_decls.is_empty()
    {
        return Err(source_failure(
            "scalar projection requires one unit and no type or constant declarations",
        ));
    }

    let mut structural = projection.clone();
    let mut function_ids = BTreeMap::new();
    for (index, function) in structural.units[0].functions.iter().enumerate() {
        function_ids.insert(function.id.clone(), format!("{language}.f{index:04}"));
    }

    structural.units[0].id = language.to_owned();
    structural.units[0].name = language.to_owned();
    let mut contract_hashes = BTreeMap::new();
    for (index, function) in structural.units[0].functions.iter_mut().enumerate() {
        let id = function_ids
            .get(&function.id)
            .ok_or_else(|| source_failure("scalar validation surrogate is absent"))?
            .clone();
        let name = format!("f{index:04}");
        function.id = id.clone();
        function.unit_id = language.to_owned();
        function.name = name;
        function.contracts.unit_id = language.to_owned();
        function.contracts.function_id = id.clone();
        function.contracts.contract_hash = contract_hash(&function.contracts)
            .map_err(|error| source_failure(error.to_string()))?;
        contract_hashes.insert(id, function.contracts.contract_hash.clone());
    }
    for function in &mut structural.units[0].functions {
        for instruction in function
            .blocks
            .iter_mut()
            .flat_map(|block| &mut block.instructions)
        {
            if let VirInstruction::CallStatic {
                function,
                contract_hash,
                ..
            } = instruction
            {
                *function = function_ids
                    .get(function)
                    .ok_or_else(|| source_failure("scalar static callee surrogate is absent"))?
                    .clone();
                *contract_hash = contract_hashes
                    .get(function)
                    .ok_or_else(|| source_failure("scalar static callee contract is absent"))?
                    .clone();
            }
        }
    }
    structural.vir_hash =
        vir_hash(&structural).map_err(|error| source_failure(error.to_string()))?;
    Ok(structural)
}

fn compiled_check_profile(
    payload: &Value,
    profile: CompiledSemanticProfile,
) -> Result<CompiledRequiredCheckProfile, SuccessorVcError> {
    let id = payload
        .get("required_check_profile_id")
        .and_then(Value::as_str)
        .ok_or_else(|| profile_failure("required-check profile ID is absent"))?;
    match (profile, id) {
        (CompiledSemanticProfile::GoFixedV0, "mpk.go.fixed.v0") => {
            Ok(CompiledRequiredCheckProfile::GoFixedV0)
        }
        (CompiledSemanticProfile::RustCheckedV0, "mpk.rust.checked.v0") => {
            Ok(CompiledRequiredCheckProfile::RustCheckedV0)
        }
        (CompiledSemanticProfile::CSharpScalarV0, "mpk.csharp.required_checks.v0") => {
            Ok(CompiledRequiredCheckProfile::CSharpScalarV0)
        }
        (CompiledSemanticProfile::JavaScalarV0, "mpk.java.required_checks.v0") => {
            Ok(CompiledRequiredCheckProfile::JavaScalarV0)
        }
        _ => Err(profile_failure(
            "required-check profile does not own the entry",
        )),
    }
}

fn build_source_projection(
    source: SuccessorVcSource<'_>,
    projection: &VirModule,
    generated: &ProfiledProgramVcModule,
) -> Result<
    (
        VcSourceContext,
        Vec<VcFunction>,
        Vec<SuccessorRequiredCheckAudit>,
    ),
    SuccessorVcError,
> {
    let mut functions_by_id = BTreeMap::new();
    for unit in &projection.units {
        for function in &unit.functions {
            functions_by_id.insert(function.id.as_str(), (unit, function));
        }
    }
    let mut successor_functions = BTreeMap::new();
    for unit in source.vir.module().units() {
        for function in unit.functions() {
            successor_functions.insert(function.id(), function);
        }
    }

    let mut source_functions = Vec::with_capacity(generated.module.functions.len());
    let mut required_checks = Vec::new();
    let mut consumed_safety_origins = 0_usize;
    for generated_function in &generated.module.functions {
        let (unit, function) = functions_by_id
            .get(generated_function.function_id.as_str())
            .ok_or_else(|| source_failure("generated function is absent from projection"))?;
        let successor_function = successor_functions
            .get(generated_function.function_id.as_str())
            .ok_or_else(|| source_failure("generated function is absent from successor VIR"))?;
        let parameters = function
            .params
            .iter()
            .map(|parameter| {
                encode_vir_type(
                    projection.semantic_profile,
                    &projection.semantic_parameters,
                    &unit.type_decls,
                    &parameter.r#type,
                )
                .map(|encoded| VcBinder {
                    id: parameter.id.clone(),
                    r#type: VcTypeTerm::from(&encoded),
                })
                .map_err(|error| source_failure(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let requires = generated_function
            .requires
            .iter()
            .map(VcTerm::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| source_failure(error.to_string()))?;
        let regenerated_members = generated_function
            .members
            .iter()
            .map(VcMember::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| source_failure(error.to_string()))?;
        let safety_members = generated_function
            .members
            .iter()
            .filter(|member| member.kind == ProgramVcMemberKind::OperationSafety)
            .collect::<Vec<_>>();
        let safety_member_count = safety_members.len();
        let mut origins_by_member = BTreeMap::new();
        for origin in generated
            .safety_origins
            .iter()
            .filter(|origin| origin.function_id == function.id)
        {
            if origins_by_member
                .insert(origin.member_id.as_str(), origin)
                .is_some()
            {
                return Err(failure(
                    SuccessorVcValidationPhase::RequiredChecks,
                    SuccessorVcErrorCode::RequiredChecks,
                    "one generated safety member has multiple source origins",
                ));
            }
        }
        if origins_by_member.len() != safety_members.len() {
            return Err(failure(
                SuccessorVcValidationPhase::RequiredChecks,
                SuccessorVcErrorCode::RequiredChecks,
                "generated safety members and source origins differ in count",
            ));
        }
        let mut expected_origins = BTreeSet::new();
        for (block_index, block) in function.blocks.iter().enumerate() {
            for (instruction_index, instruction) in block.instructions.iter().enumerate() {
                for check_index in 0..instruction_checks(instruction).len() {
                    expected_origins.insert((block_index, instruction_index, check_index));
                }
            }
        }
        let mut observed_origins = BTreeSet::new();
        for member in safety_members {
            let origin = origins_by_member
                .remove(member.id.as_str())
                .ok_or_else(|| {
                    failure(
                        SuccessorVcValidationPhase::RequiredChecks,
                        SuccessorVcErrorCode::RequiredChecks,
                        "operation-safety member has no source origin",
                    )
                })?;
            let instruction = function
                .blocks
                .get(origin.block_index)
                .and_then(|block| block.instructions.get(origin.instruction_index))
                .ok_or_else(|| {
                    failure(
                        SuccessorVcValidationPhase::RequiredChecks,
                        SuccessorVcErrorCode::RequiredChecks,
                        "operation-safety source origin is outside the function",
                    )
                })?;
            let check = instruction_checks(instruction)
                .get(origin.check_index)
                .cloned()
                .ok_or_else(|| {
                    failure(
                        SuccessorVcValidationPhase::RequiredChecks,
                        SuccessorVcErrorCode::RequiredChecks,
                        "operation-safety source origin has no required check",
                    )
                })?;
            observed_origins.insert((
                origin.block_index,
                origin.instruction_index,
                origin.check_index,
            ));
            required_checks.push(SuccessorRequiredCheckAudit {
                function_id: function.id.clone(),
                member_id: member.id.clone(),
                check,
                evidence_route: member.safety_evidence.ok_or_else(|| {
                    failure(
                        SuccessorVcValidationPhase::RequiredChecks,
                        SuccessorVcErrorCode::RequiredChecks,
                        "operation-safety member has no checked evidence route",
                    )
                })?,
            });
        }
        if !origins_by_member.is_empty() || observed_origins != expected_origins {
            return Err(failure(
                SuccessorVcValidationPhase::RequiredChecks,
                SuccessorVcErrorCode::RequiredChecks,
                "required-check source origins do not exactly cover the generated members",
            ));
        }
        consumed_safety_origins = consumed_safety_origins
            .checked_add(safety_member_count)
            .ok_or_else(|| limit_failure("required-check origin count overflows"))?;
        source_functions.push(VcSourceFunction {
            function_id: function.id.clone(),
            contract_hash: successor_function
                .contracts()
                .contract_hash()
                .as_str()
                .to_owned(),
            direct_callees: generated_function.direct_callees.clone(),
            parameters,
            requires,
            regenerated_members,
        });
    }
    if consumed_safety_origins != generated.safety_origins.len() {
        return Err(failure(
            SuccessorVcValidationPhase::RequiredChecks,
            SuccessorVcErrorCode::RequiredChecks,
            "required-check origin accounting differs from generation",
        ));
    }
    let context = VcSourceContext {
        id: "successor.validated_vir_manifest".to_owned(),
        source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
        source_ir_hash: projection.vir_hash.as_str().to_owned(),
        input_set_hash: source
            .manifest
            .manifest()
            .input_set_hash()
            .as_str()
            .to_owned(),
        semantic_profile: projection.semantic_profile,
        semantic_parameters: projection.semantic_parameters.clone(),
        verification_limit_profile: VERIFICATION_LIMIT_PROFILE.to_owned(),
        functions: source_functions,
    };
    let active = generate_vc_v1_from_context(&context).map_err(map_active_vc_error)?;
    Ok((
        context,
        active.document().functions.clone(),
        required_checks,
    ))
}

fn safety_operation(
    context: &ProgramExprContext,
    instruction: &VirInstruction,
) -> Result<(VirSafetyOperation, Vec<VirType>), SuccessorVcError> {
    match instruction {
        VirInstruction::BinOp { op, lhs, rhs, .. } => Ok((
            VirSafetyOperation::Binary(*op),
            vec![
                context
                    .value_type(lhs)
                    .map_err(|error| source_failure(error.to_string()))?,
                context
                    .value_type(rhs)
                    .map_err(|error| source_failure(error.to_string()))?,
            ],
        )),
        VirInstruction::UnaryOp { op, value, .. } => Ok((
            VirSafetyOperation::Unary(*op),
            vec![context
                .value_type(value)
                .map_err(|error| source_failure(error.to_string()))?],
        )),
        VirInstruction::Index { base, index, .. } => Ok((
            VirSafetyOperation::Index,
            vec![
                context
                    .value_type(base)
                    .map_err(|error| source_failure(error.to_string()))?,
                context
                    .value_type(index)
                    .map_err(|error| source_failure(error.to_string()))?,
            ],
        )),
        other => Ok((VirSafetyOperation::None(other.kind()), Vec::new())),
    }
}

fn instruction_checks(instruction: &VirInstruction) -> &[VirSafetyCheck] {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn instruction_checks_mut(instruction: &mut VirInstruction) -> &mut Vec<VirSafetyCheck> {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn canonical_successor_vc_json(
    document: &SuccessorVcDocument,
) -> Result<Vec<u8>, SuccessorVcError> {
    canonical_serializable(document, VC_CANONICAL_JSON_BYTES_MAX)
}

fn canonical_successor_skeleton_json(
    skeleton: &SuccessorVcCertificateSkeleton,
) -> Result<Vec<u8>, SuccessorVcError> {
    canonical_serializable(skeleton, VC_CANONICAL_SKELETON_JSON_BYTES_MAX)
}

fn canonical_serializable<T: Serialize>(
    value: &T,
    maximum: u64,
) -> Result<Vec<u8>, SuccessorVcError> {
    let bytes = serialize_json_bounded(
        value,
        usize::try_from(maximum).map_err(|_| limit_failure("canonical maximum overflows"))?,
    )
    .map_err(|error| match error {
        BoundedJsonSerializeError::OutputBytesExceeded { .. } => {
            limit_failure("canonical output exceeds its verification limit")
        }
        BoundedJsonSerializeError::Serialize(detail) => failure(
            SuccessorVcValidationPhase::Shape,
            SuccessorVcErrorCode::Shape,
            detail,
        ),
    })?;
    let strict = parse_strict_json(&bytes, SUCCESSOR_VC_LIMITS).map_err(|error| {
        failure(
            SuccessorVcValidationPhase::Shape,
            SuccessorVcErrorCode::Shape,
            error.to_string(),
        )
    })?;
    canonical_json_bytes_bounded(
        &strict,
        usize::try_from(maximum).map_err(|_| limit_failure("canonical maximum overflows"))?,
    )
    .map_err(|error| limit_failure(error.to_string()))
}

fn canonical_value(value: &Value, maximum: u64) -> Result<Vec<u8>, SuccessorVcError> {
    canonical_serializable(value, maximum)
}

fn parse_wire<T: for<'de> Deserialize<'de>>(
    input: &[u8],
    maximum: u64,
    phase: SuccessorVcValidationPhase,
) -> Result<(Vec<u8>, T), SuccessorVcError> {
    let strict = parse_strict_json(input, SUCCESSOR_VC_LIMITS)
        .map_err(|error| failure(phase, SuccessorVcErrorCode::Json, error.to_string()))?;
    let canonical = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(maximum).map_err(|_| limit_failure("canonical maximum overflows"))?,
    )
    .map_err(|error| limit_failure(error.to_string()))?;
    let wire = serde_json::from_slice(&canonical).map_err(|error| {
        failure(
            SuccessorVcValidationPhase::Shape,
            SuccessorVcErrorCode::Shape,
            error.to_string(),
        )
    })?;
    Ok((canonical, wire))
}

fn map_active_vc_error(error: crate::VcValidationError) -> SuccessorVcError {
    failure(
        SuccessorVcValidationPhase::Members,
        SuccessorVcErrorCode::Members,
        error.to_string(),
    )
}

fn source_failure(detail: impl Into<String>) -> SuccessorVcError {
    failure(
        SuccessorVcValidationPhase::SourceLinkage,
        SuccessorVcErrorCode::SourceLinkage,
        detail,
    )
}

fn profile_failure(detail: impl Into<String>) -> SuccessorVcError {
    failure(
        SuccessorVcValidationPhase::ProfileContract,
        SuccessorVcErrorCode::ProfileContract,
        detail,
    )
}

fn limit_failure(detail: impl Into<String>) -> SuccessorVcError {
    failure(
        SuccessorVcValidationPhase::CanonicalSize,
        SuccessorVcErrorCode::Limit,
        detail,
    )
}

fn failure(
    phase: SuccessorVcValidationPhase,
    code: SuccessorVcErrorCode,
    detail: impl Into<String>,
) -> SuccessorVcError {
    SuccessorVcError {
        phase,
        code,
        detail: detail.into(),
    }
}
