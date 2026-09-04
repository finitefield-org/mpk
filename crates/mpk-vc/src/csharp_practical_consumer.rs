//! Private successor VC and assembly consumers for migrated predecessors.
//!
//! W08 proves the predecessor obligation/verdict equivalence. This module
//! consumes only the successor v2 artifact references and deterministically
//! closes their v3 VC/skeleton and ordinary-context v2 assembly lineage.

use crate::csharp_practical_registry::{
    validate_successor_registry_document, SuccessorRegistryDocumentKind,
    FOUNDATION_DESCRIPTOR_CONTENT_SHA256, FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
};
use crate::csharp_practical_source_artifacts::{
    SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA, SUCCESSOR_VC_SCHEMA,
};
use crate::csharp_practical_vc_model::{
    CERTIFICATE_V0_FORMAT, CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN,
    CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE, CSHARP_PRACTICAL_VC_HASH_DOMAIN,
};
use crate::{hash_domain_separated_raw, parse_strict_json, HashDomain, StrictJsonLimits};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt;

pub const PRIVATE_PREDECESSOR_VC_SCHEMA: &str = SUCCESSOR_VC_SCHEMA;
pub const PRIVATE_PREDECESSOR_SKELETON_SCHEMA: &str = SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA;
pub const PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA: &str = CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_PROFILE;
pub const PRIVATE_CERTIFICATE_FORMAT: &str = CERTIFICATE_V0_FORMAT;

const VC_HASH_DOMAIN: HashDomain = CSHARP_PRACTICAL_VC_HASH_DOMAIN;
const ASSEMBLY_HASH_DOMAIN: HashDomain = CSHARP_PRACTICAL_PROGRAM_ASSEMBLY_HASH_DOMAIN;
const TRANSPORT_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 128, 1_048_576);
const ARTIFACT_BYTES_MAX: u64 = 268_435_456;

#[derive(Clone, Copy, Debug)]
pub struct PrivateVerificationArtifactRef<'a> {
    pub schema: &'a str,
    pub sha256: &'a str,
    pub canonical_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct PrivatePredecessorVerificationSource<'a> {
    pub semantic_context: &'a Value,
    pub source_ir: PrivateVerificationArtifactRef<'a>,
    pub source_manifest: PrivateVerificationArtifactRef<'a>,
    pub obligation_sha256: &'a str,
    pub verdict_sha256: &'a str,
    pub axiom_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateVerificationArtifact {
    schema: &'static str,
    sha256: String,
    canonical_bytes: Vec<u8>,
}

impl PrivateVerificationArtifact {
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivatePredecessorVerificationArtifacts {
    vc: PrivateVerificationArtifact,
    skeleton: PrivateVerificationArtifact,
    assembly: PrivateVerificationArtifact,
}

impl PrivatePredecessorVerificationArtifacts {
    pub fn vc(&self) -> &PrivateVerificationArtifact {
        &self.vc
    }

    pub fn skeleton(&self) -> &PrivateVerificationArtifact {
        &self.skeleton
    }

