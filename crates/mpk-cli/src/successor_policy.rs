//! Active policy, evidence, and program-certificate integration.
//!
//! The entry points accept completely validated successor artifact graphs.
//! Policy and evidence remain helper documents; proof acceptance comes only
//! from unchanged Certificate v0 bytes submitted to both source-free checkers
//! by `program_certificate`.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mpk_cert::encode::SourceManifest as CertificateSourceManifest;
use mpk_cert::{axiom_report_hash_for_report, certificate_hash, export_block_hash, hash_hex};
use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_semantic_context_linkage, CompiledProfileEnvelope,
    CompiledSemanticProfile, ProfileContractField, SelectionEnvelope, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, successor_source_manifest_hash_value,
    SuccessorSourceManifestStage, SuccessorSourceManifestValidationContext,
    ValidatedSuccessorSourceManifest, ValidatedSuccessorSourceMap, ValidatedSuccessorVir,
    SUCCESSOR_SOURCE_MANIFEST_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::successor_vc::{
    import_successor_vc_json, import_successor_vc_skeleton_json, SuccessorVcSource,
    ValidatedSuccessorVc, ValidatedSuccessorVcSkeleton, SUCCESSOR_VC_SCHEMA,
};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, serialize_json_bounded, CapturedInput,
    FrontendIdentity, InputKind, ReleaseRegistryIdentity, StrictJsonLimits, ToolchainIdentity,
};
use serde::Serialize;
use serde_json::{json, Value};

use crate::policy_schema::{
    PolicyAxiomCategoryCountsV1, PolicyAxiomReportV1, PolicyCertificateEvidenceV1,
    PolicyCheckedDeclaration, PolicyCheckerVerdictV1, PolicyDeclarationDependency,
    PolicyEvidenceReferenceV1, PolicyHelperArtifact, PolicyIssue, PolicyMemberRowV1,
    PolicyPropertyV1, PolicyReproductionRecipeV1, PolicyTrustedEvidenceV1,
    PolicyVerificationOptions, POLICY_JSON_NESTING_MAX, POLICY_JSON_TRANSPORT_BYTES_MAX,
    POLICY_STRING_BYTES_MAX,
};
use crate::program_certificate::{
    assemble_program_certificate_alpha_from_functions, ProgramCertificateError,
    ProgramCertificateOutcome, ProgramCheckerVerdict,
};

pub const SUCCESSOR_POLICY_SCAN_SCHEMA: &str = "mpk.policy.scan.v2";
pub const SUCCESSOR_POLICY_EVIDENCE_SCHEMA: &str = "mpk.policy.evidence.v2";
pub const SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE: &str = "mpk.program_certificate.alpha.v1";

const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SUCCESSOR_POLICY_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    POLICY_JSON_TRANSPORT_BYTES_MAX,
    67_108_865,
    POLICY_JSON_NESTING_MAX,
    POLICY_STRING_BYTES_MAX,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorPolicyPhase {
    SourceLinkage,
    ProfileContract,
    CertificateManifest,
    CertificateAssembly,
    EvidenceProjection,
    Transport,
    CanonicalTransport,
    DocumentLinkage,
}

impl SuccessorPolicyPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceLinkage => "source_linkage",
            Self::ProfileContract => "profile_contract",
            Self::CertificateManifest => "certificate_manifest",
            Self::CertificateAssembly => "certificate_assembly",
            Self::EvidenceProjection => "evidence_projection",
            Self::Transport => "transport",
            Self::CanonicalTransport => "canonical_transport",
            Self::DocumentLinkage => "document_linkage",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorPolicyCode {
    SourceLinkage,
    ProfileContract,
    CertificateManifest,
    CertificateAssembly,
    EvidenceProjection,
    Json,
    CanonicalTransport,
    DocumentLinkage,
}

impl SuccessorPolicyCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceLinkage => "SUCCESSOR_POLICY_SOURCE_LINKAGE",
            Self::ProfileContract => "SUCCESSOR_POLICY_PROFILE_CONTRACT",
            Self::CertificateManifest => "SUCCESSOR_POLICY_CERTIFICATE_MANIFEST",
            Self::CertificateAssembly => "SUCCESSOR_POLICY_CERTIFICATE_ASSEMBLY",
            Self::EvidenceProjection => "SUCCESSOR_POLICY_EVIDENCE_PROJECTION",
            Self::Json => "SUCCESSOR_POLICY_JSON",
            Self::CanonicalTransport => "SUCCESSOR_POLICY_CANONICAL_TRANSPORT",
            Self::DocumentLinkage => "SUCCESSOR_POLICY_DOCUMENT_LINKAGE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorPolicyError {
    phase: SuccessorPolicyPhase,
    code: SuccessorPolicyCode,
    detail: String,
}

impl SuccessorPolicyError {
    pub const fn phase(&self) -> SuccessorPolicyPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorPolicyCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for SuccessorPolicyError {
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

impl Error for SuccessorPolicyError {}

#[derive(Clone, Copy)]
pub struct SuccessorPolicySource<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub vir: &'a ValidatedSuccessorVir,
    pub source_map: &'a ValidatedSuccessorSourceMap,
    pub frontend_manifest: &'a ValidatedSuccessorSourceManifest,
    pub vc: &'a ValidatedSuccessorVc,
    pub skeleton: &'a ValidatedSuccessorVcSkeleton,
    pub policy_contract: &'a Value,
    pub evidence_contract: &'a Value,
    pub captured_inputs: &'a [CapturedInput<'a>],
}

/// Frontend-only source boundary for successor policy scan. Unlike verify,
/// scan does not require a VC, skeleton, certificate manifest, or checker run.
#[derive(Clone, Copy)]
pub struct SuccessorPolicyScanSource<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub vir: &'a ValidatedSuccessorVir,
    pub source_map: &'a ValidatedSuccessorSourceMap,
    pub frontend_manifest: &'a ValidatedSuccessorSourceManifest,
    pub policy_contract: &'a Value,
    pub captured_inputs: &'a [CapturedInput<'a>],
}

impl<'a> From<SuccessorPolicySource<'a>> for SuccessorPolicyScanSource<'a> {
    fn from(source: SuccessorPolicySource<'a>) -> Self {
        Self {
            registry: source.registry,
            vir: source.vir,
            source_map: source.source_map,
            frontend_manifest: source.frontend_manifest,
            policy_contract: source.policy_contract,
            captured_inputs: source.captured_inputs,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuccessorPolicyRegistration {
    profile: CompiledSemanticProfile,
    strategy_profile: &'static str,
    checker_profile: &'static str,
    axiom_profile: &'static str,
    recipe_profile_id: &'static str,
}

impl SuccessorPolicyRegistration {
    pub const fn profile(self) -> CompiledSemanticProfile {
        self.profile
    }

    pub const fn strategy_profile(self) -> &'static str {
        self.strategy_profile
    }

    pub const fn checker_profile(self) -> &'static str {
        self.checker_profile
    }

    pub const fn axiom_profile(self) -> &'static str {
        self.axiom_profile
    }

    pub const fn recipe_profile_id(self) -> &'static str {
        self.recipe_profile_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorPolicyScanV2 {
    schema: String,
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    policy_contract: CompiledProfileEnvelope,
    frontend_status: String,
    frontend_phase: String,
    readiness: String,
    rejected_features: Vec<PolicyIssue>,
    diagnostics: Vec<PolicyIssue>,
    limit_profile: String,
    release_registry: ReleaseRegistryIdentity,
    frontend: FrontendIdentity,
    toolchain: ToolchainIdentity,
    frontend_source_manifest_hash: String,
    input_set_hash: String,
    source_map_hash: String,
    source_ir_schema: String,
    source_ir_hash: String,
    helper_artifacts: Vec<PolicyHelperArtifact>,
}

