//! Candidate-only VC, skeleton, and ordinary-context assembly models.
//!
//! This module is the private CSHARP-03-T02-W06 seam.  It consumes only the
//! strict, context-bound `mpk.vir.v2` capability produced by W05.  It records
//! a closed ordinary-term encoding route and a later proof owner for every
//! admitted type, operation, check, and control form, but deliberately does
//! not discharge an obligation or invoke a checker.  Certificate v0 and all
//! predecessor VC/assembly profiles remain unchanged.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mpk_cert::encode::DeclarationKind;
use mpk_cert::{build_axiom_report, Certificate};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::Value;

use crate::csharp_practical_registry::{
    CSHARP_PRACTICAL_PROFILE, FOUNDATION_DESCRIPTOR_CONTENT_SHA256, FOUNDATION_DESCRIPTOR_ID,
    FOUNDATION_DESCRIPTOR_SCHEMA,
};
use crate::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, ArtifactRef, CapturedInputSet, PracticalArtifactContext,
    SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA, SUCCESSOR_VC_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use crate::csharp_practical_vir_model::{
    AbruptCompletion, ClosedOperationSignature, ClosedOperationTag, ControlNodeTag, PatternTag,
    RequiredCheckTag, TypedValueRef,
};
use crate::csharp_practical_vir_validation::{
    PracticalConstructionAction, PracticalVirFunction, ValidatedPracticalVir,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};

pub const CSHARP_PRACTICAL_VC_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VC-3.0");
pub const CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-PROGRAM-ASSEMBLY-2.0");
pub const CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE: &str =
    "mpk.program_certificate.ordinary_context.v2";
pub const CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE: &str = "mpk.csharp.foundation_expansion.v1";
pub const CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE: &str = "mpk.csharp.limits.v1";
pub const CERTIFICATE_V0_FORMAT: &str = mpk_cert::encode::CERT_FORMAT;

pub const ORDINARY_TERM_NODES_MAX: u64 = 262_144;
pub const GENERATED_DECLARATIONS_MAX: u64 = 8_192;
pub const BINDER_DEPTH_MAX: u64 = 256;
pub const STATIC_TRANSFORMERS_MAX: u64 = 16_384;

const VC_TRANSPORT_BYTES_MAX: u64 = 268_435_456;
const FOUNDATION_GROUP_ID: &str = "vc.group.0000.ordinary_foundation";
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Copy)]
pub struct PracticalVcSource<'a> {
    pub artifact_context: &'a PracticalArtifactContext,
    pub captured_inputs: &'a CapturedInputSet,
    pub vir: &'a ValidatedPracticalVir,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalFoundationLink {
    schema: String,
    id: String,
    content_sha256: String,
}

impl PracticalFoundationLink {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalArtifactLink {
    schema: String,
    sha256: String,
    canonical_bytes: u64,
}

impl PracticalArtifactLink {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub const fn canonical_bytes(&self) -> u64 {
        self.canonical_bytes
    }