    pub fn assembly(&self) -> &PrivateVerificationArtifact {
        &self.assembly
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateConsumerPhase {
    Source,
    Transport,
    Linkage,
    Hash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateConsumerError {
    phase: PrivateConsumerPhase,
}

impl PrivateConsumerError {
    pub const fn phase(&self) -> PrivateConsumerPhase {
        self.phase
    }
}

impl fmt::Display for PrivateConsumerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "private predecessor consumer at {:?}",
            self.phase
        )
    }
}

impl Error for PrivateConsumerError {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    schema: String,
    sha256: String,
    canonical_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FoundationDescriptor {
    schema: String,
    id: String,
    content_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ZeroAxiomReport {
    core_axiom_count: u64,
    builtin_theory_axiom_count: u64,
    go_semantics_axiom_count: u64,
    external_axiom_count: u64,
    total_axiom_count: u64,
    entries: Vec<Value>,
    declaration_dependencies: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PredecessorVc {
    schema: String,
    semantic_context: Value,
    foundation_descriptor: FoundationDescriptor,
    source_ir: ArtifactRef,
    source_manifest: ArtifactRef,
    obligation_sha256: String,
    verdict_sha256: String,
    axiom_count: u64,
    vc_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PredecessorSkeleton {
    schema: String,
    semantic_context: Value,
    foundation_descriptor: FoundationDescriptor,
    source_ir: ArtifactRef,
    source_vc: ArtifactRef,
    source_manifest: ArtifactRef,
    obligation_sha256: String,
    theorem_declarations: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PredecessorAssembly {
    schema: String,
    semantic_context: Value,
    foundation_descriptor: FoundationDescriptor,
    source_ir: ArtifactRef,
    source_vc: ArtifactRef,
    source_skeleton: ArtifactRef,
    certificate_format: String,
    imports: Vec<Value>,
    proof_node_table: Vec<Value>,
    theory_certificates: Vec<Value>,
    axiom_report: ZeroAxiomReport,
    assembly_sha256: String,
}

/// Builds and independently re-imports all predecessor verification consumers.
pub fn build_private_predecessor_verification_artifacts(
    source: PrivatePredecessorVerificationSource<'_>,
) -> Result<PrivatePredecessorVerificationArtifacts, PrivateConsumerError> {
    validate_source(source)?;
    let mut vc = PredecessorVc {
        schema: PRIVATE_PREDECESSOR_VC_SCHEMA.to_owned(),
        semantic_context: source.semantic_context.clone(),
        foundation_descriptor: expected_foundation(),
        source_ir: artifact_ref(source.source_ir),
        source_manifest: artifact_ref(source.source_manifest),
        obligation_sha256: source.obligation_sha256.to_owned(),
        verdict_sha256: source.verdict_sha256.to_owned(),
        axiom_count: source.axiom_count,
        vc_sha256: String::new(),
    };
    vc.vc_sha256 = predecessor_vc_hash(&vc)?;
    let vc_bytes = encode(&vc)?;
    let vc_artifact = PrivateVerificationArtifact {
        schema: PRIVATE_PREDECESSOR_VC_SCHEMA,
        sha256: vc.vc_sha256.clone(),
        canonical_bytes: vc_bytes,
    };
    let skeleton = PredecessorSkeleton {
        schema: PRIVATE_PREDECESSOR_SKELETON_SCHEMA.to_owned(),
        semantic_context: source.semantic_context.clone(),
        foundation_descriptor: expected_foundation(),
        source_ir: artifact_ref(source.source_ir),
        source_vc: generated_artifact_ref(&vc_artifact),
        source_manifest: artifact_ref(source.source_manifest),
        obligation_sha256: source.obligation_sha256.to_owned(),
        theorem_declarations: Vec::new(),
    };
    let skeleton_bytes = encode(&skeleton)?;
    let skeleton_artifact = PrivateVerificationArtifact {
        schema: PRIVATE_PREDECESSOR_SKELETON_SCHEMA,
        sha256: hash_complete(VC_HASH_DOMAIN, &skeleton_bytes)?,
        canonical_bytes: skeleton_bytes,
    };
    let mut assembly = PredecessorAssembly {
        schema: PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA.to_owned(),
        semantic_context: source.semantic_context.clone(),
        foundation_descriptor: expected_foundation(),
        source_ir: artifact_ref(source.source_ir),
        source_vc: generated_artifact_ref(&vc_artifact),
        source_skeleton: generated_artifact_ref(&skeleton_artifact),
        certificate_format: PRIVATE_CERTIFICATE_FORMAT.to_owned(),
        imports: Vec::new(),
        proof_node_table: Vec::new(),
        theory_certificates: Vec::new(),
        axiom_report: zero_axiom_report(),
        assembly_sha256: String::new(),
    };
    assembly.assembly_sha256 = predecessor_assembly_hash(&assembly)?;
    let assembly_artifact = PrivateVerificationArtifact {
        schema: PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA,
        sha256: assembly.assembly_sha256.clone(),
        canonical_bytes: encode(&assembly)?,
    };
    let artifacts = PrivatePredecessorVerificationArtifacts {
        vc: vc_artifact,
        skeleton: skeleton_artifact,
        assembly: assembly_artifact,
    };
    validate_private_predecessor_verification_artifacts(source, &artifacts)?;
    Ok(artifacts)
}

pub fn validate_private_predecessor_verification_artifacts(
    source: PrivatePredecessorVerificationSource<'_>,
    artifacts: &PrivatePredecessorVerificationArtifacts,
) -> Result<(), PrivateConsumerError> {
    validate_source(source)?;
    let vc: PredecessorVc = decode(artifacts.vc.canonical_bytes())?;
    let skeleton: PredecessorSkeleton = decode(artifacts.skeleton.canonical_bytes())?;
    let assembly: PredecessorAssembly = decode(artifacts.assembly.canonical_bytes())?;
    let expected_ir = artifact_ref(source.source_ir);
    let expected_manifest = artifact_ref(source.source_manifest);
    if vc.schema != PRIVATE_PREDECESSOR_VC_SCHEMA
        || skeleton.schema != PRIVATE_PREDECESSOR_SKELETON_SCHEMA
        || assembly.schema != PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA
        || vc.semantic_context != *source.semantic_context
        || skeleton.semantic_context != *source.semantic_context
        || assembly.semantic_context != *source.semantic_context
        || vc.foundation_descriptor != expected_foundation()
        || skeleton.foundation_descriptor != expected_foundation()
        || assembly.foundation_descriptor != expected_foundation()
        || vc.source_ir != expected_ir
        || vc.source_manifest != expected_manifest
        || skeleton.source_ir != expected_ir
        || skeleton.source_manifest != expected_manifest
        || assembly.source_ir != expected_ir
        || vc.obligation_sha256 != source.obligation_sha256
        || skeleton.obligation_sha256 != source.obligation_sha256
        || vc.verdict_sha256 != source.verdict_sha256
        || vc.axiom_count != source.axiom_count
        || assembly.axiom_report != zero_axiom_report()
        || !skeleton.theorem_declarations.is_empty()
        || !assembly.imports.is_empty()
        || !assembly.proof_node_table.is_empty()
        || !assembly.theory_certificates.is_empty()
        || assembly.certificate_format != PRIVATE_CERTIFICATE_FORMAT
    {
        return Err(error(PrivateConsumerPhase::Linkage));
    }
    let expected_vc = generated_artifact_ref(&artifacts.vc);
    let expected_skeleton = generated_artifact_ref(&artifacts.skeleton);
    if skeleton.source_vc != expected_vc
        || assembly.source_vc != expected_vc
        || assembly.source_skeleton != expected_skeleton
    {
        return Err(error(PrivateConsumerPhase::Linkage));
    }
    if predecessor_vc_hash(&vc)? != vc.vc_sha256
        || vc.vc_sha256 != artifacts.vc.sha256
        || hash_complete(VC_HASH_DOMAIN, artifacts.skeleton.canonical_bytes())?
            != artifacts.skeleton.sha256
        || predecessor_assembly_hash(&assembly)? != assembly.assembly_sha256
        || assembly.assembly_sha256 != artifacts.assembly.sha256
    {
        return Err(error(PrivateConsumerPhase::Hash));
    }
    Ok(())
}

/// Re-imports raw consumer transports before constructing their typed wrapper.
/// This is the mutation-test boundary used by W09 and accepts no prior schema.
pub fn validate_private_predecessor_verification_transports(
    source: PrivatePredecessorVerificationSource<'_>,
    vc: &[u8],
    skeleton: &[u8],
    assembly: &[u8],
) -> Result<(), PrivateConsumerError> {
    let vc_wire: PredecessorVc = decode(vc)?;
    let _skeleton_wire: PredecessorSkeleton = decode(skeleton)?;
    let assembly_wire: PredecessorAssembly = decode(assembly)?;
    let artifacts = PrivatePredecessorVerificationArtifacts {
        vc: PrivateVerificationArtifact {
            schema: PRIVATE_PREDECESSOR_VC_SCHEMA,
            sha256: vc_wire.vc_sha256.clone(),
            canonical_bytes: vc.to_vec(),
        },
        skeleton: PrivateVerificationArtifact {
            schema: PRIVATE_PREDECESSOR_SKELETON_SCHEMA,
            sha256: hash_complete(VC_HASH_DOMAIN, skeleton)?,
            canonical_bytes: skeleton.to_vec(),
        },
        assembly: PrivateVerificationArtifact {
            schema: PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA,
            sha256: assembly_wire.assembly_sha256.clone(),
            canonical_bytes: assembly.to_vec(),
        },
    };
    validate_private_predecessor_verification_artifacts(source, &artifacts)
}

/// Independently validates the assembly bytes at the certificate-consumer
/// boundary. Source/VC lineage is checked by the full transport importer;
/// this narrower gate prevents a caller from substituting a hash-only token
/// for the strict assembly document consumed by `mpk-cli`.
pub fn validate_private_predecessor_program_assembly_transport(
    input: &[u8],
    expected_sha256: &str,
) -> Result<(), PrivateConsumerError> {
    let assembly: PredecessorAssembly = decode(input)?;
    if assembly.schema != PRIVATE_PREDECESSOR_ASSEMBLY_SCHEMA
        || !valid_semantic_context(&assembly.semantic_context)
        || assembly.foundation_descriptor != expected_foundation()
        || assembly.source_ir.schema != "mpk.vir.v2"
        || assembly.source_vc.schema != PRIVATE_PREDECESSOR_VC_SCHEMA
        || assembly.source_skeleton.schema != PRIVATE_PREDECESSOR_SKELETON_SCHEMA
        || [
            &assembly.source_ir,
            &assembly.source_vc,
            &assembly.source_skeleton,
        ]
        .into_iter()
        .any(|reference| {
            !valid_sha256(&reference.sha256)
                || reference.canonical_bytes == 0
                || reference.canonical_bytes > ARTIFACT_BYTES_MAX
        })
        || assembly.certificate_format != PRIVATE_CERTIFICATE_FORMAT
        || !assembly.imports.is_empty()
        || !assembly.proof_node_table.is_empty()
        || !assembly.theory_certificates.is_empty()
        || assembly.axiom_report != zero_axiom_report()
    {
        return Err(error(PrivateConsumerPhase::Linkage));
    }
    if !valid_sha256(expected_sha256)
        || predecessor_assembly_hash(&assembly)? != assembly.assembly_sha256
        || assembly.assembly_sha256 != expected_sha256
    {
        return Err(error(PrivateConsumerPhase::Hash));
    }
    Ok(())
}

fn validate_source(
    source: PrivatePredecessorVerificationSource<'_>,
) -> Result<(), PrivateConsumerError> {
    if !valid_semantic_context(source.semantic_context)
        || source.source_ir.schema != "mpk.vir.v2"
        || source.source_manifest.schema != "mpk.source_manifest.frontend.v2"
        || !valid_sha256(source.source_ir.sha256)
        || !valid_sha256(source.source_manifest.sha256)
        || source.source_ir.canonical_bytes == 0
        || source.source_manifest.canonical_bytes == 0
        || source.source_ir.canonical_bytes > ARTIFACT_BYTES_MAX
        || source.source_manifest.canonical_bytes > ARTIFACT_BYTES_MAX
        || !valid_sha256(source.obligation_sha256)
        || !valid_sha256(source.verdict_sha256)
        || source.axiom_count != 0
    {
        return Err(error(PrivateConsumerPhase::Source));
    }
    Ok(())
}

fn valid_semantic_context(value: &Value) -> bool {
    serde_json::to_vec(value).is_ok_and(|transport| {
        validate_successor_registry_document(
            SuccessorRegistryDocumentKind::SemanticContext,
            &transport,
        )
        .is_ok()
    })
}

fn artifact_ref(value: PrivateVerificationArtifactRef<'_>) -> ArtifactRef {
    ArtifactRef {
        schema: value.schema.to_owned(),
        sha256: value.sha256.to_owned(),
        canonical_bytes: value.canonical_bytes,
    }
}

fn generated_artifact_ref(value: &PrivateVerificationArtifact) -> ArtifactRef {
    ArtifactRef {
        schema: value.schema.to_owned(),
        sha256: value.sha256.clone(),
        canonical_bytes: value.canonical_bytes.len() as u64,
    }
}

fn expected_foundation() -> FoundationDescriptor {
    FoundationDescriptor {
        schema: FOUNDATION_DESCRIPTOR_SCHEMA.to_owned(),
        id: FOUNDATION_DESCRIPTOR_ID.to_owned(),
        content_sha256: FOUNDATION_DESCRIPTOR_CONTENT_SHA256.to_owned(),
    }
}

fn zero_axiom_report() -> ZeroAxiomReport {
    ZeroAxiomReport {
        core_axiom_count: 0,
        builtin_theory_axiom_count: 0,
        go_semantics_axiom_count: 0,
        external_axiom_count: 0,
        total_axiom_count: 0,
        entries: Vec::new(),
        declaration_dependencies: Vec::new(),
    }
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, PrivateConsumerError> {
    serde_json::to_vec(value).map_err(|_| error(PrivateConsumerPhase::Transport))
}

fn decode<T: DeserializeOwned + Serialize>(input: &[u8]) -> Result<T, PrivateConsumerError> {
    parse_strict_json(input, TRANSPORT_LIMITS)
        .map_err(|_| error(PrivateConsumerPhase::Transport))?;
    let value =
        serde_json::from_slice(input).map_err(|_| error(PrivateConsumerPhase::Transport))?;
    if encode(&value)? != input {
        return Err(error(PrivateConsumerPhase::Transport));
    }
    Ok(value)
}

fn predecessor_vc_hash(value: &PredecessorVc) -> Result<String, PrivateConsumerError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        semantic_context: &'a Value,
        foundation_descriptor: &'a FoundationDescriptor,
        source_ir: &'a ArtifactRef,
        source_manifest: &'a ArtifactRef,
        obligation_sha256: &'a str,
        verdict_sha256: &'a str,
        axiom_count: u64,
    }
    let bytes = encode(&Preimage {
        schema: &value.schema,
        semantic_context: &value.semantic_context,
        foundation_descriptor: &value.foundation_descriptor,
        source_ir: &value.source_ir,
        source_manifest: &value.source_manifest,
        obligation_sha256: &value.obligation_sha256,
        verdict_sha256: &value.verdict_sha256,
        axiom_count: value.axiom_count,
    })?;
    hash_complete(VC_HASH_DOMAIN, &bytes)
}

fn predecessor_assembly_hash(value: &PredecessorAssembly) -> Result<String, PrivateConsumerError> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        semantic_context: &'a Value,
        foundation_descriptor: &'a FoundationDescriptor,
        source_ir: &'a ArtifactRef,
        source_vc: &'a ArtifactRef,
        source_skeleton: &'a ArtifactRef,
        certificate_format: &'a str,
        imports: &'a [Value],
        proof_node_table: &'a [Value],
        theory_certificates: &'a [Value],
        axiom_report: &'a ZeroAxiomReport,
    }
    let bytes = encode(&Preimage {
        schema: &value.schema,
        semantic_context: &value.semantic_context,
        foundation_descriptor: &value.foundation_descriptor,
        source_ir: &value.source_ir,
        source_vc: &value.source_vc,
        source_skeleton: &value.source_skeleton,
        certificate_format: &value.certificate_format,
        imports: &value.imports,
        proof_node_table: &value.proof_node_table,
        theory_certificates: &value.theory_certificates,
        axiom_report: &value.axiom_report,
    })?;
    hash_complete(ASSEMBLY_HASH_DOMAIN, &bytes)
}

fn hash_complete(domain: HashDomain, bytes: &[u8]) -> Result<String, PrivateConsumerError> {
    hash_domain_separated_raw(domain, bytes)
        .map(|hash| hash.to_hex())
        .map_err(|_| error(PrivateConsumerPhase::Hash))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn error(phase: PrivateConsumerPhase) -> PrivateConsumerError {
    PrivateConsumerError { phase }
}