impl SuccessorPolicyScanV2 {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }

    pub fn policy_contract(&self) -> &CompiledProfileEnvelope {
        &self.policy_contract
    }

    pub fn helper_artifacts(&self) -> &[PolicyHelperArtifact] {
        &self.helper_artifacts
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorPolicyEvidenceV2 {
    schema: String,
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    policy_contract: CompiledProfileEnvelope,
    evidence_contract: CompiledProfileEnvelope,
    limit_profile: String,
    release_registry: ReleaseRegistryIdentity,
    frontend: FrontendIdentity,
    toolchain: ToolchainIdentity,
    frontend_source_manifest_hash: String,
    input_set_hash: String,
    source_map_hash: String,
    source_ir_schema: String,
    source_ir_hash: String,
    certificate_source_manifest_hash: String,
    source_vc_schema: String,
    vc_hash: String,
    verification_limit_profile: String,
    program_certificate_profile: String,
    verification_options: PolicyVerificationOptions,
    helper_artifacts: Vec<PolicyHelperArtifact>,
    trusted_evidence: PolicyTrustedEvidenceV1,
    properties: Vec<PolicyPropertyV1>,
    reproduction_recipes: Vec<PolicyReproductionRecipeV1>,
}

impl SuccessorPolicyEvidenceV2 {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }

    pub fn policy_contract(&self) -> &CompiledProfileEnvelope {
        &self.policy_contract
    }

    pub fn evidence_contract(&self) -> &CompiledProfileEnvelope {
        &self.evidence_contract
    }

    pub fn program_certificate_profile(&self) -> &str {
        &self.program_certificate_profile
    }

    pub fn trusted_evidence(&self) -> &PolicyTrustedEvidenceV1 {
        &self.trusted_evidence
    }

    pub fn properties(&self) -> &[PolicyPropertyV1] {
        &self.properties
    }

    pub fn helper_artifacts(&self) -> &[PolicyHelperArtifact] {
        &self.helper_artifacts
    }

    pub fn reproduction_recipes(&self) -> &[PolicyReproductionRecipeV1] {
        &self.reproduction_recipes
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorPolicyScanV2 {
    document: SuccessorPolicyScanV2,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorPolicyScanV2 {
    pub fn document(&self) -> &SuccessorPolicyScanV2 {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorPolicyEvidenceV2 {
    document: SuccessorPolicyEvidenceV2,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorPolicyEvidenceV2 {
    pub fn document(&self) -> &SuccessorPolicyEvidenceV2 {
        &self.document
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Debug)]
pub struct SuccessorPolicyRun {
    registration: SuccessorPolicyRegistration,
    scan: ValidatedSuccessorPolicyScanV2,
    certificate_manifest: ValidatedSuccessorSourceManifest,
    program_certificate: ProgramCertificateOutcome,
    evidence: ValidatedSuccessorPolicyEvidenceV2,
}

impl SuccessorPolicyRun {
    pub const fn registration(&self) -> SuccessorPolicyRegistration {
        self.registration
    }

    pub fn scan(&self) -> &ValidatedSuccessorPolicyScanV2 {
        &self.scan
    }

    pub fn certificate_manifest(&self) -> &ValidatedSuccessorSourceManifest {
        &self.certificate_manifest
    }

    pub fn program_certificate(&self) -> &ProgramCertificateOutcome {
        &self.program_certificate
    }

    pub fn evidence(&self) -> &ValidatedSuccessorPolicyEvidenceV2 {
        &self.evidence
    }

    /// Reimports candidate scan bytes against this complete validated run.
    pub fn import_scan_json(
        &self,
        input: &[u8],
    ) -> Result<ValidatedSuccessorPolicyScanV2, SuccessorPolicyError> {
        validate_exact_document(input, &self.scan.document, "successor policy scan")?;
        Ok(ValidatedSuccessorPolicyScanV2 {
            document: self.scan.document.clone(),
            canonical_bytes: input.to_vec(),
        })
    }

    /// Reimports candidate evidence bytes against this complete validated run.
    pub fn import_evidence_json(
        &self,
        input: &[u8],
    ) -> Result<ValidatedSuccessorPolicyEvidenceV2, SuccessorPolicyError> {
        validate_exact_document(input, &self.evidence.document, "successor policy evidence")?;
        Ok(ValidatedSuccessorPolicyEvidenceV2 {
            document: self.evidence.document.clone(),
            canonical_bytes: input.to_vec(),
        })
    }
}

struct PreparedPolicySource {
    registration: SuccessorPolicyRegistration,
    policy_contract: CompiledProfileEnvelope,
    evidence_contract: CompiledProfileEnvelope,
}

struct PreparedPolicyScanSource {
    registration: SuccessorPolicyRegistration,
    policy_contract: CompiledProfileEnvelope,
}

/// Generates a canonical successor scan from frontend-stage artifacts only.
/// Certificate assembly failures therefore cannot suppress valid scan output.
pub fn generate_successor_policy_scan(
    source: SuccessorPolicyScanSource<'_>,
) -> Result<ValidatedSuccessorPolicyScanV2, SuccessorPolicyError> {
    let prepared = prepare_scan_source(source)?;
    validated_scan(source, &prepared)
}

/// Imports canonical successor scan bytes by regenerating the complete
/// document from the explicitly injected frontend-stage source graph.
pub fn import_successor_policy_scan_json(
    input: &[u8],
    source: SuccessorPolicyScanSource<'_>,
) -> Result<ValidatedSuccessorPolicyScanV2, SuccessorPolicyError> {
    let prepared = prepare_scan_source(source)?;
    let document = expected_scan(source, &prepared)?;
    validate_exact_document(input, &document, "successor policy scan")?;
    Ok(ValidatedSuccessorPolicyScanV2 {
        document,
        canonical_bytes: input.to_vec(),
    })
}

/// Runs the active successor policy pipeline. No document is returned until
/// all source linkage, profile
/// contracts, Certificate v0 assembly, checker execution, and canonical
/// regeneration checks have completed.
pub fn run_successor_policy(
    source: SuccessorPolicySource<'_>,
    verification_options: PolicyVerificationOptions,
) -> Result<SuccessorPolicyRun, SuccessorPolicyError> {
    let prepared = prepare_source(source)?;
    let certificate_manifest = attach_successor_vc_hash(source)?;
    let certificate_source_manifest = CertificateSourceManifest {
        payload: certificate_manifest.canonical_bytes().to_vec(),
    };
    let program_certificate = assemble_program_certificate_alpha_from_functions(
        source.vc.document().functions(),
        source.skeleton.skeleton().theorem_declarations(),
        certificate_source_manifest.clone(),
    )
    .map_err(certificate_error)?;
    validate_certificate_manifest_binding(&program_certificate, &certificate_source_manifest)?;

    let scan_source = SuccessorPolicyScanSource::from(source);
    let scan_prepared = PreparedPolicyScanSource {
        registration: prepared.registration,
        policy_contract: prepared.policy_contract.clone(),
    };
    let scan = validated_scan(scan_source, &scan_prepared)?;

    let evidence_document = expected_evidence(
        source,
        &prepared,
        &certificate_manifest,
        &program_certificate,
        verification_options,
    )?;
    let evidence_bytes = canonical_document(&evidence_document)?;
    validate_exact_document(
        &evidence_bytes,
        &evidence_document,
        "generated successor policy evidence",
    )?;
    let evidence = ValidatedSuccessorPolicyEvidenceV2 {
        document: evidence_document,
        canonical_bytes: evidence_bytes,
    };

    Ok(SuccessorPolicyRun {
        registration: prepared.registration,
        scan,
        certificate_manifest,
        program_certificate,
        evidence,
    })
}

fn prepare_source(
    source: SuccessorPolicySource<'_>,
) -> Result<PreparedPolicySource, SuccessorPolicyError> {
    let prepared_scan = prepare_scan_source(source.into())?;
    let context = source.vir.module().semantic_context();
    validate_semantic_context_linkage(context, source.vc.document().semantic_context())
        .and_then(|_| {
            validate_semantic_context_linkage(
                context,
                source.skeleton.skeleton().semantic_context(),
            )
        })
        .map_err(|error| {
            failure(
                SuccessorPolicyPhase::SourceLinkage,
                SuccessorPolicyCode::SourceLinkage,
                error.to_string(),
            )
        })?;
    let manifest = source.frontend_manifest.manifest();
    if source.vc.document().source_manifest_schema() != SUCCESSOR_SOURCE_MANIFEST_SCHEMA
        || source.vc.document().source_manifest_hash() != source.frontend_manifest.hash()
        || source.vc.document().source_ir_schema() != SUCCESSOR_VIR_SCHEMA
        || source.vc.document().source_ir_hash() != source.vir.hash()
        || source.vc.document().input_set_hash() != manifest.input_set_hash()
        || source.skeleton.skeleton().source_vc_schema() != SUCCESSOR_VC_SCHEMA
        || source.skeleton.skeleton().source_vc_hash() != source.vc.hash()
        || source.skeleton.skeleton().source_manifest_hash() != source.frontend_manifest.hash()
    {
        return Err(failure(
            SuccessorPolicyPhase::SourceLinkage,
            SuccessorPolicyCode::SourceLinkage,
            "successor policy inputs do not form one exact artifact graph",
        ));
    }

    let vc_contract =
        serde_json::to_value(source.vc.document().profile_contract()).map_err(|error| {
            failure(
                SuccessorPolicyPhase::SourceLinkage,
                SuccessorPolicyCode::SourceLinkage,
                format!("serialize successor VC profile contract: {error}"),
            )
        })?;
    let vc_source = SuccessorVcSource {
        registry: source.registry,
        vir: source.vir,
        manifest: source.frontend_manifest,
        profile_contract: &vc_contract,
    };
    import_successor_vc_json(source.vc.canonical_bytes(), vc_source)
        .and_then(|vc| {
            import_successor_vc_skeleton_json(
                source.skeleton.canonical_bytes(),
                vc.canonical_bytes(),
                vc_source,
            )
        })
        .map_err(|error| {
            failure(
                SuccessorPolicyPhase::SourceLinkage,
                SuccessorPolicyCode::SourceLinkage,
                format!("reimport successor VC graph: {error}"),
            )
        })?;

    let evidence_contract = validate_compiled_profile_envelope(
        source.registry,
        source.evidence_contract,
        ProfileContractField::Evidence,
    )
    .map_err(profile_error)?;
    if evidence_contract.profile_entry_sha256() != context.profile_entry_sha256() {
        return Err(failure(
            SuccessorPolicyPhase::ProfileContract,
            SuccessorPolicyCode::ProfileContract,
            "evidence contract belongs to another semantic entry",
        ));
    }
    validate_evidence_registration(&evidence_contract, prepared_scan.registration)?;

    Ok(PreparedPolicySource {
        registration: prepared_scan.registration,
        policy_contract: prepared_scan.policy_contract,
        evidence_contract,
    })
}

fn prepare_scan_source(
    source: SuccessorPolicyScanSource<'_>,
) -> Result<PreparedPolicyScanSource, SuccessorPolicyError> {
    if source.frontend_manifest.stage() != SuccessorSourceManifestStage::Frontend {
        return Err(failure(
            SuccessorPolicyPhase::SourceLinkage,
            SuccessorPolicyCode::SourceLinkage,
            "policy scan requires a frontend-stage successor manifest",
        ));
    }
    let context = source.vir.module().semantic_context();
    validate_semantic_context_linkage(context, source.source_map.map().semantic_context())
        .and_then(|_| {
            validate_semantic_context_linkage(
                context,
                source.frontend_manifest.manifest().semantic_context(),
            )
        })
        .map_err(|error| {
            failure(
                SuccessorPolicyPhase::SourceLinkage,
                SuccessorPolicyCode::SourceLinkage,
                error.to_string(),
            )
        })?;
    let manifest = source.frontend_manifest.manifest();
    if source.source_map.map().source_ir_schema() != SUCCESSOR_VIR_SCHEMA
        || source.source_map.map().source_ir_hash() != source.vir.hash()
        || manifest.vir_hash() != source.vir.hash()
        || manifest.source_map_hash() != source.source_map.hash()
    {
        return Err(failure(
            SuccessorPolicyPhase::SourceLinkage,
            SuccessorPolicyCode::SourceLinkage,
            "successor policy scan inputs do not form one exact frontend artifact graph",
        ));
    }
    import_successor_source_manifest_json(
        source.frontend_manifest.canonical_bytes(),
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: source.registry,
            vir: source.vir,
            source_map: source.source_map,
            captured_inputs: source.captured_inputs,
            expected_release_registry: manifest.release_registry(),
        },
    )
    .map_err(|error| {
        failure(
            SuccessorPolicyPhase::SourceLinkage,
            SuccessorPolicyCode::SourceLinkage,
            format!("reimport successor frontend manifest: {error}"),
        )
    })?;

    let profile = CompiledSemanticProfile::from_identity(
        context.source_language(),
        context.semantic_profile(),
    )
    .ok_or_else(|| {
        failure(
            SuccessorPolicyPhase::ProfileContract,
            SuccessorPolicyCode::ProfileContract,
            "semantic context does not select a compiled policy profile",
        )
    })?;
    let entry = source
        .registry
        .lookup(context.source_language(), context.semantic_profile())
        .ok_or_else(|| {
            failure(
                SuccessorPolicyPhase::ProfileContract,
                SuccessorPolicyCode::ProfileContract,
                "semantic profile entry is absent",
            )
        })?;
    if entry.compiled_profile() != profile || entry.entry_sha256() != context.profile_entry_sha256()
    {
        return Err(failure(
            SuccessorPolicyPhase::ProfileContract,
            SuccessorPolicyCode::ProfileContract,
            "semantic profile entry linkage differs",
        ));
    }
    let policy_contract = validate_compiled_profile_envelope(
        source.registry,
        source.policy_contract,
        ProfileContractField::Policy,
    )
    .map_err(profile_error)?;
    if policy_contract.profile_entry_sha256() != context.profile_entry_sha256() {
        return Err(failure(
            SuccessorPolicyPhase::ProfileContract,
            SuccessorPolicyCode::ProfileContract,
            "policy contract belongs to another semantic entry",
        ));
    }
    let registration = registration(profile);
    validate_policy_registration(&policy_contract, registration)?;

    Ok(PreparedPolicyScanSource {
        registration,
        policy_contract,
    })
}

fn attach_successor_vc_hash(
    source: SuccessorPolicySource<'_>,
) -> Result<ValidatedSuccessorSourceManifest, SuccessorPolicyError> {
    let mut value = serde_json::to_value(source.frontend_manifest.manifest()).map_err(|error| {
        failure(
            SuccessorPolicyPhase::CertificateManifest,
            SuccessorPolicyCode::CertificateManifest,
            format!("serialize frontend-stage manifest: {error}"),
        )
    })?;
    value["vc_hash"] = Value::String(source.vc.hash().as_str().to_owned());
    value["source_manifest_hash"] = Value::String(ZERO_SHA256.to_owned());
    value["source_manifest_hash"] = Value::String(
        successor_source_manifest_hash_value(&value)
            .map_err(|error| {
                failure(
                    SuccessorPolicyPhase::CertificateManifest,
                    SuccessorPolicyCode::CertificateManifest,
                    error.to_string(),
                )
            })?
            .as_str()
            .to_owned(),
    );
    let bytes = canonical_document(&value)?;
    let expected_release_registry = source.frontend_manifest.manifest().release_registry();
    let certificate = import_successor_source_manifest_json(
        &bytes,
        SuccessorSourceManifestStage::Certificate,
        SuccessorSourceManifestValidationContext {
            registry: source.registry,
            vir: source.vir,
            source_map: source.source_map,
            captured_inputs: source.captured_inputs,
            expected_release_registry,
        },
    )
    .map_err(|error| {
        failure(
            SuccessorPolicyPhase::CertificateManifest,
            SuccessorPolicyCode::CertificateManifest,
            error.to_string(),
        )
    })?;
    if certificate.manifest().vc_hash() != Some(source.vc.hash()) {
        return Err(failure(
            SuccessorPolicyPhase::CertificateManifest,
            SuccessorPolicyCode::CertificateManifest,
            "certificate-stage manifest did not retain the exact successor VC hash",
        ));
    }
    Ok(certificate)
}

fn expected_scan(
    source: SuccessorPolicyScanSource<'_>,
    prepared: &PreparedPolicyScanSource,
) -> Result<SuccessorPolicyScanV2, SuccessorPolicyError> {
    let manifest = source.frontend_manifest.manifest();
    Ok(SuccessorPolicyScanV2 {
        schema: SUCCESSOR_POLICY_SCAN_SCHEMA.to_owned(),
        semantic_context: source.vir.module().semantic_context().clone(),
        selection: manifest.selection().clone(),
        policy_contract: prepared.policy_contract.clone(),
        frontend_status: "ir-lowered".to_owned(),
        frontend_phase: "emission".to_owned(),
        readiness: "ready".to_owned(),
        rejected_features: Vec::new(),
        diagnostics: Vec::new(),
        limit_profile: manifest.limit_profile().to_owned(),
        release_registry: manifest.release_registry().clone(),
        frontend: manifest.frontend().clone(),
        toolchain: manifest.toolchain().clone(),
        frontend_source_manifest_hash: source.frontend_manifest.hash().as_str().to_owned(),
        input_set_hash: manifest.input_set_hash().as_str().to_owned(),
        source_map_hash: source.source_map.hash().as_str().to_owned(),
        source_ir_schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
        source_ir_hash: source.vir.hash().as_str().to_owned(),
        helper_artifacts: frontend_helper_artifacts(source)?,
    })
}

fn validated_scan(
    source: SuccessorPolicyScanSource<'_>,
    prepared: &PreparedPolicyScanSource,
) -> Result<ValidatedSuccessorPolicyScanV2, SuccessorPolicyError> {
    let document = expected_scan(source, prepared)?;
    let canonical_bytes = canonical_document(&document)?;
    validate_exact_document(
        &canonical_bytes,
        &document,
        "generated successor policy scan",
    )?;
    Ok(ValidatedSuccessorPolicyScanV2 {
        document,
        canonical_bytes,
    })
}

fn expected_evidence(
    source: SuccessorPolicySource<'_>,
    prepared: &PreparedPolicySource,
    certificate_manifest: &ValidatedSuccessorSourceManifest,
    outcome: &ProgramCertificateOutcome,
    verification_options: PolicyVerificationOptions,
) -> Result<SuccessorPolicyEvidenceV2, SuccessorPolicyError> {
    let declaration_hashes = declaration_hashes(outcome)?;
    let declarations = checked_declarations(source, &declaration_hashes)?;
    let verified = matches!(outcome, ProgramCertificateOutcome::Candidate(_));
    let trusted_evidence = trusted_evidence(
        outcome,
        &declarations,
        prepared.registration.checker_profile,
    )?;
    let properties = properties(source, &declaration_hashes, verified)?;
    let manifest = source.frontend_manifest.manifest();
    Ok(SuccessorPolicyEvidenceV2 {
        schema: SUCCESSOR_POLICY_EVIDENCE_SCHEMA.to_owned(),
        semantic_context: source.vir.module().semantic_context().clone(),
        selection: manifest.selection().clone(),
        policy_contract: prepared.policy_contract.clone(),
        evidence_contract: prepared.evidence_contract.clone(),
        limit_profile: manifest.limit_profile().to_owned(),
        release_registry: manifest.release_registry().clone(),
        frontend: manifest.frontend().clone(),
        toolchain: manifest.toolchain().clone(),
        frontend_source_manifest_hash: source.frontend_manifest.hash().as_str().to_owned(),
        input_set_hash: manifest.input_set_hash().as_str().to_owned(),
        source_map_hash: source.source_map.hash().as_str().to_owned(),
        source_ir_schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
        source_ir_hash: source.vir.hash().as_str().to_owned(),
        certificate_source_manifest_hash: certificate_manifest.hash().as_str().to_owned(),
        source_vc_schema: SUCCESSOR_VC_SCHEMA.to_owned(),
        vc_hash: source.vc.hash().as_str().to_owned(),
        verification_limit_profile: source.vc.document().verification_limit_profile().to_owned(),
        program_certificate_profile: SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE.to_owned(),
        verification_options: verification_options.clone(),
        helper_artifacts: verification_helper_artifacts(source)?,
        trusted_evidence,
        properties,
        reproduction_recipes: reproduction_recipes(source, prepared, verification_options)?,
    })
}

fn frontend_helper_artifacts(
    source: SuccessorPolicyScanSource<'_>,
) -> Result<Vec<PolicyHelperArtifact>, SuccessorPolicyError> {
    let mut helpers = source
        .frontend_manifest
        .manifest()
        .inputs()
        .iter()
        .filter(|input| input.kind == InputKind::Source)
        .map(|input| PolicyHelperArtifact::Source {
            id: format!("source:{}", input.normalized_path),
            normalized_path: input.normalized_path.clone(),
            sha256: input.sha256.clone(),
        })
        .collect::<Vec<_>>();
    for input in source
        .frontend_manifest
        .manifest()
        .inputs()
        .iter()
        .filter(|input| input.kind == InputKind::Contract)
    {
        let captured = source
            .captured_inputs
            .iter()
            .find(|captured| {
                captured.kind == InputKind::Contract
                    && captured.normalized_path == input.normalized_path
            })
            .ok_or_else(|| source_error("manifest contract bytes are not retained"))?;
        let (schema, raw_function) = contract_identity(
            captured.bytes,
            source.vir.module().semantic_context().source_language(),
        )?;
        let function = resolve_contract_function(
            source.vir,
            source.vir.module().semantic_context().source_language(),
            &raw_function,
        )?;
        helpers.push(PolicyHelperArtifact::Contract {
            id: format!("contract:{}", function.id()),
            normalized_path: input.normalized_path.clone(),
            schema,
            raw_input_sha256: input.sha256.clone(),
            function_id: function.id().to_owned(),
            contract_hash: function.contracts().contract_hash().as_str().to_owned(),
        });
    }
    helpers.push(PolicyHelperArtifact::VerificationIr {
        id: "verification_ir".to_owned(),
        schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
        sha256: source.vir.hash().as_str().to_owned(),
    });
    helpers.sort_by(|left, right| {
        (helper_rank(left), left.id().as_bytes()).cmp(&(helper_rank(right), right.id().as_bytes()))
    });
    if helpers.windows(2).any(|pair| pair[0].id() == pair[1].id()) {
        return Err(source_error(
            "successor frontend produced duplicate helper artifact IDs",
        ));
    }
    Ok(helpers)
}

fn verification_helper_artifacts(
    source: SuccessorPolicySource<'_>,
) -> Result<Vec<PolicyHelperArtifact>, SuccessorPolicyError> {
    let mut helpers = frontend_helper_artifacts(source.into())?;
    helpers.push(PolicyHelperArtifact::Vc {
        id: "vc".to_owned(),
        schema: SUCCESSOR_VC_SCHEMA.to_owned(),
        sha256: source.vc.hash().as_str().to_owned(),
    });
    Ok(helpers)
}

fn contract_identity(
    bytes: &[u8],
    source_language: &str,
) -> Result<(String, String), SuccessorPolicyError> {
    parse_strict_json(bytes, SUCCESSOR_POLICY_LIMITS)
        .map_err(|error| source_error(format!("parse validated successor contract: {error}")))?;
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|error| source_error(format!("decode validated successor contract: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| source_error("validated successor contract is not an object"))?;
    let (expected_schema, function_field) = match source_language {
        "go" => ("mpk.go.contract.v0", "function"),
        "rust" => ("mpk.rust.contract.v0", "function"),
        "csharp" => ("mpk.csharp.contract.v0", "method"),
        _ => {
            return Err(source_error(
                "successor contract selected an uncompiled source language",
            ));
        }
    };
    let schema = object
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| source_error("validated successor contract schema is absent"))?;
    if schema != expected_schema {
        return Err(source_error(
            "validated successor contract schema differs from the source language",
        ));
    }
    let function = object
        .get(function_field)
        .and_then(Value::as_str)
        .filter(|function| !function.is_empty())
        .ok_or_else(|| source_error("validated successor contract function is absent"))?;
    Ok((
        schema.to_owned(),
        if source_language == "go" {
            function.trim()
        } else {
            function
        }
        .to_owned(),
    ))
}

fn resolve_contract_function<'a>(
    vir: &'a ValidatedSuccessorVir,
    source_language: &str,
    raw_function: &str,
) -> Result<&'a mpk_vc::successor_source_artifacts::SuccessorVirFunction, SuccessorPolicyError> {
    let mut matches = vir
        .module()
        .units()
        .iter()
        .flat_map(|unit| unit.functions())
        .filter(|function| {
            function.id() == raw_function
                || (source_language == "go"
                    && (function
                        .id()
                        .rsplit_once('/')
                        .is_some_and(|(_, suffix)| suffix == raw_function)
                        || function.id().ends_with(&format!(".{raw_function}"))))
        });
    let function = matches
        .next()
        .ok_or_else(|| source_error("contract function is absent from successor VIR"))?;
    if matches.next().is_some() {
        return Err(source_error(
            "contract function is ambiguous in successor VIR",
        ));
    }
    Ok(function)
}

fn helper_rank(helper: &PolicyHelperArtifact) -> u8 {
    match helper {
        PolicyHelperArtifact::Source { .. } => 0,
        PolicyHelperArtifact::Contract { .. } => 1,
        PolicyHelperArtifact::VerificationIr { .. } => 2,
        PolicyHelperArtifact::Vc { .. } => 3,
        PolicyHelperArtifact::AiAnalysis { .. } => 4,
        PolicyHelperArtifact::CiStatus { .. } => 5,
    }
}

fn declaration_hashes(
    outcome: &ProgramCertificateOutcome,
) -> Result<BTreeMap<String, String>, SuccessorPolicyError> {
    let declarations = outcome.generated_declarations();
    let hashes = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.name.clone(),
                declaration.declaration_hash.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if hashes.len() != declarations.len() {
        return Err(evidence_error(
            "program-certificate plan contains duplicate generated declaration names",
        ));
    }
    Ok(hashes)
}

fn checked_declarations(
    source: SuccessorPolicySource<'_>,
    declaration_hashes: &BTreeMap<String, String>,
) -> Result<Vec<PolicyCheckedDeclaration>, SuccessorPolicyError> {
    let skeleton = source.skeleton.skeleton().theorem_declarations();
    if declaration_hashes.len() != skeleton.len() {
        return Err(evidence_error(
            "program-certificate plan differs from the complete successor skeleton",
        ));
    }
    skeleton
        .iter()
        .map(|declaration| {
            let declaration_hash = declaration_hashes
                .get(&declaration.name)
                .ok_or_else(|| evidence_error("planned declaration hash is absent"))?
                .clone();
            let dependencies = declaration
                .dependencies
                .iter()
                .map(|name| {
                    declaration_hashes
                        .get(name)
                        .map(|declaration_hash| PolicyDeclarationDependency {
                            name: name.clone(),
                            declaration_hash: declaration_hash.clone(),
                        })
                        .ok_or_else(|| evidence_error("planned dependency hash is absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(PolicyCheckedDeclaration {
                name: declaration.name.clone(),
                declaration_hash,
                function_id: declaration.function_id.clone(),
                group_id: declaration.group_id.clone(),
                group_kind: declaration.group_kind.as_str().to_owned(),
                member_ids: declaration.member_ids.clone(),
                dependencies,
            })
        })
        .collect()
}

fn trusted_evidence(
    outcome: &ProgramCertificateOutcome,
    declarations: &[PolicyCheckedDeclaration],
    checker_profile: &str,
) -> Result<PolicyTrustedEvidenceV1, SuccessorPolicyError> {
    let (certificate, bytes, rust_verdict, reference_verdict) = match outcome {
        ProgramCertificateOutcome::Pending { .. } => {
            return Ok(PolicyTrustedEvidenceV1 {
                certificates: Vec::new(),
                theory_certificates: Vec::new(),
                axiom_report: PolicyAxiomReportV1::NotGenerated,
                checker_verdicts: not_run_verdicts(checker_profile),
            });
        }
        ProgramCertificateOutcome::Candidate(candidate) => (
            &candidate.certificate,
            candidate.bytes.as_slice(),
            ProgramCheckerVerdict::Accepted,
            ProgramCheckerVerdict::Accepted,
        ),
        ProgramCertificateOutcome::Unaccepted(candidate) => (
            &candidate.certificate,
            candidate.bytes.as_slice(),
            candidate.rust_verdict,
            candidate.reference_verdict,
        ),
    };
    let summary = &certificate.axiom_report.summary;
    if summary.total_axiom_count != 0 {
        return Err(evidence_error(
            "successor alpha program certificate is not zero-axiom",
        ));
    }
    let export_hash = export_block_hash(&certificate.export_block);
    let axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    if certificate.hashes.export_hash != export_hash
        || certificate.hashes.axiom_report_hash != axiom_report_hash
    {
        return Err(evidence_error(
            "program-certificate retained hashes differ from canonical contents",
        ));
    }
    let certificate_id = "program".to_owned();
    let certificate_ids = vec![certificate_id.clone()];
    Ok(PolicyTrustedEvidenceV1 {
        certificates: vec![PolicyCertificateEvidenceV1 {
            id: certificate_id,
            module: certificate.module.clone(),
            certificate_hash: hash_hex(&certificate_hash(bytes)),
            export_hash: hash_hex(&export_hash),
            axiom_report_hash: hash_hex(&axiom_report_hash),
            checked_declarations: declarations.to_vec(),
        }],
        theory_certificates: Vec::new(),
        axiom_report: PolicyAxiomReportV1::Checked {
            axiom_report_hash: hash_hex(&axiom_report_hash),
            category_counts: PolicyAxiomCategoryCountsV1 {
                total_axiom_count: checked_axiom_count(summary.total_axiom_count)?,
                core_axiom_count: checked_axiom_count(summary.core_axiom_count)?,
                builtin_theory_axiom_count: checked_axiom_count(
                    summary.builtin_theory_axiom_count,
                )?,
                go_semantics_axiom_count: checked_axiom_count(summary.go_semantics_axiom_count)?,
                external_axiom_count: checked_axiom_count(summary.external_axiom_count)?,
            },
        },
        checker_verdicts: vec![
            PolicyCheckerVerdictV1 {
                checker: "rust_fast_kernel".to_owned(),
                checker_profile: checker_profile.to_owned(),
                verdict: rust_verdict.as_str().to_owned(),
                certificate_ids: certificate_ids.clone(),
            },
            PolicyCheckerVerdictV1 {
                checker: "reference_checker".to_owned(),
                checker_profile: checker_profile.to_owned(),
                verdict: reference_verdict.as_str().to_owned(),
                certificate_ids,
            },
        ],
    })
}

fn checked_axiom_count(value: u64) -> Result<i64, SuccessorPolicyError> {
    i64::try_from(value).map_err(|_| evidence_error("axiom count exceeds safe policy range"))
}

fn not_run_verdicts(checker_profile: &str) -> Vec<PolicyCheckerVerdictV1> {
    ["rust_fast_kernel", "reference_checker"]
        .into_iter()
        .map(|checker| PolicyCheckerVerdictV1 {
            checker: checker.to_owned(),
            checker_profile: checker_profile.to_owned(),
            verdict: "not_run".to_owned(),
            certificate_ids: Vec::new(),
        })
        .collect()
}

fn properties(
    source: SuccessorPolicySource<'_>,
    declaration_hashes: &BTreeMap<String, String>,
    verified: bool,
) -> Result<Vec<PolicyPropertyV1>, SuccessorPolicyError> {
    let mut properties = Vec::new();
    let mut property_ids = BTreeSet::new();
    for function_id in selected_function_ids(
        source.vir.module().semantic_context(),
        source.frontend_manifest.manifest().selection(),
    )? {
        let mut matches = source
            .vc
            .document()
            .functions()
            .iter()
            .filter(|function| function.function_id == function_id);
        let function = matches
            .next()
            .ok_or_else(|| evidence_error("selected function is absent from successor VC"))?;
        if matches.next().is_some() {
            return Err(evidence_error(
                "selected function resolves more than once in successor VC",
            ));
        }
        let groups = function
            .groups
            .iter()
            .map(|group| (group.id.as_str(), group))
            .collect::<BTreeMap<_, _>>();
        if groups.len() != function.groups.len() {
            return Err(evidence_error("successor VC contains duplicate group IDs"));
        }
        let mut by_kind = BTreeMap::<&str, Vec<PolicyMemberRowV1>>::new();
        for member in &function.members {
            let group = groups
                .get(member.group_id.as_str())
                .ok_or_else(|| evidence_error("VC member group is absent"))?;
            let declaration_hash = declaration_hashes
                .get(&group.declaration_name)
                .ok_or_else(|| evidence_error("VC group declaration hash is absent"))?
                .clone();
            by_kind
                .entry(member.kind.as_str())
                .or_default()
                .push(PolicyMemberRowV1 {
                    member_id: member.id.clone(),
                    function_id: member.function_id.clone(),
                    kind: member.kind.as_str().to_owned(),
                    group_id: member.group_id.clone(),
                    declaration_name: group.declaration_name.clone(),
                    declaration_hash,
                    status: if verified {
                        "mpk_verified".to_owned()
                    } else {
                        "proof_pending".to_owned()
                    },
                    evidence: if verified {
                        vec![PolicyEvidenceReferenceV1::CheckedDeclaration {
                            certificate_id: "program".to_owned(),
                        }]
                    } else {
                        vec![PolicyEvidenceReferenceV1::HelperArtifact {
                            artifact_id: "vc".to_owned(),
                        }]
                    },
                });
        }
        if by_kind.is_empty() {
            return Err(evidence_error(
                "selected policy strategy produced no classifiable VC members",
            ));
        }
        for (kind, mut members) in by_kind {
            members
                .sort_by(|left, right| left.member_id.as_bytes().cmp(right.member_id.as_bytes()));
            let id = property_id(
                source.vir.module().semantic_context().source_language(),
                &function_id,
                kind,
            );
            if !property_ids.insert(id.clone()) {
                return Err(evidence_error(
                    "selected functions produce a duplicate normalized property ID",
                ));
            }
            properties.push(PolicyPropertyV1 {
                id,
                description: property_description(&function_id, kind),
                status: if verified {
                    "mpk_verified".to_owned()
                } else {
                    "proof_pending".to_owned()
                },
                members,
                notes: Vec::new(),
            });
        }
    }
    if properties.is_empty() {
        return Err(evidence_error(
            "successor policy strategy produced no classifiable VC members",
        ));
    }
    properties.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    Ok(properties)
}

fn selected_function_ids(
    context: &SemanticContext,
    selection: &SelectionEnvelope,
) -> Result<Vec<String>, SuccessorPolicyError> {
    let value = selection.value();
    let selected = match context.source_language() {
        "go" | "rust" => vec![value
            .get("function")
            .and_then(Value::as_str)
            .filter(|function| !function.is_empty())
            .ok_or_else(|| evidence_error("selection has no function"))?
            .to_owned()],
        "csharp" => value
            .get("methods")
            .and_then(Value::as_array)
            .ok_or_else(|| evidence_error("C# selection has no methods"))?
            .iter()
            .map(|method| {
                method
                    .as_str()
                    .filter(|method| !method.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| evidence_error("C# selection contains an invalid method"))
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(evidence_error(
                "successor policy strategy selected an uncompiled source language",
            ));
        }
    };
    if selected.is_empty() {
        return Err(evidence_error(
            "successor policy strategy selected no functions",
        ));
    }
    let unique = selected.iter().collect::<BTreeSet<_>>();
    if unique.len() != selected.len() {
        return Err(evidence_error(
            "successor policy strategy selected a function more than once",
        ));
    }
    Ok(selected)
}

fn property_id(language: &str, function: &str, kind: &str) -> String {
    let symbol = if language == "csharp" {
        function
    } else {
        function
            .rsplit(['.', ':', '/'])
            .find(|segment| !segment.is_empty())
            .unwrap_or("selected_function")
    };
    format!("{}_{}", ascii_snake(symbol), kind)
}

fn ascii_snake(value: &str) -> String {
    let mut result = String::new();
    for (index, byte) in value.bytes().enumerate() {
        if byte.is_ascii_uppercase() {
            if index > 0 && !result.ends_with('_') {
                result.push('_');
            }
            result.push(char::from(byte.to_ascii_lowercase()));
        } else if byte.is_ascii_alphanumeric() || byte == b'_' {
            result.push(char::from(byte.to_ascii_lowercase()));
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    result.trim_matches('_').to_owned()
}

fn property_description(function: &str, kind: &str) -> String {
    if kind == "postcondition"
        && (function.ends_with(".Identity")
            || function.contains("::Identity(")
            || function.contains(".Identity("))
    {
        "The selected identity result equals its input.".to_owned()
    } else {
        format!("The selected function satisfies its {kind} verification condition.")
    }
}

fn reproduction_recipes(
    source: SuccessorPolicySource<'_>,
    prepared: &PreparedPolicySource,
    options: PolicyVerificationOptions,
) -> Result<Vec<PolicyReproductionRecipeV1>, SuccessorPolicyError> {
    let context = source.vir.module().semantic_context();
    let registry = context.profile_registry();
    let manifest = source.frontend_manifest.manifest();
    let mut prefix = vec![
        "mpk".to_owned(),
        "policy".to_owned(),
        "scan".to_owned(),
        ".".to_owned(),
        "--profile-registry-id".to_owned(),
        registry.id().to_owned(),
        "--profile-registry-revision".to_owned(),
        registry.revision().to_string(),
        "--profile-registry-sha256".to_owned(),
        registry.registry_sha256().to_owned(),
        "--profile-entry-sha256".to_owned(),
        context.profile_entry_sha256().to_owned(),
        "--language".to_owned(),
        context.source_language().to_owned(),
        "--semantic-profile".to_owned(),
        context.semantic_profile().to_owned(),
        "--require-release-registry-id".to_owned(),
        manifest.release_registry().id.clone(),
        "--require-release-registry-sha256".to_owned(),
        manifest.release_registry().registry_sha256.clone(),
        "--frontend-bundle".to_owned(),
        manifest.frontend().bundle_id.clone(),
        "--toolchain-bundle".to_owned(),
        manifest.toolchain().bundle_id.clone(),
        "--target".to_owned(),
        manifest.target().id().to_owned(),
    ];
    append_selection_arguments(&mut prefix, context, manifest.selection())?;

    let mut scan = prefix.clone();
    scan.extend([
        "--json-out".to_owned(),
        "mpk-reproduction-scan.json".to_owned(),
    ]);
    prefix[2] = "verify".to_owned();
    prefix.extend([
        "--strategy-profile".to_owned(),
        prepared.registration.strategy_profile.to_owned(),
        "--checker-profile".to_owned(),
        prepared.registration.checker_profile.to_owned(),
        "--axiom-profile".to_owned(),
        prepared.registration.axiom_profile.to_owned(),
        "--program-certificate-profile".to_owned(),
        SUCCESSOR_PROGRAM_CERTIFICATE_PROFILE.to_owned(),
        "--evidence-json".to_owned(),
        "mpk-reproduction-evidence.json".to_owned(),
        "--evidence-md".to_owned(),
        "mpk-reproduction-evidence.md".to_owned(),
    ]);
    if options.strict {
        prefix.push("--strict".to_owned());
    }
    if options.update_fixtures {
        prefix.push("--update-fixtures".to_owned());
    }
    Ok(vec![
        PolicyReproductionRecipeV1 {
            label: "scan".to_owned(),
            working_directory_role: "source_root".to_owned(),
            argv: scan,
        },
        PolicyReproductionRecipeV1 {
            label: "verify".to_owned(),
            working_directory_role: "source_root".to_owned(),
            argv: prefix,
        },
    ])
}

fn append_selection_arguments(
    argv: &mut Vec<String>,
    context: &SemanticContext,
    selection: &SelectionEnvelope,
) -> Result<(), SuccessorPolicyError> {
    let value = selection.value();
    let string = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| evidence_error(format!("selection field {name:?} is absent")))
    };
    let array = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_array)
            .ok_or_else(|| evidence_error(format!("selection field {name:?} is absent")))?
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| evidence_error(format!("selection field {name:?} is invalid")))
            })
            .collect::<Result<Vec<_>, _>>()
    };
    match context.source_language() {
        "go" => argv.extend([
            "--package".to_owned(),
            string("package")?,
            "--function".to_owned(),
            string("function")?,
        ]),
        "rust" => argv.extend([
            "--package".to_owned(),
            string("package")?,
            "--crate".to_owned(),
            string("crate")?,
            "--unit-kind".to_owned(),
            string("kind")?,
            "--function".to_owned(),
            string("function")?,
        ]),
        "csharp" => {
            argv.extend(["--compilation".to_owned(), string("compilation")?]);
            append_pairs(argv, "--source", &array("sources")?);
            append_pairs(argv, "--contract", &array("contracts")?);
            append_pairs(argv, "--method", &array("methods")?);
        }
        _ => {
            return Err(evidence_error(
                "successor recipe selected an uncompiled source language",
            ));
        }
    }
    Ok(())
}

fn append_pairs(argv: &mut Vec<String>, option: &str, values: &[String]) {
    for value in values {
        argv.extend([option.to_owned(), value.clone()]);
    }
}

fn registration(profile: CompiledSemanticProfile) -> SuccessorPolicyRegistration {
    match profile {
        CompiledSemanticProfile::GoFixedV0 => SuccessorPolicyRegistration {
            profile,
            strategy_profile: "payment-policy-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "zero-axiom",
            recipe_profile_id: "mpk.go.evidence_recipe.v0",
        },
        CompiledSemanticProfile::RustCheckedV0 => SuccessorPolicyRegistration {
            profile,
            strategy_profile: "payment-policy-rust-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "mvp-theory",
            recipe_profile_id: "mpk.rust.evidence_recipe.v0",
        },
        CompiledSemanticProfile::CSharpScalarV0 => SuccessorPolicyRegistration {
            profile,
            strategy_profile: "payment-policy-csharp-alpha",
            checker_profile: "mvp-strict",
            axiom_profile: "mvp-theory",
            recipe_profile_id: "mpk.csharp.evidence_recipe.v0",
        },
    }
}

fn validate_policy_registration(
    contract: &CompiledProfileEnvelope,
    registration: SuccessorPolicyRegistration,
) -> Result<(), SuccessorPolicyError> {
    let expected = json!({
        "axiom_profile": registration.axiom_profile,
        "checker_profile": registration.checker_profile,
        "strategy_profile": registration.strategy_profile,
    });
    if contract.value() != &expected {
        return Err(profile_error(
            "policy contract does not select the compiled strategy/checker/axiom registration",
        ));
    }
    Ok(())
}

fn validate_evidence_registration(
    contract: &CompiledProfileEnvelope,
    registration: SuccessorPolicyRegistration,
) -> Result<(), SuccessorPolicyError> {
    let expected = json!({
        "proof_authority": "certificate_only",
        "recipe_profile_id": registration.recipe_profile_id,
        "require_reference_checker": true,
        "require_source_free_check": true,
    });
    if contract.value() != &expected {
        return Err(profile_error(
            "evidence contract does not select the compiled source-free recipe registration",
        ));
    }
    Ok(())
}

fn validate_certificate_manifest_binding(
    outcome: &ProgramCertificateOutcome,
    expected: &CertificateSourceManifest,
) -> Result<(), SuccessorPolicyError> {
    let retained = match outcome {
        ProgramCertificateOutcome::Pending { .. } => return Ok(()),
        ProgramCertificateOutcome::Candidate(candidate) => &candidate.certificate,
        ProgramCertificateOutcome::Unaccepted(candidate) => &candidate.certificate,
    };
    if retained.source_manifest.as_ref() != Some(expected) {
        return Err(failure(
            SuccessorPolicyPhase::CertificateAssembly,
            SuccessorPolicyCode::CertificateAssembly,
            "Certificate v0 did not embed the exact successor certificate-stage manifest bytes",
        ));
    }
    Ok(())
}

fn canonical_document<T: Serialize>(value: &T) -> Result<Vec<u8>, SuccessorPolicyError> {
    let maximum = usize::try_from(POLICY_JSON_TRANSPORT_BYTES_MAX).map_err(|_| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            "policy transport maximum exceeds usize",
        )
    })?;
    let serialized = serialize_json_bounded(value, maximum).map_err(|error| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            error.to_string(),
        )
    })?;
    let strict = parse_strict_json(&serialized, SUCCESSOR_POLICY_LIMITS).map_err(|error| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            error.to_string(),
        )
    })?;
    canonical_json_bytes_bounded(&strict, maximum).map_err(|error| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            error.to_string(),
        )
    })
}