    fn from_validated(reference: &ArtifactRef) -> Self {
        Self {
            schema: reference.schema().to_owned(),
            sha256: reference.sha256().to_owned(),
            canonical_bytes: reference.canonical_bytes(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum LaterProofOwner {
    #[serde(rename = "CSHARP-03-T06-W02")]
    ConstructionAndTypeInvariants,
    #[serde(rename = "CSHARP-03-T06-W03")]
    DataAndCollections,
    #[serde(rename = "CSHARP-03-T06-W04")]
    LoopSwitchAndPatterns,
    #[serde(rename = "CSHARP-03-T06-W05")]
    ExceptionalControl,
    #[serde(rename = "CSHARP-03-T06-W06")]
    BindingsAndSpecialization,
    #[serde(rename = "CSHARP-03-T06-W07")]
    BoundaryRoundTrip,
    #[serde(rename = "CSHARP-03-T06-W08")]
    PureTransition,
    #[serde(rename = "CSHARP-03-T06-W09")]
    OrdinaryFoundationAndAssembly,
}

impl LaterProofOwner {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConstructionAndTypeInvariants => "CSHARP-03-T06-W02",
            Self::DataAndCollections => "CSHARP-03-T06-W03",
            Self::LoopSwitchAndPatterns => "CSHARP-03-T06-W04",
            Self::ExceptionalControl => "CSHARP-03-T06-W05",
            Self::BindingsAndSpecialization => "CSHARP-03-T06-W06",
            Self::BoundaryRoundTrip => "CSHARP-03-T06-W07",
            Self::PureTransition => "CSHARP-03-T06-W08",
            Self::OrdinaryFoundationAndAssembly => "CSHARP-03-T06-W09",
        }
    }

    const fn order(self) -> u16 {
        match self {
            Self::ConstructionAndTypeInvariants => 200,
            Self::DataAndCollections => 300,
            Self::LoopSwitchAndPatterns => 400,
            Self::ExceptionalControl => 500,
            Self::BindingsAndSpecialization => 600,
            Self::BoundaryRoundTrip => 700,
            Self::PureTransition => 800,
            Self::OrdinaryFoundationAndAssembly => 900,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryTypeRoute {
    CheckedBoolLeaf,
    RegisteredBooleanCube,
    ApplicationBooleanCubeProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalTypeEncoding {
    type_id: String,
    route: OrdinaryTypeRoute,
    declaration_name: String,
    proof_owner: LaterProofOwner,
}

impl PracticalTypeEncoding {
    pub fn type_id(&self) -> &str {
        &self.type_id
    }

    pub const fn route(&self) -> OrdinaryTypeRoute {
        self.route
    }

    pub fn declaration_name(&self) -> &str {
        &self.declaration_name
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryOperationRoute {
    RegisteredFoundationEquation,
    FiniteBooleanCircuit,
    SourceBodyRelation,
    BindingProjectionRelation,
    BoundaryCodecRelation,
    ClosedExceptionRelation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryCheckRoute {
    BooleanPredicate,
    TaggedParseOutcome,
    ClosedExceptionalEdge,
    TaggedErrorOutcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalCheckEncoding {
    check_id: String,
    route: OrdinaryCheckRoute,
    failure_type_id: Option<String>,
    proof_owner: LaterProofOwner,
}

impl PracticalCheckEncoding {
    pub fn check_id(&self) -> &str {
        &self.check_id
    }

    pub const fn route(&self) -> OrdinaryCheckRoute {
        self.route
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalOperationEncoding {
    operation_id: String,
    operation_tag: ClosedOperationTag,
    route: OrdinaryOperationRoute,
    declaration_name: String,
    checks: Vec<PracticalCheckEncoding>,
    proof_owner: LaterProofOwner,
}

impl PracticalOperationEncoding {
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub const fn operation_tag(&self) -> ClosedOperationTag {
        self.operation_tag
    }

    pub const fn route(&self) -> OrdinaryOperationRoute {
        self.route
    }

    pub fn checks(&self) -> &[PracticalCheckEncoding] {
        &self.checks
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticalControlSubjectKind {
    Node,
    ConstructionAction,
    Loop,
    PatternDecision,
    PatternArm,
    ExceptionRegion,
    Catch,
    Finally,
    Unwind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryControlRoute {
    DirectFlow,
    BooleanDecision,
    OperationRelation,
    ConstructionState,
    LoopContract,
    PatternDecision,
    AbruptCompletion,
    ExceptionDispatch,
    FinallyComposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalControlEncoding {
    id: String,
    function_id: String,
    subject_kind: PracticalControlSubjectKind,
    source_tag: String,
    route: OrdinaryControlRoute,
    proof_owner: LaterProofOwner,
}

impl PracticalControlEncoding {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    pub const fn subject_kind(&self) -> PracticalControlSubjectKind {
        self.subject_kind
    }

    pub fn source_tag(&self) -> &str {
        &self.source_tag
    }

    pub const fn route(&self) -> OrdinaryControlRoute {
        self.route
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticalObligationGroupKind {
    OrdinaryFoundation,
    ConstructionAndTypeInvariant,
    DataAndCollection,
    LoopSwitchAndPattern,
    ExceptionalControl,
    BindingAndSpecialization,
    BoundaryRoundTrip,
    PureTransition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalObligationGroup {
    id: String,
    kind: PracticalObligationGroupKind,
    function_id: Option<String>,
    subject_ids: Vec<String>,
    dependencies: Vec<String>,
    proof_owner: LaterProofOwner,
}

impl PracticalObligationGroup {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn kind(&self) -> PracticalObligationGroupKind {
        self.kind
    }

    pub fn function_id(&self) -> Option<&str> {
        self.function_id.as_deref()
    }

    pub fn subject_ids(&self) -> &[String] {
        &self.subject_ids
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalOrdinaryTermLimits {
    ordinary_term_nodes_maximum: u64,
    generated_declarations_maximum: u64,
    binder_depth_maximum: u64,
    static_transformers_maximum: u64,
}

impl PracticalOrdinaryTermLimits {
    pub const fn ordinary_term_nodes_maximum(&self) -> u64 {
        self.ordinary_term_nodes_maximum
    }

    pub const fn generated_declarations_maximum(&self) -> u64 {
        self.generated_declarations_maximum
    }

    pub const fn binder_depth_maximum(&self) -> u64 {
        self.binder_depth_maximum
    }

    pub const fn static_transformers_maximum(&self) -> u64 {
        self.static_transformers_maximum
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalVcResourceReservation {
    ordinary_term_nodes_minimum: u64,
    generated_declarations_minimum: u64,
    binder_depth_minimum: u64,
    static_transformers_minimum: u64,
}

impl PracticalVcResourceReservation {
    pub const fn ordinary_term_nodes_minimum(&self) -> u64 {
        self.ordinary_term_nodes_minimum
    }

    pub const fn generated_declarations_minimum(&self) -> u64 {
        self.generated_declarations_minimum
    }

    pub const fn binder_depth_minimum(&self) -> u64 {
        self.binder_depth_minimum
    }

    pub const fn static_transformers_minimum(&self) -> u64 {
        self.static_transformers_minimum
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WirePracticalVcDocument {
    schema: String,
    semantic_context: Box<RawValue>,
    foundation_descriptor: PracticalFoundationLink,
    compilation_id: String,
    input_set_sha256: String,
    source_ir: PracticalArtifactLink,
    ordinary_encoding_profile: String,
    verification_limit_profile: String,
    ordinary_term_forms: Vec<String>,
    type_encodings: Vec<PracticalTypeEncoding>,
    operation_encodings: Vec<PracticalOperationEncoding>,
    control_encodings: Vec<PracticalControlEncoding>,
    obligation_groups: Vec<PracticalObligationGroup>,
    limits: PracticalOrdinaryTermLimits,
    resource_reservation: PracticalVcResourceReservation,
    vc_sha256: String,
}

pub struct ValidatedPracticalVc {
    wire: WirePracticalVcDocument,
    canonical_bytes: Vec<u8>,
    artifact_ref: ArtifactRef,
}

impl ValidatedPracticalVc {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &str {
        &self.wire.vc_sha256
    }

    pub fn artifact_ref(&self) -> ArtifactRef {
        self.artifact_ref.clone()
    }

    pub fn type_encodings(&self) -> &[PracticalTypeEncoding] {
        &self.wire.type_encodings
    }

    pub fn operation_encodings(&self) -> &[PracticalOperationEncoding] {
        &self.wire.operation_encodings
    }

    pub fn control_encodings(&self) -> &[PracticalControlEncoding] {
        &self.wire.control_encodings
    }

    pub fn obligation_groups(&self) -> &[PracticalObligationGroup] {
        &self.wire.obligation_groups
    }

    pub fn limits(&self) -> &PracticalOrdinaryTermLimits {
        &self.wire.limits
    }

    pub fn resource_reservation(&self) -> &PracticalVcResourceReservation {
        &self.wire.resource_reservation
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalTheoremSkeleton {
    name: String,
    group_id: String,
    group_kind: PracticalObligationGroupKind,
    function_id: Option<String>,
    subject_ids: Vec<String>,
    dependencies: Vec<String>,
    proof_owner: LaterProofOwner,
}

impl PracticalTheoremSkeleton {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn subject_ids(&self) -> &[String] {
        &self.subject_ids
    }

    pub const fn proof_owner(&self) -> LaterProofOwner {
        self.proof_owner
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WirePracticalVcSkeleton {
    schema: String,
    semantic_context: Box<RawValue>,
    foundation_descriptor: PracticalFoundationLink,
    source_ir: PracticalArtifactLink,
    source_vc: PracticalArtifactLink,
    ordinary_encoding_profile: String,
    verification_limit_profile: String,
    program_assembly_profile: String,
    theorem_declarations: Vec<PracticalTheoremSkeleton>,
    limits: PracticalOrdinaryTermLimits,
    resource_reservation: PracticalVcResourceReservation,
}

pub struct ValidatedPracticalVcSkeleton {
    wire: WirePracticalVcSkeleton,
    canonical_bytes: Vec<u8>,
    hash: String,
    artifact_ref: ArtifactRef,
}

impl ValidatedPracticalVcSkeleton {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn artifact_ref(&self) -> ArtifactRef {
        self.artifact_ref.clone()
    }

    pub fn theorem_declarations(&self) -> &[PracticalTheoremSkeleton] {
        &self.wire.theorem_declarations
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PracticalZeroAxiomReport {
    core_axiom_count: u64,
    builtin_theory_axiom_count: u64,
    go_semantics_axiom_count: u64,
    external_axiom_count: u64,
    total_axiom_count: u64,
    entries: Vec<Value>,
    declaration_dependencies: Vec<Value>,
}

impl PracticalZeroAxiomReport {
    pub const fn total_axiom_count(&self) -> u64 {
        self.total_axiom_count
    }

    pub fn entries(&self) -> &[Value] {
        &self.entries
    }

    pub fn declaration_dependencies(&self) -> &[Value] {
        &self.declaration_dependencies
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WirePracticalProgramAssemblyPlan {
    schema: String,
    semantic_context: Box<RawValue>,
    foundation_descriptor: PracticalFoundationLink,
    source_ir: PracticalArtifactLink,
    source_vc: PracticalArtifactLink,
    source_skeleton: PracticalArtifactLink,
    certificate_format: String,
    generated_declaration_kinds: Vec<String>,
    ordinary_term_forms: Vec<String>,
    imports: Vec<Value>,
    proof_node_table: Vec<Value>,
    theory_certificates: Vec<Value>,
    axiom_report: PracticalZeroAxiomReport,
    limits: PracticalOrdinaryTermLimits,
    assembly_sha256: String,
}

pub struct ValidatedPracticalProgramAssemblyPlan {
    wire: WirePracticalProgramAssemblyPlan,
    canonical_bytes: Vec<u8>,
}

impl ValidatedPracticalProgramAssemblyPlan {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &str {
        &self.wire.assembly_sha256
    }

    pub fn axiom_report(&self) -> &PracticalZeroAxiomReport {
        &self.wire.axiom_report
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalVcValidationPhase {
    Transport,
    Schema,
    Canonical,
    Context,
    Linkage,
    Encoding,
    Obligations,
    Limits,
    Hash,
    Assembly,
}

impl PracticalVcValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Schema => "schema",
            Self::Canonical => "canonical",
            Self::Context => "context",
            Self::Linkage => "linkage",
            Self::Encoding => "encoding",
            Self::Obligations => "obligations",
            Self::Limits => "limits",
            Self::Hash => "hash",
            Self::Assembly => "assembly",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalVcErrorCode {
    Json,
    Schema,
    Canonical,
    Context,
    Linkage,
    Encoding,
    Obligation,
    Limit,
    Hash,
    AssemblyProfile,
    CertificateStructure,
    AxiomReport,
}

impl PracticalVcErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "CSHARP_PRACTICAL_VC_JSON",
            Self::Schema => "CSHARP_PRACTICAL_VC_SCHEMA",
            Self::Canonical => "CSHARP_PRACTICAL_VC_CANONICAL",
            Self::Context => "CSHARP_PRACTICAL_VC_CONTEXT",
            Self::Linkage => "CSHARP_PRACTICAL_VC_LINKAGE",
            Self::Encoding => "CSHARP_PRACTICAL_VC_ENCODING",
            Self::Obligation => "CSHARP_PRACTICAL_VC_OBLIGATION",
            Self::Limit => "CSHARP_PRACTICAL_VC_LIMIT",
            Self::Hash => "CSHARP_PRACTICAL_VC_HASH",
            Self::AssemblyProfile => "CSHARP_PRACTICAL_ASSEMBLY_PROFILE",
            Self::CertificateStructure => "CSHARP_PRACTICAL_CERTIFICATE_STRUCTURE",
            Self::AxiomReport => "CSHARP_PRACTICAL_AXIOM_REPORT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PracticalVcError {
    phase: PracticalVcValidationPhase,
    code: PracticalVcErrorCode,
}

impl PracticalVcError {
    pub const fn phase(&self) -> PracticalVcValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> PracticalVcErrorCode {
        self.code
    }
}

impl fmt::Display for PracticalVcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at practical VC phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for PracticalVcError {}

const fn failure(
    phase: PracticalVcValidationPhase,
    code: PracticalVcErrorCode,
) -> PracticalVcError {
    PracticalVcError { phase, code }
}

/// Builds the only W06 VC document for one already validated practical VIR.
/// The returned bytes are immediately re-imported through the same strict
/// boundary used by later producers.
pub fn generate_csharp_practical_vc(
    source: PracticalVcSource<'_>,
) -> Result<ValidatedPracticalVc, PracticalVcError> {
    let wire = expected_vc(source)?;
    let bytes = encode_bounded(&wire)?;
    import_csharp_practical_vc_json(&bytes, source)
}

/// Strictly imports a canonical `mpk.vc.v3` document and reconstructs every
/// encoding, group, resource reservation, linkage field, and hash from W05.
pub fn import_csharp_practical_vc_json(
    input: &[u8],
    source: PracticalVcSource<'_>,
) -> Result<ValidatedPracticalVc, PracticalVcError> {
    validate_source(source)?;
    require_transport_bound(input)?;
    require_schema(input, SUCCESSOR_VC_SCHEMA)?;
    let wire: WirePracticalVcDocument = decode_wire(input)?;
    let reencoded = encode_bounded(&wire)?;
    if reencoded != input {
        return Err(failure(
            PracticalVcValidationPhase::Canonical,
            PracticalVcErrorCode::Canonical,
        ));
    }

    let expected = expected_vc(source)?;
    validate_common_linkage(
        &wire.semantic_context,
        &wire.foundation_descriptor,
        &wire.source_ir,
        source,
    )?;
    if wire.compilation_id != source.artifact_context.compilation_id()
        || wire.input_set_sha256 != source.captured_inputs.snapshot_sha256()
        || wire.ordinary_encoding_profile != CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE
        || wire.verification_limit_profile != CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE
    {
        return Err(failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        ));
    }
    if wire.ordinary_term_forms != expected.ordinary_term_forms
        || wire.type_encodings != expected.type_encodings
        || wire.operation_encodings != expected.operation_encodings
        || wire.control_encodings != expected.control_encodings
    {
        return Err(failure(
            PracticalVcValidationPhase::Encoding,
            PracticalVcErrorCode::Encoding,
        ));
    }
    if wire.obligation_groups != expected.obligation_groups {
        return Err(failure(
            PracticalVcValidationPhase::Obligations,
            PracticalVcErrorCode::Obligation,
        ));
    }
    if wire.limits != exact_limits() || wire.resource_reservation != expected.resource_reservation {
        return Err(failure(
            PracticalVcValidationPhase::Limits,
            PracticalVcErrorCode::Limit,
        ));
    }
    let recomputed = vc_hash(&wire)?;
    if !valid_sha256(&wire.vc_sha256) || wire.vc_sha256 != recomputed {
        return Err(failure(
            PracticalVcValidationPhase::Hash,
            PracticalVcErrorCode::Hash,
        ));
    }
    let artifact_ref = ArtifactRef::validated_successor(
        source.artifact_context,
        source.captured_inputs,
        SUCCESSOR_VC_SCHEMA,
        &wire.vc_sha256,
        u64::try_from(input.len()).unwrap_or(u64::MAX),
    )
    .map_err(|_| {
        failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        )
    })?;
    Ok(ValidatedPracticalVc {
        wire,
        canonical_bytes: input.to_vec(),
        artifact_ref,
    })
}

fn expected_vc(source: PracticalVcSource<'_>) -> Result<WirePracticalVcDocument, PracticalVcError> {
    validate_source(source)?;
    let semantic_context = canonical_context_raw(source.artifact_context)?;
    let foundation_descriptor = expected_foundation(source.artifact_context)?;
    let source_ir_ref = source.vir.artifact_ref();
    let source_ir = PracticalArtifactLink::from_validated(&source_ir_ref);
    let type_encodings = build_type_encodings(source.vir)?;
    let operation_encodings = build_operation_encodings(source.vir.operation_signatures())?;
    let control_encodings = build_control_encodings(source.vir.functions());
    let obligation_groups = build_obligation_groups(
        source.vir,
        &type_encodings,
        &operation_encodings,
        &control_encodings,
    );
    let resource_reservation = resource_reservation(
        source.vir.functions(),
        &type_encodings,
        &operation_encodings,
        &control_encodings,
        &obligation_groups,
    )?;
    validate_resource_reservation(&resource_reservation)?;
    let mut wire = WirePracticalVcDocument {
        schema: SUCCESSOR_VC_SCHEMA.to_owned(),
        semantic_context,
        foundation_descriptor,
        compilation_id: source.artifact_context.compilation_id().to_owned(),
        input_set_sha256: source.captured_inputs.snapshot_sha256().to_owned(),
        source_ir,
        ordinary_encoding_profile: CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE.to_owned(),
        verification_limit_profile: CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE.to_owned(),
        ordinary_term_forms: ordinary_term_forms(),
        type_encodings,
        operation_encodings,
        control_encodings,
        obligation_groups,
        limits: exact_limits(),
        resource_reservation,
        vc_sha256: ZERO_SHA256.to_owned(),
    };
    wire.vc_sha256 = vc_hash(&wire)?;
    Ok(wire)
}

fn build_type_encodings(
    vir: &ValidatedPracticalVir,
) -> Result<Vec<PracticalTypeEncoding>, PracticalVcError> {
    let mut type_ids = BTreeSet::new();
    for operation in vir.operation_signatures() {
        for type_id in operation
            .argument_type_ids
            .iter()
            .chain(std::iter::once(&operation.normal_result_type_id))
        {
            insert_type_id(&mut type_ids, type_id);
        }
        for check in &operation.ordered_checks {
            if let Some(type_id) = &check.failure_type_id {
                insert_type_id(&mut type_ids, type_id);
            }
        }
    }
    for function in vir.functions() {
        collect_function_type_ids(function, &mut type_ids);
    }
    for exception in vir.source_exceptions() {
        insert_type_id(&mut type_ids, &exception.type_id);
    }
    for projection in vir.binding_projections() {
        insert_type_id(&mut type_ids, &projection.source_type_id);
        insert_type_id(&mut type_ids, &projection.semantic_type_id);
    }
    for commutation in vir.binding_commutations() {
        for operation in [
            &commutation.source_operation,
            &commutation.semantic_operation,
        ] {
            for type_id in operation
                .argument_type_ids
                .iter()
                .chain(std::iter::once(&operation.normal_result_type_id))
            {
                insert_type_id(&mut type_ids, type_id);
            }
        }
    }

    type_ids
        .into_iter()
        .map(|type_id| {
            let route = ordinary_type_route(&type_id)?;
            Ok(PracticalTypeEncoding {
                declaration_name: format!("Mpk.CSharp.Practical.Type.{type_id}"),
                type_id,
                route,
                proof_owner: LaterProofOwner::OrdinaryFoundationAndAssembly,
            })
        })
        .collect()
}

fn insert_type_id(type_ids: &mut BTreeSet<String>, type_id: &str) {
    if type_id.starts_with("mpk.csharp.") {
        type_ids.insert(type_id.to_owned());
    }
}

fn collect_function_type_ids(function: &PracticalVirFunction, output: &mut BTreeSet<String>) {
    for value in &function.parameter_values {
        insert_typed_value(output, value);
    }
    for type_id in &function.result_type_ids {
        insert_type_id(output, type_id);
    }
    for block in &function.blocks {
        if let Some(type_id) = &block.node.condition_type_id {
            insert_type_id(output, type_id);
        }
        if let Some(value) = &block.handler_exception_value {
            insert_typed_value(output, value);
        }
        if let Some(invocation) = &block.invocation {
            for value in &invocation.operands {
                insert_typed_value(output, value);
            }
            insert_typed_value(output, &invocation.result);
        }
        for phi in &block.phi_values {
            insert_typed_value(output, &phi.value);
        }
        for state in block.ownership_in.iter().chain(&block.ownership_out) {
            insert_type_id(output, &state.instance_id);
            insert_type_id(output, &state.element_type_id);
            insert_type_id(output, &state.published_type_id);
        }
        for action in &block.construction_actions {
            match action {
                PracticalConstructionAction::Allocate { instance_id, .. } => {
                    insert_type_id(output, instance_id);
                }
                PracticalConstructionAction::Read { result, .. }
                | PracticalConstructionAction::Freeze { result, .. } => {
                    insert_typed_value(output, result);
                }
                PracticalConstructionAction::Fill { value, .. }
                | PracticalConstructionAction::Rewrite { value, .. } => {
                    insert_typed_value(output, value);
                }
                PracticalConstructionAction::Borrow { .. }
                | PracticalConstructionAction::EndBorrow { .. }
                | PracticalConstructionAction::Transfer { .. }
                | PracticalConstructionAction::Discard { .. } => {}
            }
        }
        if let Some(abrupt) = &block.node.abrupt {
            match abrupt {
                AbruptCompletion::Return {
                    value_type_id: Some(type_id),
                }
                | AbruptCompletion::Throw {
                    exception_type_id: type_id,
                    ..
                } => insert_type_id(output, type_id),
                AbruptCompletion::Normal
                | AbruptCompletion::Return {
                    value_type_id: None,
                }
                | AbruptCompletion::Break { .. }
                | AbruptCompletion::Continue { .. } => {}
            }
        }
    }
    for pattern in &function.patterns {
        insert_type_id(output, &pattern.governing_type_id);
        for arm in &pattern.arms {
            if let Some(type_id) = &arm.guard_type_id {
                insert_type_id(output, type_id);
            }
            for type_id in &arm.bound_parameter_type_ids {
                insert_type_id(output, type_id);
            }
        }
    }
    for region in &function.exception_regions {
        for catch in &region.catches {
            if let Some(filter) = &catch.filter {
                insert_type_id(output, &filter.condition_type_id);
            }
        }
    }
}

fn insert_typed_value(output: &mut BTreeSet<String>, value: &TypedValueRef) {
    insert_type_id(output, &value.type_id);
}

/// Closed type-ID routing table. Application types are admitted only through
/// the explicit projection route; every other accepted type is a registered,
/// monomorphic Boolean-cube carrier (with Bool exposed as the checked leaf).
pub fn ordinary_type_route(type_id: &str) -> Result<OrdinaryTypeRoute, PracticalVcError> {
    if type_id == "mpk.csharp.value.bool.v1" {
        Ok(OrdinaryTypeRoute::CheckedBoolLeaf)
    } else if type_id.starts_with("mpk.csharp.value.")
        || type_id.starts_with("mpk.csharp.instance.")
    {
        Ok(OrdinaryTypeRoute::RegisteredBooleanCube)
    } else if type_id.starts_with("mpk.csharp.source.") {
        Ok(OrdinaryTypeRoute::ApplicationBooleanCubeProjection)
    } else {
        Err(failure(
            PracticalVcValidationPhase::Encoding,
            PracticalVcErrorCode::Encoding,
        ))
    }
}

fn build_operation_encodings(
    operations: &[ClosedOperationSignature],
) -> Result<Vec<PracticalOperationEncoding>, PracticalVcError> {
    operations
        .iter()
        .map(|operation| {
            let (route, proof_owner) = ordinary_operation_route(operation.tag);
            Ok(PracticalOperationEncoding {
                operation_id: operation.id.clone(),
                operation_tag: operation.tag,
                route,
                declaration_name: format!("Mpk.CSharp.Practical.Operation.{}", operation.id),
                checks: operation
                    .ordered_checks
                    .iter()
                    .map(|check| {
                        let (route, proof_owner) = ordinary_check_route(&check.id, check.tag)?;
                        Ok(PracticalCheckEncoding {
                            check_id: check.id.clone(),
                            route,
                            failure_type_id: check.failure_type_id.clone(),
                            proof_owner,
                        })
                    })
                    .collect::<Result<Vec<_>, PracticalVcError>>()?,
                proof_owner,
            })
        })
        .collect()
}

/// Closed operation-tag routing table.  There is deliberately no intrinsic,
/// theory, runtime-call, or host-evaluation route.
pub const fn ordinary_operation_route(
    tag: ClosedOperationTag,
) -> (OrdinaryOperationRoute, LaterProofOwner) {
    match tag {
        ClosedOperationTag::Foundation => (
            OrdinaryOperationRoute::RegisteredFoundationEquation,
            LaterProofOwner::OrdinaryFoundationAndAssembly,
        ),
        ClosedOperationTag::SourceCall => (
            OrdinaryOperationRoute::SourceBodyRelation,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        ClosedOperationTag::BindingProject | ClosedOperationTag::BindingReconstruct => (
            OrdinaryOperationRoute::BindingProjectionRelation,
            LaterProofOwner::BindingsAndSpecialization,
        ),
        ClosedOperationTag::BoundaryParse | ClosedOperationTag::BoundaryFormat => (
            OrdinaryOperationRoute::BoundaryCodecRelation,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        ClosedOperationTag::ExceptionConstruct
        | ClosedOperationTag::ExceptionIsType
        | ClosedOperationTag::ExceptionPayload => (
            OrdinaryOperationRoute::ClosedExceptionRelation,
            LaterProofOwner::ExceptionalControl,
        ),
        ClosedOperationTag::FieldRead
        | ClosedOperationTag::ValueConstruct
        | ClosedOperationTag::StructuralEqual
        | ClosedOperationTag::CanonicalCompare
        | ClosedOperationTag::Data => (
            OrdinaryOperationRoute::FiniteBooleanCircuit,
            LaterProofOwner::DataAndCollections,
        ),
    }
}

/// Closed required-check routing table. The ID/tag pair is repeated here so a
/// broad tag cannot silently assign a construction or transition obligation
/// to the data owner. W03 has already validated the same pair at the VIR
/// boundary; this independent match protects the VC ownership contract.
pub fn ordinary_check_route(
    check_id: &str,
    tag: RequiredCheckTag,
) -> Result<(OrdinaryCheckRoute, LaterProofOwner), PracticalVcError> {
    let (expected_tag, route, owner) = match check_id {
        "already_initialized"
        | "construction_bound"
        | "incomplete"
        | "ownership"
        | "publication_bound"
        | "uninitialized" => (
            RequiredCheckTag::StaticObligation,
            OrdinaryCheckRoute::BooleanPredicate,
            LaterProofOwner::ConstructionAndTypeInvariants,
        ),
        "invalid_representation" | "obligation.output_bound" => (
            RequiredCheckTag::StaticObligation,
            OrdinaryCheckRoute::BooleanPredicate,
            LaterProofOwner::DataAndCollections,
        ),
        "parse_error.input_bound"
        | "parse_error.syntax"
        | "parse_error.noncanonical"
        | "parse_error.scale_precision"
        | "parse_error.range" => (
            RequiredCheckTag::ParseError,
            OrdinaryCheckRoute::TaggedParseOutcome,
            LaterProofOwner::BoundaryRoundTrip,
        ),
        "negative_length"
        | "exception.overflow"
        | "index_range"
        | "invalid_operation"
        | "exception.division_by_zero"
        | "exception.range"
        | "exception.null_receiver"
        | "exception.null_argument" => (
            RequiredCheckTag::Exception,
            OrdinaryCheckRoute::ClosedExceptionalEdge,
            LaterProofOwner::ExceptionalControl,
        ),
        "event_bound" => (
            RequiredCheckTag::ErrorOutcome,
            OrdinaryCheckRoute::TaggedErrorOutcome,
            LaterProofOwner::PureTransition,
        ),
        "capacity" | "currency_mismatch" | "decimal_overflow" | "division_by_zero"
        | "duplicate_element" | "duplicate_key" | "empty_errors" | "invalid_currency"
        | "invalid_precision" | "invalid_rounding" | "invalid_scale" | "missing_key"
        | "precision" | "range" | "validation_bound" => (
            RequiredCheckTag::ErrorOutcome,
            OrdinaryCheckRoute::TaggedErrorOutcome,
            LaterProofOwner::DataAndCollections,
        ),
        _ => {
            return Err(failure(
                PracticalVcValidationPhase::Encoding,
                PracticalVcErrorCode::Encoding,
            ));
        }
    };
    if tag != expected_tag {
        return Err(failure(
            PracticalVcValidationPhase::Encoding,
            PracticalVcErrorCode::Encoding,
        ));
    }
    Ok((route, owner))
}

fn build_control_encodings(functions: &[PracticalVirFunction]) -> Vec<PracticalControlEncoding> {
    let mut encodings = Vec::new();
    for function in functions {
        for block in &function.blocks {
            let (route, proof_owner) = ordinary_control_route(block.node.tag);
            encodings.push(PracticalControlEncoding {
                id: format!(
                    "{}#node:{:010}:{}",
                    function.id, block.node.ordinal, block.node.id
                ),
                function_id: function.id.clone(),
                subject_kind: PracticalControlSubjectKind::Node,
                source_tag: block.node.tag.as_str().to_owned(),
                route,
                proof_owner,
            });
            for (ordinal, action) in block.construction_actions.iter().enumerate() {
                encodings.push(PracticalControlEncoding {
                    id: format!(
                        "{}#action:{:010}:{ordinal:010}",
                        function.id, block.node.ordinal
                    ),
                    function_id: function.id.clone(),
                    subject_kind: PracticalControlSubjectKind::ConstructionAction,
                    source_tag: construction_action_tag(action).to_owned(),
                    route: OrdinaryControlRoute::ConstructionState,
                    proof_owner: LaterProofOwner::ConstructionAndTypeInvariants,
                });
            }
        }
        for loop_region in &function.loops {
            encodings.push(PracticalControlEncoding {
                id: format!("{}#loop:{}", function.id, loop_region.id),
                function_id: function.id.clone(),
                subject_kind: PracticalControlSubjectKind::Loop,
                source_tag: "loop".to_owned(),
                route: OrdinaryControlRoute::LoopContract,
                proof_owner: LaterProofOwner::LoopSwitchAndPatterns,
            });
        }
        for pattern in &function.patterns {
            encodings.push(PracticalControlEncoding {
                id: format!("{}#pattern:{}", function.id, pattern.node_id),
                function_id: function.id.clone(),
                subject_kind: PracticalControlSubjectKind::PatternDecision,
                source_tag: "pattern_decision".to_owned(),
                route: OrdinaryControlRoute::PatternDecision,
                proof_owner: LaterProofOwner::LoopSwitchAndPatterns,
            });
            for arm in &pattern.arms {
                encodings.push(PracticalControlEncoding {
                    id: format!(
                        "{}#pattern:{}:arm:{:010}",
                        function.id, pattern.node_id, arm.ordinal
                    ),
                    function_id: function.id.clone(),
                    subject_kind: PracticalControlSubjectKind::PatternArm,
                    source_tag: arm.tag.as_str().to_owned(),
                    route: OrdinaryControlRoute::PatternDecision,
                    proof_owner: LaterProofOwner::LoopSwitchAndPatterns,
                });
            }
        }
        for region in &function.exception_regions {
            encodings.push(PracticalControlEncoding {
                id: format!("{}#exception_region:{}", function.id, region.id),
                function_id: function.id.clone(),
                subject_kind: PracticalControlSubjectKind::ExceptionRegion,
                source_tag: "exception_region".to_owned(),
                route: OrdinaryControlRoute::ExceptionDispatch,
                proof_owner: LaterProofOwner::ExceptionalControl,
            });
            for catch in &region.catches {
                encodings.push(PracticalControlEncoding {
                    id: format!(
                        "{}#exception_region:{}:catch:{:010}",
                        function.id, region.id, catch.ordinal
                    ),
                    function_id: function.id.clone(),
                    subject_kind: PracticalControlSubjectKind::Catch,
                    source_tag: "catch".to_owned(),
                    route: OrdinaryControlRoute::ExceptionDispatch,
                    proof_owner: LaterProofOwner::ExceptionalControl,
                });
            }
            if region.finally_entry_node_id.is_some() {
                encodings.push(PracticalControlEncoding {
                    id: format!("{}#exception_region:{}:finally", function.id, region.id),
                    function_id: function.id.clone(),
                    subject_kind: PracticalControlSubjectKind::Finally,
                    source_tag: "finally".to_owned(),
                    route: OrdinaryControlRoute::FinallyComposition,
                    proof_owner: LaterProofOwner::ExceptionalControl,
                });
            }
        }
        for unwind in &function.unwind_plans {
            encodings.push(PracticalControlEncoding {
                id: format!(
                    "{}#unwind:{}:{}:{}",
                    function.id, unwind.source_node_id, unwind.check_id, unwind.destination_node_id
                ),
                function_id: function.id.clone(),
                subject_kind: PracticalControlSubjectKind::Unwind,
                source_tag: "unwind".to_owned(),
                route: OrdinaryControlRoute::ExceptionDispatch,
                proof_owner: LaterProofOwner::ExceptionalControl,
            });
        }
    }
    encodings.sort_by(|left, right| left.id.cmp(&right.id));
    encodings
}

/// Closed control-node routing table.  Adding a W03 control variant cannot
/// compile without selecting one ordinary route and one later proof owner.
pub const fn ordinary_control_route(
    tag: ControlNodeTag,
) -> (OrdinaryControlRoute, LaterProofOwner) {
    match tag {
        ControlNodeTag::Entry | ControlNodeTag::Jump | ControlNodeTag::Exit => (
            OrdinaryControlRoute::DirectFlow,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
        ControlNodeTag::Operation => (
            OrdinaryControlRoute::OperationRelation,
            LaterProofOwner::DataAndCollections,
        ),
        ControlNodeTag::Branch => (
            OrdinaryControlRoute::BooleanDecision,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
        ControlNodeTag::LoopHeader => (
            OrdinaryControlRoute::LoopContract,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
        ControlNodeTag::PatternDecision => (
            OrdinaryControlRoute::PatternDecision,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
        ControlNodeTag::Return | ControlNodeTag::Break | ControlNodeTag::Continue => (
            OrdinaryControlRoute::AbruptCompletion,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
        ControlNodeTag::Throw | ControlNodeTag::Rethrow | ControlNodeTag::HandlerEntry => (
            OrdinaryControlRoute::ExceptionDispatch,
            LaterProofOwner::ExceptionalControl,
        ),
        ControlNodeTag::FinallyEntry | ControlNodeTag::FinallyExit => (
            OrdinaryControlRoute::FinallyComposition,
            LaterProofOwner::ExceptionalControl,
        ),
    }
}

/// Every pattern is compiled into the one finite, ordered Boolean decision
/// route; source pattern spelling never selects an intrinsic.
pub const fn ordinary_pattern_route(tag: PatternTag) -> (OrdinaryControlRoute, LaterProofOwner) {
    match tag {
        PatternTag::Constant
        | PatternTag::Discard
        | PatternTag::Var
        | PatternTag::Null
        | PatternTag::NotNull
        | PatternTag::Relational
        | PatternTag::Parenthesized
        | PatternTag::And
        | PatternTag::Or
        | PatternTag::Not
        | PatternTag::DeclarationType
        | PatternTag::ExactTag
        | PatternTag::Property
        | PatternTag::List => (
            OrdinaryControlRoute::PatternDecision,
            LaterProofOwner::LoopSwitchAndPatterns,
        ),
    }
}

fn construction_action_tag(action: &PracticalConstructionAction) -> &'static str {
    match action {
        PracticalConstructionAction::Allocate { .. } => "allocate",
        PracticalConstructionAction::Read { .. } => "read",
        PracticalConstructionAction::Fill { .. } => "fill",
        PracticalConstructionAction::Rewrite { .. } => "rewrite",
        PracticalConstructionAction::Borrow { .. } => "borrow",
        PracticalConstructionAction::EndBorrow { .. } => "end_borrow",
        PracticalConstructionAction::Transfer { .. } => "transfer",
        PracticalConstructionAction::Freeze { .. } => "freeze",
        PracticalConstructionAction::Discard { .. } => "discard",
    }
}

fn build_obligation_groups(
    vir: &ValidatedPracticalVir,
    types: &[PracticalTypeEncoding],
    operations: &[PracticalOperationEncoding],
    controls: &[PracticalControlEncoding],
) -> Vec<PracticalObligationGroup> {
    let mut foundation_subjects = types
        .iter()
        .map(|encoding| format!("type:{}", encoding.type_id))
        .collect::<BTreeSet<_>>();
    let mut pending: BTreeMap<(Option<String>, LaterProofOwner), BTreeSet<String>> =
        BTreeMap::new();
    for operation in operations {
        let subject = format!("operation:{}", operation.operation_id);
        if operation.operation_tag == ClosedOperationTag::Foundation {
            foundation_subjects.insert(subject);
        } else {
            pending
                .entry((None, operation.proof_owner))
                .or_default()
                .insert(subject);
        }
        for check in &operation.checks {
            pending
                .entry((None, check.proof_owner))
                .or_default()
                .insert(format!(
                    "check:{}:{}",
                    operation.operation_id, check.check_id
                ));
        }
    }
    for projection in vir.binding_projections() {
        pending
            .entry((None, LaterProofOwner::BindingsAndSpecialization))
            .or_default()
            .insert(format!("binding_projection:{}", projection.id));
    }
    for commutation in vir.binding_commutations() {
        pending
            .entry((None, LaterProofOwner::BindingsAndSpecialization))
            .or_default()
            .insert(format!(
                "binding_commutation:{}:{}:{}",
                commutation.binding_id,
                commutation.source_operation.id,
                commutation.semantic_operation.id
            ));
    }
    for exception in vir.source_exceptions() {
        pending
            .entry((None, LaterProofOwner::ExceptionalControl))
            .or_default()
            .insert(format!("source_exception:{}", exception.type_id));
    }
    for control in controls {
        pending
            .entry((Some(control.function_id.clone()), control.proof_owner))
            .or_default()
            .insert(format!("control:{}", control.id));
    }

    let mut groups = vec![PracticalObligationGroup {
        id: FOUNDATION_GROUP_ID.to_owned(),
        kind: PracticalObligationGroupKind::OrdinaryFoundation,
        function_id: None,
        subject_ids: foundation_subjects.into_iter().collect(),
        dependencies: Vec::new(),
        proof_owner: LaterProofOwner::OrdinaryFoundationAndAssembly,
    }];
    for ((function_id, proof_owner), subjects) in pending {
        let scope = match &function_id {
            Some(function_id) => format!("function.{function_id}"),
            None => "global".to_owned(),
        };
        groups.push(PracticalObligationGroup {
            id: format!("vc.group.{:04}.{scope}", proof_owner.order()),
            kind: group_kind(proof_owner),
            function_id,
            subject_ids: subjects.into_iter().collect(),
            dependencies: vec![FOUNDATION_GROUP_ID.to_owned()],
            proof_owner,
        });
    }
    groups.sort_by(|left, right| left.id.cmp(&right.id));
    groups
}

const fn group_kind(owner: LaterProofOwner) -> PracticalObligationGroupKind {
    match owner {
        LaterProofOwner::ConstructionAndTypeInvariants => {
            PracticalObligationGroupKind::ConstructionAndTypeInvariant
        }
        LaterProofOwner::DataAndCollections => PracticalObligationGroupKind::DataAndCollection,
        LaterProofOwner::LoopSwitchAndPatterns => {
            PracticalObligationGroupKind::LoopSwitchAndPattern
        }
        LaterProofOwner::ExceptionalControl => PracticalObligationGroupKind::ExceptionalControl,
        LaterProofOwner::BindingsAndSpecialization => {
            PracticalObligationGroupKind::BindingAndSpecialization
        }
        LaterProofOwner::BoundaryRoundTrip => PracticalObligationGroupKind::BoundaryRoundTrip,
        LaterProofOwner::PureTransition => PracticalObligationGroupKind::PureTransition,
        LaterProofOwner::OrdinaryFoundationAndAssembly => {
            PracticalObligationGroupKind::OrdinaryFoundation
        }
    }
}

fn resource_reservation(
    functions: &[PracticalVirFunction],
    types: &[PracticalTypeEncoding],
    operations: &[PracticalOperationEncoding],
    controls: &[PracticalControlEncoding],
    groups: &[PracticalObligationGroup],
) -> Result<PracticalVcResourceReservation, PracticalVcError> {
    let generated_declarations_minimum =
        checked_sum([types.len(), operations.len(), groups.len()])?;
    let subject_count = groups.iter().try_fold(0_u64, |count, group| {
        count
            .checked_add(usize_to_u64(group.subject_ids.len())?)
            .ok_or_else(limit_error)
    })?;
    let ordinary_term_nodes_minimum =
        checked_sum([types.len(), operations.len(), controls.len(), groups.len()])?
            .checked_add(subject_count)
            .ok_or_else(limit_error)?;
    let binder_depth_minimum = functions
        .iter()
        .flat_map(|function| &function.blocks)
        .map(|block| {
            usize_to_u64(block.node.region_stack.len()).map(|depth| depth.saturating_add(1))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .unwrap_or(0);
    let static_transformers_minimum = functions
        .iter()
        .flat_map(|function| &function.blocks)
        .flat_map(|block| &block.construction_actions)
        .try_fold(0_u64, |count, action| match action {
            PracticalConstructionAction::Allocate {
                publication_length_maximum,
                ..
            } => count
                .checked_add(u64::from(*publication_length_maximum))
                .ok_or_else(limit_error),
            PracticalConstructionAction::Read { .. }
            | PracticalConstructionAction::Fill { .. }
            | PracticalConstructionAction::Rewrite { .. }
            | PracticalConstructionAction::Borrow { .. }
            | PracticalConstructionAction::EndBorrow { .. }
            | PracticalConstructionAction::Transfer { .. }
            | PracticalConstructionAction::Freeze { .. }
            | PracticalConstructionAction::Discard { .. } => Ok(count),
        })?;
    Ok(PracticalVcResourceReservation {
        ordinary_term_nodes_minimum,
        generated_declarations_minimum,
        binder_depth_minimum,
        static_transformers_minimum,
    })
}

fn checked_sum<const N: usize>(counts: [usize; N]) -> Result<u64, PracticalVcError> {
    counts.into_iter().try_fold(0_u64, |total, count| {
        total
            .checked_add(usize_to_u64(count)?)
            .ok_or_else(limit_error)
    })
}

fn usize_to_u64(value: usize) -> Result<u64, PracticalVcError> {
    u64::try_from(value).map_err(|_| limit_error())
}

const fn limit_error() -> PracticalVcError {
    failure(
        PracticalVcValidationPhase::Limits,
        PracticalVcErrorCode::Limit,
    )
}

fn validate_resource_reservation(
    reservation: &PracticalVcResourceReservation,
) -> Result<(), PracticalVcError> {
    if reservation.ordinary_term_nodes_minimum > ORDINARY_TERM_NODES_MAX
        || reservation.generated_declarations_minimum > GENERATED_DECLARATIONS_MAX
        || reservation.binder_depth_minimum > BINDER_DEPTH_MAX
        || reservation.static_transformers_minimum > STATIC_TRANSFORMERS_MAX
    {
        Err(limit_error())
    } else {
        Ok(())
    }
}

const fn exact_limits() -> PracticalOrdinaryTermLimits {
    PracticalOrdinaryTermLimits {
        ordinary_term_nodes_maximum: ORDINARY_TERM_NODES_MAX,
        generated_declarations_maximum: GENERATED_DECLARATIONS_MAX,
        binder_depth_maximum: BINDER_DEPTH_MAX,
        static_transformers_maximum: STATIC_TRANSFORMERS_MAX,
    }
}

fn ordinary_term_forms() -> Vec<String> {
    ["sort", "var", "const", "app", "lam", "pi", "let"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn generated_declaration_kinds() -> Vec<String> {
    ["def", "theorem"].into_iter().map(str::to_owned).collect()
}

/// Emits the deterministic theorem-interface skeleton for one validated VC.
/// It contains only declaration plans and owners; no proof is manufactured.
pub fn emit_csharp_practical_vc_skeleton(
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
) -> Result<ValidatedPracticalVcSkeleton, PracticalVcError> {
    let wire = expected_skeleton(source, vc)?;
    let bytes = encode_bounded(&wire)?;
    import_csharp_practical_vc_skeleton_json(&bytes, source, vc)
}

pub fn import_csharp_practical_vc_skeleton_json(
    input: &[u8],
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
) -> Result<ValidatedPracticalVcSkeleton, PracticalVcError> {
    validate_source(source)?;
    validate_vc_lineage(source, vc)?;
    require_transport_bound(input)?;
    require_schema(input, SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA)?;
    let wire: WirePracticalVcSkeleton = decode_wire(input)?;
    let reencoded = encode_bounded(&wire)?;
    if reencoded != input {
        return Err(failure(
            PracticalVcValidationPhase::Canonical,
            PracticalVcErrorCode::Canonical,
        ));
    }
    validate_common_linkage(
        &wire.semantic_context,
        &wire.foundation_descriptor,
        &wire.source_ir,
        source,
    )?;
    let expected = expected_skeleton(source, vc)?;
    if wire.source_vc != expected.source_vc
        || wire.ordinary_encoding_profile != CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE
        || wire.verification_limit_profile != CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE
        || wire.program_assembly_profile != CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE
    {
        return Err(failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        ));
    }
    if wire.theorem_declarations != expected.theorem_declarations {
        return Err(failure(
            PracticalVcValidationPhase::Obligations,
            PracticalVcErrorCode::Obligation,
        ));
    }
    if wire.limits != exact_limits() || wire.resource_reservation != expected.resource_reservation {
        return Err(limit_error());
    }
    let hash = hash_complete(CSHARP_PRACTICAL_VC_HASH_DOMAIN, input)?;
    let artifact_ref = ArtifactRef::validated_successor(
        source.artifact_context,
        source.captured_inputs,
        SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA,
        &hash,
        u64::try_from(input.len()).unwrap_or(u64::MAX),
    )
    .map_err(|_| {
        failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        )
    })?;
    Ok(ValidatedPracticalVcSkeleton {
        wire,
        canonical_bytes: input.to_vec(),
        hash,
        artifact_ref,
    })
}

fn expected_skeleton(
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
) -> Result<WirePracticalVcSkeleton, PracticalVcError> {
    validate_source(source)?;
    validate_vc_lineage(source, vc)?;
    let declarations = vc
        .obligation_groups()
        .iter()
        .map(|group| PracticalTheoremSkeleton {
            name: format!("Mpk.CSharp.Practical.Obligation.{}", group.id),
            group_id: group.id.clone(),
            group_kind: group.kind,
            function_id: group.function_id.clone(),
            subject_ids: group.subject_ids.clone(),
            dependencies: group.dependencies.clone(),
            proof_owner: group.proof_owner,
        })
        .collect();
    Ok(WirePracticalVcSkeleton {
        schema: SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA.to_owned(),
        semantic_context: canonical_context_raw(source.artifact_context)?,
        foundation_descriptor: expected_foundation(source.artifact_context)?,
        source_ir: PracticalArtifactLink::from_validated(&source.vir.artifact_ref()),
        source_vc: PracticalArtifactLink::from_validated(&vc.artifact_ref()),
        ordinary_encoding_profile: CSHARP_PRACTICAL_ORDINARY_ENCODING_PROFILE.to_owned(),
        verification_limit_profile: CSHARP_PRACTICAL_VERIFICATION_LIMIT_PROFILE.to_owned(),
        program_assembly_profile: CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE.to_owned(),
        theorem_declarations: declarations,
        limits: exact_limits(),
        resource_reservation: vc.resource_reservation().clone(),
    })
}

/// Builds the context-specific ordinary-context assembly plan.  The plan is
/// intentionally proof-empty at T02; later assembly must satisfy this exact
/// profile and then pass [`validate_csharp_practical_certificate_structure`].
pub fn generate_csharp_practical_program_assembly_plan(
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
    skeleton: &ValidatedPracticalVcSkeleton,
) -> Result<ValidatedPracticalProgramAssemblyPlan, PracticalVcError> {
    let wire = expected_assembly(source, vc, skeleton)?;
    let bytes = encode_bounded(&wire)?;
    import_csharp_practical_program_assembly_plan_json(&bytes, source, vc, skeleton)
}

pub fn import_csharp_practical_program_assembly_plan_json(
    input: &[u8],
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
    skeleton: &ValidatedPracticalVcSkeleton,
) -> Result<ValidatedPracticalProgramAssemblyPlan, PracticalVcError> {
    validate_source(source)?;
    validate_vc_lineage(source, vc)?;
    validate_skeleton_lineage(source, skeleton)?;
    require_transport_bound(input)?;
    require_schema(input, CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE)?;
    let wire: WirePracticalProgramAssemblyPlan = decode_wire(input)?;
    let reencoded = encode_bounded(&wire)?;
    if reencoded != input {
        return Err(failure(
            PracticalVcValidationPhase::Canonical,
            PracticalVcErrorCode::Canonical,
        ));
    }
    if !wire.imports.is_empty()
        || !wire.proof_node_table.is_empty()
        || !wire.theory_certificates.is_empty()
        || !zero_axiom_report(&wire.axiom_report)
    {
        return Err(failure(
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::CertificateStructure,
        ));
    }
    let expected = expected_assembly(source, vc, skeleton)?;
    validate_common_linkage(
        &wire.semantic_context,
        &wire.foundation_descriptor,
        &wire.source_ir,
        source,
    )?;
    if wire.source_vc != expected.source_vc
        || wire.source_skeleton != expected.source_skeleton
        || wire.certificate_format != CERTIFICATE_V0_FORMAT
        || wire.generated_declaration_kinds != generated_declaration_kinds()
        || wire.ordinary_term_forms != ordinary_term_forms()
    {
        return Err(failure(
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::AssemblyProfile,
        ));
    }
    if wire.limits != exact_limits() {
        return Err(limit_error());
    }
    let recomputed = assembly_hash(&wire)?;
    if !valid_sha256(&wire.assembly_sha256) || wire.assembly_sha256 != recomputed {
        return Err(failure(
            PracticalVcValidationPhase::Hash,
            PracticalVcErrorCode::Hash,
        ));
    }
    Ok(ValidatedPracticalProgramAssemblyPlan {
        wire,
        canonical_bytes: input.to_vec(),
    })
}

fn expected_assembly(
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
    skeleton: &ValidatedPracticalVcSkeleton,
) -> Result<WirePracticalProgramAssemblyPlan, PracticalVcError> {
    validate_source(source)?;
    validate_vc_lineage(source, vc)?;
    validate_skeleton_lineage(source, skeleton)?;
    let mut wire = WirePracticalProgramAssemblyPlan {
        schema: CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE.to_owned(),
        semantic_context: canonical_context_raw(source.artifact_context)?,
        foundation_descriptor: expected_foundation(source.artifact_context)?,
        source_ir: PracticalArtifactLink::from_validated(&source.vir.artifact_ref()),
        source_vc: PracticalArtifactLink::from_validated(&vc.artifact_ref()),
        source_skeleton: PracticalArtifactLink::from_validated(&skeleton.artifact_ref()),
        certificate_format: CERTIFICATE_V0_FORMAT.to_owned(),
        generated_declaration_kinds: generated_declaration_kinds(),
        ordinary_term_forms: ordinary_term_forms(),
        imports: Vec::new(),
        proof_node_table: Vec::new(),
        theory_certificates: Vec::new(),
        axiom_report: PracticalZeroAxiomReport {
            core_axiom_count: 0,
            builtin_theory_axiom_count: 0,
            go_semantics_axiom_count: 0,
            external_axiom_count: 0,
            total_axiom_count: 0,
            entries: Vec::new(),
            declaration_dependencies: Vec::new(),
        },
        limits: exact_limits(),
        assembly_sha256: ZERO_SHA256.to_owned(),
    };
    wire.assembly_sha256 = assembly_hash(&wire)?;
    Ok(wire)
}

/// Enforces the ordinary-context intersection directly on a real Certificate
/// v0 value.  This is opt-in and therefore cannot alter predecessor assembly
/// acceptance.  It performs no proof search and no checker invocation.
pub fn validate_csharp_practical_certificate_structure(
    certificate: &Certificate,
) -> Result<(), PracticalVcError> {
    if !certificate.imports.is_empty()
        || !certificate.proof_node_table.is_empty()
        || !certificate.theory_certificates.is_empty()
        || certificate.declarations.iter().any(|declaration| {
            matches!(
                declaration.kind,
                DeclarationKind::Axiom { .. } | DeclarationKind::TheoryPrimitive { .. }
            )
        })
    {
        return Err(failure(
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::CertificateStructure,
        ));
    }
    let recomputed = build_axiom_report(certificate).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::AxiomReport,
        )
    })?;
    if certificate.axiom_report != recomputed
        || !certificate.axiom_report.entries.is_empty()
        || !certificate.axiom_report.declaration_dependencies.is_empty()
        || certificate.axiom_report.summary.core_axiom_count != 0
        || certificate.axiom_report.summary.builtin_theory_axiom_count != 0
        || certificate.axiom_report.summary.go_semantics_axiom_count != 0
        || certificate.axiom_report.summary.external_axiom_count != 0
        || certificate.axiom_report.summary.total_axiom_count != 0
    {
        return Err(failure(
            PracticalVcValidationPhase::Assembly,
            PracticalVcErrorCode::AxiomReport,
        ));
    }
    Ok(())
}

fn zero_axiom_report(report: &PracticalZeroAxiomReport) -> bool {
    report.core_axiom_count == 0
        && report.builtin_theory_axiom_count == 0
        && report.go_semantics_axiom_count == 0
        && report.external_axiom_count == 0
        && report.total_axiom_count == 0
        && report.entries.is_empty()
        && report.declaration_dependencies.is_empty()
}

fn validate_source(source: PracticalVcSource<'_>) -> Result<(), PracticalVcError> {
    let context = source.artifact_context.typed_context();
    let foundation = context.foundation_descriptor();
    let vir_ref = source.vir.artifact_ref();
    if context.semantic_profile() != CSHARP_PRACTICAL_PROFILE
        || foundation.schema() != FOUNDATION_DESCRIPTOR_SCHEMA
        || foundation.id() != FOUNDATION_DESCRIPTOR_ID
        || foundation.content_sha256() != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
        || vir_ref.schema() != SUCCESSOR_VIR_SCHEMA
        || vir_ref.sha256() != source.vir.hash()
        || vir_ref.canonical_bytes()
            != u64::try_from(source.vir.canonical_bytes().len()).unwrap_or(u64::MAX)
        || !vir_ref.matches_validated_lineage(source.artifact_context, source.captured_inputs)
    {
        return Err(failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        ));
    }
    Ok(())
}

fn validate_vc_lineage(
    source: PracticalVcSource<'_>,
    vc: &ValidatedPracticalVc,
) -> Result<(), PracticalVcError> {
    let reference = vc.artifact_ref();
    if reference.schema() != SUCCESSOR_VC_SCHEMA
        || reference.sha256() != vc.hash()
        || reference.canonical_bytes()
            != u64::try_from(vc.canonical_bytes().len()).unwrap_or(u64::MAX)
        || !reference.matches_validated_lineage(source.artifact_context, source.captured_inputs)
    {
        return Err(failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        ));
    }
    Ok(())
}

fn validate_skeleton_lineage(
    source: PracticalVcSource<'_>,
    skeleton: &ValidatedPracticalVcSkeleton,
) -> Result<(), PracticalVcError> {
    let reference = skeleton.artifact_ref();
    if reference.schema() != SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA
        || reference.sha256() != skeleton.hash()
        || reference.canonical_bytes()
            != u64::try_from(skeleton.canonical_bytes().len()).unwrap_or(u64::MAX)
        || !reference.matches_validated_lineage(source.artifact_context, source.captured_inputs)
    {
        return Err(failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        ));
    }
    Ok(())
}

fn validate_common_linkage(
    semantic_context: &RawValue,
    foundation: &PracticalFoundationLink,
    source_ir: &PracticalArtifactLink,
    source: PracticalVcSource<'_>,
) -> Result<(), PracticalVcError> {
    let expected_context = canonical_practical_json_bytes(
        source.artifact_context.semantic_context(),
    )
    .map_err(|_| {
        failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        )
    })?;
    if semantic_context.get().as_bytes() != expected_context {
        return Err(failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        ));
    }
    let expected_foundation = expected_foundation(source.artifact_context)?;
    let expected_vir = PracticalArtifactLink::from_validated(&source.vir.artifact_ref());
    if foundation != &expected_foundation || source_ir != &expected_vir {
        return Err(failure(
            PracticalVcValidationPhase::Linkage,
            PracticalVcErrorCode::Linkage,
        ));
    }
    Ok(())
}

fn expected_foundation(
    context: &PracticalArtifactContext,
) -> Result<PracticalFoundationLink, PracticalVcError> {
    let foundation = context.typed_context().foundation_descriptor();
    if foundation.schema() != FOUNDATION_DESCRIPTOR_SCHEMA
        || foundation.id() != FOUNDATION_DESCRIPTOR_ID
        || foundation.content_sha256() != FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    {
        return Err(failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        ));
    }
    Ok(PracticalFoundationLink {
        schema: foundation.schema().to_owned(),
        id: foundation.id().to_owned(),
        content_sha256: foundation.content_sha256().to_owned(),
    })
}

fn canonical_context_raw(
    context: &PracticalArtifactContext,
) -> Result<Box<RawValue>, PracticalVcError> {
    let bytes = canonical_practical_json_bytes(context.semantic_context()).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        )
    })?;
    let text = String::from_utf8(bytes).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        )
    })?;
    RawValue::from_string(text).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Context,
            PracticalVcErrorCode::Context,
        )
    })
}

fn vc_hash(wire: &WirePracticalVcDocument) -> Result<String, PracticalVcError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        semantic_context: &'a RawValue,
        foundation_descriptor: &'a PracticalFoundationLink,
        compilation_id: &'a str,
        input_set_sha256: &'a str,
        source_ir: &'a PracticalArtifactLink,
        ordinary_encoding_profile: &'a str,
        verification_limit_profile: &'a str,
        ordinary_term_forms: &'a [String],
        type_encodings: &'a [PracticalTypeEncoding],
        operation_encodings: &'a [PracticalOperationEncoding],
        control_encodings: &'a [PracticalControlEncoding],
        obligation_groups: &'a [PracticalObligationGroup],
        limits: &'a PracticalOrdinaryTermLimits,
        resource_reservation: &'a PracticalVcResourceReservation,
    }
    let payload = serialize_hash_preimage(&Preimage {
        schema: &wire.schema,
        semantic_context: &wire.semantic_context,
        foundation_descriptor: &wire.foundation_descriptor,
        compilation_id: &wire.compilation_id,
        input_set_sha256: &wire.input_set_sha256,
        source_ir: &wire.source_ir,
        ordinary_encoding_profile: &wire.ordinary_encoding_profile,
        verification_limit_profile: &wire.verification_limit_profile,
        ordinary_term_forms: &wire.ordinary_term_forms,
        type_encodings: &wire.type_encodings,
        operation_encodings: &wire.operation_encodings,
        control_encodings: &wire.control_encodings,
        obligation_groups: &wire.obligation_groups,
        limits: &wire.limits,
        resource_reservation: &wire.resource_reservation,
    })?;
    hash_complete(CSHARP_PRACTICAL_VC_HASH_DOMAIN, &payload)
}

fn assembly_hash(wire: &WirePracticalProgramAssemblyPlan) -> Result<String, PracticalVcError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        semantic_context: &'a RawValue,
        foundation_descriptor: &'a PracticalFoundationLink,
        source_ir: &'a PracticalArtifactLink,
        source_vc: &'a PracticalArtifactLink,
        source_skeleton: &'a PracticalArtifactLink,
        certificate_format: &'a str,
        generated_declaration_kinds: &'a [String],
        ordinary_term_forms: &'a [String],
        imports: &'a [Value],
        proof_node_table: &'a [Value],
        theory_certificates: &'a [Value],
        axiom_report: &'a PracticalZeroAxiomReport,
        limits: &'a PracticalOrdinaryTermLimits,
    }
    let payload = serialize_hash_preimage(&Preimage {
        schema: &wire.schema,
        semantic_context: &wire.semantic_context,
        foundation_descriptor: &wire.foundation_descriptor,
        source_ir: &wire.source_ir,
        source_vc: &wire.source_vc,
        source_skeleton: &wire.source_skeleton,
        certificate_format: &wire.certificate_format,
        generated_declaration_kinds: &wire.generated_declaration_kinds,
        ordinary_term_forms: &wire.ordinary_term_forms,
        imports: &wire.imports,
        proof_node_table: &wire.proof_node_table,
        theory_certificates: &wire.theory_certificates,
        axiom_report: &wire.axiom_report,
        limits: &wire.limits,
    })?;
    hash_complete(CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN, &payload)
}

fn serialize_hash_preimage<T: Serialize>(value: &T) -> Result<Vec<u8>, PracticalVcError> {
    serde_json::to_vec(value)
        .map_err(|_| failure(PracticalVcValidationPhase::Hash, PracticalVcErrorCode::Hash))
}

fn hash_complete(domain: HashDomain, payload: &[u8]) -> Result<String, PracticalVcError> {
    hash_domain_separated_raw(domain, payload)
        .map(|hash| hash.to_hex())
        .map_err(|_| failure(PracticalVcValidationPhase::Hash, PracticalVcErrorCode::Hash))
}

fn encode_bounded<T: Serialize>(value: &T) -> Result<Vec<u8>, PracticalVcError> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Canonical,
            PracticalVcErrorCode::Canonical,
        )
    })?;
    require_transport_bound(&bytes)?;
    Ok(bytes)
}

fn decode_wire<T: for<'de> Deserialize<'de>>(input: &[u8]) -> Result<T, PracticalVcError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let wire = T::deserialize(&mut deserializer).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Transport,
            PracticalVcErrorCode::Json,
        )
    })?;
    deserializer.end().map_err(|_| {
        failure(
            PracticalVcValidationPhase::Transport,
            PracticalVcErrorCode::Json,
        )
    })?;
    Ok(wire)
}

fn require_schema(input: &[u8], expected: &str) -> Result<(), PracticalVcError> {
    #[derive(Deserialize)]
    struct SchemaProbe {
        schema: String,
    }
    let probe: SchemaProbe = serde_json::from_slice(input).map_err(|_| {
        failure(
            PracticalVcValidationPhase::Transport,
            PracticalVcErrorCode::Json,
        )
    })?;
    if probe.schema == expected {
        Ok(())
    } else {
        Err(failure(
            PracticalVcValidationPhase::Schema,
            PracticalVcErrorCode::Schema,
        ))
    }
}

fn require_transport_bound(input: &[u8]) -> Result<(), PracticalVcError> {
    if input.is_empty()
        || input.starts_with(&[0xef, 0xbb, 0xbf])
        || u64::try_from(input.len()).unwrap_or(u64::MAX) > VC_TRANSPORT_BYTES_MAX
    {
        Err(failure(
            PracticalVcValidationPhase::Transport,
            PracticalVcErrorCode::Json,
        ))
    } else {
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