fn validate_exact_document<T: Serialize>(
    input: &[u8],
    expected: &T,
    label: &str,
) -> Result<(), SuccessorPolicyError> {
    let strict = parse_strict_json(input, SUCCESSOR_POLICY_LIMITS).map_err(|error| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            error.to_string(),
        )
    })?;
    let maximum = usize::try_from(POLICY_JSON_TRANSPORT_BYTES_MAX).map_err(|_| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            "policy transport maximum exceeds usize",
        )
    })?;
    let canonical = canonical_json_bytes_bounded(&strict, maximum).map_err(|error| {
        failure(
            SuccessorPolicyPhase::Transport,
            SuccessorPolicyCode::Json,
            error.to_string(),
        )
    })?;
    if canonical != input {
        return Err(failure(
            SuccessorPolicyPhase::CanonicalTransport,
            SuccessorPolicyCode::CanonicalTransport,
            format!("{label} is not byte-identical canonical JSON"),
        ));
    }
    if canonical != canonical_document(expected)? {
        return Err(failure(
            SuccessorPolicyPhase::DocumentLinkage,
            SuccessorPolicyCode::DocumentLinkage,
            format!("{label} differs from complete source regeneration"),
        ));
    }
    Ok(())
}

fn certificate_error(error: ProgramCertificateError) -> SuccessorPolicyError {
    failure(
        SuccessorPolicyPhase::CertificateAssembly,
        SuccessorPolicyCode::CertificateAssembly,
        error.to_string(),
    )
}

fn profile_error(error: impl fmt::Display) -> SuccessorPolicyError {
    failure(
        SuccessorPolicyPhase::ProfileContract,
        SuccessorPolicyCode::ProfileContract,
        error.to_string(),
    )
}

fn source_error(detail: impl Into<String>) -> SuccessorPolicyError {
    failure(
        SuccessorPolicyPhase::SourceLinkage,
        SuccessorPolicyCode::SourceLinkage,
        detail,
    )
}

fn evidence_error(detail: impl Into<String>) -> SuccessorPolicyError {
    failure(
        SuccessorPolicyPhase::EvidenceProjection,
        SuccessorPolicyCode::EvidenceProjection,
        detail,
    )
}

fn failure(
    phase: SuccessorPolicyPhase,
    code: SuccessorPolicyCode,
    detail: impl Into<String>,
) -> SuccessorPolicyError {
    SuccessorPolicyError {
        phase,
        code,
        detail: detail.into(),
    }
}
