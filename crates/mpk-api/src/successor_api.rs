//! Inactive successor AI API staging boundary.
//!
//! This module is reachable only through explicit staging calls. It does not
//! replace the released `mpk.ai.api.v1` router. Every session is bound to one
//! validated successor semantic context and selection, and every source/VC
//! operation repeats that identity before state can change. All responses are
//! untrusted helper data; canonical Certificate v0 checking remains outside
//! this API.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_context_linkage, CompiledProfileEnvelope,
    ProfileContractField, ProfileRegistryIdentity, SelectionEnvelope, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    SuccessorSourceManifestStage, ValidatedSuccessorSourceManifest, ValidatedSuccessorSourceMap,
    ValidatedSuccessorVir, SUCCESSOR_SOURCE_MANIFEST_SCHEMA, SUCCESSOR_SOURCE_MAP_SCHEMA,
    SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::successor_vc::{
    generate_successor_vc, SuccessorVcSource, ValidatedSuccessorVc, SUCCESSOR_VC_SCHEMA,
};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, StrictJsonLimits, VcFunction,
    VC_CANONICAL_JSON_BYTES_MAX, VIR_INPUT_JSON_BYTES_MAX,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::proof_api::ApiProofId;
use crate::session::{ApiService, ProofProfile, SessionId, SessionStatus, StartSessionRequest};
use crate::term_api::ApiTermId;
use crate::v1_router::V1Handler;
use crate::vc_api::{
    check_candidate_root, materialize_target, resolve_target_term, HelperStatus,
    VcCandidateBinding, VcCandidateResult, VcMemberSummary, VcProofTarget,
};

pub const SUCCESSOR_AI_API_PROFILE: &str = "mpk.ai.api.v2";
pub const SUCCESSOR_API_REJECTION_MESSAGE: &str = "AI API v2 request rejected";

const API_ENVELOPE_OVERHEAD: u64 = 1_048_576;
const API_WRAPPER_LEVELS: u64 = 768;
const API_STRING_BYTES_MAX: u64 = 1_048_576;
const API_TRANSPORT_BYTES_MAX: u64 = 269_484_032;
const API_SMALL_REQUEST_BYTES_MAX: u64 = 1_048_576;
const CHECK_MODE: &str = "fail_fast_per_candidate";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessorValidationPhase {
    Route,
    Transport,
    Shape,
    Scalar,
    Session,
    Artifact,
    Context,
    CanonicalTransport,
}

impl SuccessorValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Route => "route",
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Session => "session",
            Self::Artifact => "artifact",
            Self::Context => "context",
            Self::CanonicalTransport => "canonical_transport",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum SuccessorApiErrorCode {
    #[serde(rename = "AI_API_V2_ROUTE_UNKNOWN")]
    RouteUnknown,
    #[serde(rename = "AI_API_V2_PROFILE")]
    ApiProfile,
    #[serde(rename = "AI_API_V2_JSON_INVALID")]
    JsonInvalid,
    #[serde(rename = "AI_API_V2_SHAPE")]
    Shape,
    #[serde(rename = "AI_API_V2_SCALAR")]
    Scalar,
    #[serde(rename = "AI_API_V2_UNKNOWN_SESSION")]
    UnknownSession,
    #[serde(rename = "AI_API_V2_SESSION_STATE")]
    SessionState,
    #[serde(rename = "AI_API_V2_VIR_SCHEMA")]
    VirSchema,
    #[serde(rename = "AI_API_V2_VIR_INVALID")]
    VirInvalid,
    #[serde(rename = "AI_API_V2_VIR_HASH")]
    VirHash,
    #[serde(rename = "AI_API_V2_SOURCE_CONTEXT_UNKNOWN")]
    SourceContextUnknown,
    #[serde(rename = "AI_API_V2_SOURCE_MANIFEST_SCHEMA")]
    SourceManifestSchema,
    #[serde(rename = "AI_API_V2_SOURCE_MANIFEST_INVALID")]
    SourceManifestInvalid,
    #[serde(rename = "AI_API_V2_SOURCE_MANIFEST_HASH")]
    SourceManifestHash,
    #[serde(rename = "AI_API_V2_VC_INVALID")]
    VcInvalid,
    #[serde(rename = "AI_API_V2_CONTEXT_MISMATCH")]
    ContextMismatch,
    #[serde(rename = "AI_API_V2_TARGET_UNKNOWN")]
    TargetUnknown,
    #[serde(rename = "AI_API_V2_UNKNOWN_PROOF")]
    UnknownProof,
    #[serde(rename = "AI_API_V2_CANDIDATE_UNKNOWN")]
    CandidateUnknown,
    #[serde(rename = "AI_API_V2_CANONICAL_TRANSPORT")]
    CanonicalTransport,
}

impl SuccessorApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteUnknown => "AI_API_V2_ROUTE_UNKNOWN",
            Self::ApiProfile => "AI_API_V2_PROFILE",
            Self::JsonInvalid => "AI_API_V2_JSON_INVALID",
            Self::Shape => "AI_API_V2_SHAPE",
            Self::Scalar => "AI_API_V2_SCALAR",
            Self::UnknownSession => "AI_API_V2_UNKNOWN_SESSION",
            Self::SessionState => "AI_API_V2_SESSION_STATE",
            Self::VirSchema => "AI_API_V2_VIR_SCHEMA",
            Self::VirInvalid => "AI_API_V2_VIR_INVALID",
            Self::VirHash => "AI_API_V2_VIR_HASH",
            Self::SourceContextUnknown => "AI_API_V2_SOURCE_CONTEXT_UNKNOWN",
            Self::SourceManifestSchema => "AI_API_V2_SOURCE_MANIFEST_SCHEMA",
            Self::SourceManifestInvalid => "AI_API_V2_SOURCE_MANIFEST_INVALID",
            Self::SourceManifestHash => "AI_API_V2_SOURCE_MANIFEST_HASH",
            Self::VcInvalid => "AI_API_V2_VC_INVALID",
            Self::ContextMismatch => "AI_API_V2_CONTEXT_MISMATCH",
            Self::TargetUnknown => "AI_API_V2_TARGET_UNKNOWN",
            Self::UnknownProof => "AI_API_V2_UNKNOWN_PROOF",
            Self::CandidateUnknown => "AI_API_V2_CANDIDATE_UNKNOWN",
            Self::CanonicalTransport => "AI_API_V2_CANONICAL_TRANSPORT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorApiError {
    pub code: SuccessorApiErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip)]
    phase: SuccessorValidationPhase,
}

impl SuccessorApiError {
    fn new(
        phase: SuccessorValidationPhase,
        code: SuccessorApiErrorCode,
        field: Option<&'static str>,
        detail: Option<impl Into<String>>,
    ) -> Self {
        Self {
            code,
            message: SUCCESSOR_API_REJECTION_MESSAGE.to_owned(),
            field,
            detail: detail.map(Into::into),
            phase,
        }
    }

    pub const fn phase(&self) -> SuccessorValidationPhase {
        self.phase
    }
}

impl fmt::Display for SuccessorApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} during {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for SuccessorApiError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuccessorRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub handler: V1Handler,
}

pub const SUCCESSOR_ROUTES: &[SuccessorRoute] = &[
    route("POST", "/module/new", V1Handler::ModuleNew),
    route("POST", "/module/import", V1Handler::ModuleImport),
    route("POST", "/module/freeze", V1Handler::ModuleFreeze),
    route(
        "POST",
        "/module/export-certificate",
        V1Handler::ModuleExportCertificate,
    ),
    route("POST", "/term/sort", V1Handler::TermSort),
    route("POST", "/term/var", V1Handler::TermVar),
    route("POST", "/term/const", V1Handler::TermConst),
    route("POST", "/term/app", V1Handler::TermApp),
    route("POST", "/term/lam", V1Handler::TermLam),
    route("POST", "/term/pi", V1Handler::TermPi),
    route("POST", "/term/let", V1Handler::TermLet),
    route("POST", "/term/check", V1Handler::TermCheck),
    route("POST", "/term/infer", V1Handler::TermInfer),
    route("POST", "/term/defeq", V1Handler::TermDefeq),
    route("POST", "/proof/exact", V1Handler::ProofExact),
    route("POST", "/proof/apply", V1Handler::ProofApply),
    route("POST", "/proof/intro", V1Handler::ProofIntro),
    route("POST", "/proof/refl", V1Handler::ProofRefl),
    route("POST", "/proof/let", V1Handler::ProofLet),
    route("POST", "/proof/rewrite", V1Handler::ProofRewrite),
    route("POST", "/proof/eq-rec", V1Handler::ProofEqRec),
    route("POST", "/proof/constructor", V1Handler::ProofConstructor),
    route("POST", "/proof/recursor", V1Handler::ProofRecursor),
    route("POST", "/proof/conv", V1Handler::ProofConv),
    route("POST", "/proof/theory", V1Handler::ProofTheory),
    route("POST", "/proof/check-node", V1Handler::ProofCheckNode),
    route("POST", "/proof/check-decl", V1Handler::ProofCheckDecl),
    route("POST", "/vir/import", V1Handler::VirImport),
    route("POST", "/vc/generate", V1Handler::VcGenerate),
    route("GET", "/vc/list", V1Handler::VcList),
    route("POST", "/vc/start-proof", V1Handler::VcStartProof),
    route("POST", "/vc/attach-candidate", V1Handler::VcAttachCandidate),
    route("POST", "/vc/check-candidate", V1Handler::VcCheckCandidate),
];

const fn route(method: &'static str, path: &'static str, handler: V1Handler) -> SuccessorRoute {
    SuccessorRoute {
        method,
        path,
        handler,
    }
}

pub fn resolve_staged_route(
    api_profile: &str,
    method: &str,
    path: &str,
) -> Result<V1Handler, SuccessorApiError> {
    validate_api_profile(api_profile)?;
    SUCCESSOR_ROUTES
        .iter()
        .find(|route| route.method == method && route.path == path)
        .map(|route| route.handler)
        .ok_or_else(|| {
            SuccessorApiError::new(
                SuccessorValidationPhase::Route,
                SuccessorApiErrorCode::RouteUnknown,
                None,
                None::<String>,
            )
        })
}

#[derive(Clone, Debug)]
pub struct SuccessorFrontendArtifacts {
    pub vir: ValidatedSuccessorVir,
    pub source_map: ValidatedSuccessorSourceMap,
    pub source_manifest: ValidatedSuccessorSourceManifest,
    pub vc_profile_contract: Value,
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorFrontendArtifactRecord {
    vir: ValidatedSuccessorVir,
    source_map: ValidatedSuccessorSourceMap,
    source_manifest: ValidatedSuccessorSourceManifest,
    vc_profile_contract: Value,
    compiled_vc_profile_contract: CompiledProfileEnvelope,
}

impl ValidatedSuccessorFrontendArtifactRecord {
    pub fn vir(&self) -> &ValidatedSuccessorVir {
        &self.vir
    }

    pub fn source_map(&self) -> &ValidatedSuccessorSourceMap {
        &self.source_map
    }

    pub fn source_manifest(&self) -> &ValidatedSuccessorSourceManifest {
        &self.source_manifest
    }

    pub fn vc_profile_contract(&self) -> &Value {
        &self.vc_profile_contract
    }

    fn same_capability(&self, other: &Self) -> bool {
        self.vir.canonical_bytes() == other.vir.canonical_bytes()
            && self.source_map.canonical_bytes() == other.source_map.canonical_bytes()
            && self.source_manifest.canonical_bytes() == other.source_manifest.canonical_bytes()
            && self.vc_profile_contract == other.vc_profile_contract
    }
}

#[derive(Clone, Debug)]
pub struct SuccessorFrontendArtifactStore {
    registry_identity: ProfileRegistryIdentity,
    records: BTreeMap<String, ValidatedSuccessorFrontendArtifactRecord>,
}

impl SuccessorFrontendArtifactStore {
    pub fn empty(registry: &ValidatedSemanticProfileRegistry) -> Self {
        Self {
            registry_identity: registry.identity().clone(),
            records: BTreeMap::new(),
        }
    }

    pub fn from_frontend_successes(
        registry: &ValidatedSemanticProfileRegistry,
        artifacts: impl IntoIterator<Item = SuccessorFrontendArtifacts>,
    ) -> Result<Self, SuccessorApiError> {
        let mut records = BTreeMap::<String, ValidatedSuccessorFrontendArtifactRecord>::new();
        for artifacts in artifacts {
            let record = validate_frontend_record(registry, artifacts)?;
            let hash = record.source_manifest.hash().as_str().to_owned();
            if let Some(existing) = records.get(&hash) {
                if !existing.same_capability(&record) {
                    return Err(source_manifest_invalid(
                        "SOURCE_MANIFEST_STORE_HASH_COLLISION",
                    ));
                }
                continue;
            }
            records.insert(hash, record);
        }
        Ok(Self {
            registry_identity: registry.identity().clone(),
            records,
        })
    }

    pub fn get(&self, manifest_hash: &str) -> Option<&ValidatedSuccessorFrontendArtifactRecord> {
        self.records.get(manifest_hash)
    }
}

fn validate_frontend_record(
    registry: &ValidatedSemanticProfileRegistry,
    artifacts: SuccessorFrontendArtifacts,
) -> Result<ValidatedSuccessorFrontendArtifactRecord, SuccessorApiError> {
    if artifacts.source_manifest.stage() != SuccessorSourceManifestStage::Frontend {
        return Err(source_manifest_invalid("SOURCE_MANIFEST_STAGE"));
    }
    let vir_context = artifacts.vir.module().semantic_context();
    let map = artifacts.source_map.map();
    let manifest = artifacts.source_manifest.manifest();
    validate_semantic_context_linkage(vir_context, map.semantic_context())
        .and_then(|_| validate_semantic_context_linkage(vir_context, manifest.semantic_context()))
        .map_err(|_| context_mismatch("semantic_context"))?;
    if map.schema() != SUCCESSOR_SOURCE_MAP_SCHEMA
        || map.source_ir_schema() != SUCCESSOR_VIR_SCHEMA
        || map.source_ir_hash() != artifacts.vir.hash()
        || manifest.schema() != SUCCESSOR_SOURCE_MANIFEST_SCHEMA
        || manifest.vir_hash() != artifacts.vir.hash()
        || manifest.source_map_hash() != artifacts.source_map.hash()
        || manifest.vc_hash().is_some()
    {
        return Err(source_manifest_invalid("SOURCE_MANIFEST_ARTIFACT_LINKAGE"));
    }
    let compiled_vc_profile_contract = validate_compiled_profile_envelope(
        registry,
        &artifacts.vc_profile_contract,
        ProfileContractField::Vc,
    )
    .map_err(|_| vc_invalid("VC_PROFILE_CONTRACT"))?;
    if compiled_vc_profile_contract.profile_entry_sha256() != vir_context.profile_entry_sha256() {
        return Err(context_mismatch("vc_profile_contract"));
    }
    generate_successor_vc(SuccessorVcSource {
        registry,
        vir: &artifacts.vir,
        manifest: &artifacts.source_manifest,
        profile_contract: &artifacts.vc_profile_contract,
    })
    .map_err(|error| vc_invalid(error.code().as_str()))?;
    Ok(ValidatedSuccessorFrontendArtifactRecord {
        vir: artifacts.vir,
        source_map: artifacts.source_map,
        source_manifest: artifacts.source_manifest,
        vc_profile_contract: artifacts.vc_profile_contract,
        compiled_vc_profile_contract,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorSessionIdentity {
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
}

impl SuccessorSessionIdentity {
    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }
}

#[derive(Clone, Debug)]
pub struct ImportedSuccessorVir {
    vir: ValidatedSuccessorVir,
}

impl ImportedSuccessorVir {
    pub fn vir(&self) -> &ValidatedSuccessorVir {
        &self.vir
    }
}

#[derive(Clone, Debug)]
pub enum SuccessorSessionSourceState {
    Empty,
    VirImported(Box<ImportedSuccessorVir>),
    VcGenerated(Box<SuccessorVcSessionState>),
}

impl SuccessorSessionSourceState {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::VirImported(_) => "vir_imported",
            Self::VcGenerated(_) => "vc_generated",
        }
    }
}

#[derive(Clone, Debug)]
struct SuccessorSessionBinding {
    identity: SuccessorSessionIdentity,
    source: SuccessorSessionSourceState,
}

#[derive(Clone, Debug)]
pub struct SuccessorProofTargetRecord {
    target: VcProofTarget,
    target_term: ApiTermId,
    candidates: BTreeMap<String, ApiProofId>,
}

#[derive(Clone, Debug)]
pub struct SuccessorVcSessionState {
    imported: ImportedSuccessorVir,
    source_manifest_hash: String,
    input_set_hash: String,
    vc: ValidatedSuccessorVc,
    targets: BTreeMap<String, SuccessorProofTargetRecord>,
    next_target_index: u64,
}

#[derive(Debug)]
pub struct SuccessorApiService {
    registry: ValidatedSemanticProfileRegistry,
    store: SuccessorFrontendArtifactStore,
    proof_service: ApiService,
    sessions: BTreeMap<SessionId, SuccessorSessionBinding>,
    mutation_count: u64,
}

impl SuccessorApiService {
    pub fn new(
        registry: ValidatedSemanticProfileRegistry,
        store: SuccessorFrontendArtifactStore,
    ) -> Result<Self, SuccessorApiError> {
        if store.registry_identity != *registry.identity() {
            return Err(context_mismatch("profile_registry"));
        }
        Ok(Self {
            registry,
            store,
            proof_service: ApiService::new(),
            sessions: BTreeMap::new(),
            mutation_count: 0,
        })
    }

    pub fn proof_service(&self) -> &ApiService {
        &self.proof_service
    }

    pub fn proof_service_mut(&mut self) -> &mut ApiService {
        &mut self.proof_service
    }

    pub const fn mutation_count(&self) -> u64 {
        self.mutation_count
    }

    pub fn session_identity(&self, session_id: &SessionId) -> Option<&SuccessorSessionIdentity> {
        self.sessions
            .get(session_id)
            .map(|session| &session.identity)
    }

    pub fn source_state(&self, session_id: &SessionId) -> Option<&SuccessorSessionSourceState> {
        self.sessions.get(session_id).map(|session| &session.source)
    }

    pub fn target_term_id(&self, session_id: &SessionId, target_id: &str) -> Option<ApiTermId> {
        match &self.sessions.get(session_id)?.source {
            SuccessorSessionSourceState::VcGenerated(state) => state
                .targets
                .get(target_id)
                .map(|target| target.target_term),
            _ => None,
        }
    }

    pub fn target_binding(
        &self,
        session_id: &SessionId,
        target_id: &str,
    ) -> Option<&VcProofTarget> {
        match &self.sessions.get(session_id)?.source {
            SuccessorSessionSourceState::VcGenerated(state) => {
                state.targets.get(target_id).map(|target| &target.target)
            }
            _ => None,
        }
    }

    pub fn handle_start_session(&mut self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorStartSessionRequest>(input, API_SMALL_REQUEST_BYTES_MAX)?;
        let identity = self.validate_identity(
            &parsed.value.api_profile,
            &parsed.value.semantic_context,
            &parsed.value.selection,
        )?;
        let legacy_request = StartSessionRequest::new(parsed.value.module_name.clone())
            .with_proof_profile(parsed.value.proof_profile);
        let preview = self
            .proof_service
            .preview_start_session(&legacy_request)
            .map_err(map_session_error)?;
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation("session_id")?;
        let response = SuccessorStartSessionResponse {
            api_profile: SUCCESSOR_AI_API_PROFILE.to_owned(),
            session_id: preview.session_id.clone(),
            module_name: preview.module_name.clone(),
            proof_profile: preview.proof_profile,
            status: preview.status,
            semantic_context: identity.semantic_context.clone(),
            selection: identity.selection.clone(),
            helper_only: true,
        };
        let response_bytes = encode_response(&response)?;
        let committed = self
            .proof_service
            .start_session(legacy_request)
            .map_err(map_session_error)?;
        debug_assert_eq!(committed, preview);
        self.sessions.insert(
            committed.session_id,
            SuccessorSessionBinding {
                identity,
                source: SuccessorSessionSourceState::Empty,
            },
        );
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    pub fn handle_vir_import(&mut self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed = parse_request::<SuccessorVirImportRequest>(input, VIR_INPUT_JSON_BYTES_MAX)?;
        validate_api_profile(&parsed.value.api_profile)?;
        if parsed.value.source_ir_schema != SUCCESSOR_VIR_SCHEMA {
            return Err(SuccessorApiError::new(
                SuccessorValidationPhase::Shape,
                SuccessorApiErrorCode::VirSchema,
                Some("source_ir_schema"),
                None::<String>,
            ));
        }
        validate_session_id(parsed.value.session_id.as_str())?;
        validate_sha256(&parsed.value.source_ir_hash, "source_ir_hash")?;
        let identity = self.validate_bound_identity(
            &parsed.value.api_profile,
            &parsed.value.session_id,
            &parsed.value.semantic_context,
            &parsed.value.selection,
        )?;
        if !matches!(
            self.sessions
                .get(&parsed.value.session_id)
                .map(|session| &session.source),
            Some(SuccessorSessionSourceState::Empty)
        ) {
            return Err(session_state("session_id"));
        }
        let vir_bytes = canonical_embedded_value(&parsed.value.vir, VIR_INPUT_JSON_BYTES_MAX)?;
        let vir = mpk_vc::successor_source_artifacts::import_successor_vir_json(
            &vir_bytes,
            &self.registry,
        )
        .map_err(|error| vir_invalid(error.code().as_str()))?;
        if vir.module().schema() != parsed.value.source_ir_schema
            || vir.hash().as_str() != parsed.value.source_ir_hash
        {
            return Err(SuccessorApiError::new(
                SuccessorValidationPhase::Artifact,
                SuccessorApiErrorCode::VirHash,
                Some("source_ir_hash"),
                None::<String>,
            ));
        }
        validate_semantic_context_linkage(
            identity.semantic_context(),
            vir.module().semantic_context(),
        )
        .map_err(|_| context_mismatch("semantic_context"))?;
        let unit_count = checked_count(vir.module().units().len(), "vir")?;
        let function_count = vir.module().units().iter().try_fold(0_u64, |count, unit| {
            count
                .checked_add(u64::try_from(unit.functions().len()).unwrap_or(u64::MAX))
                .ok_or_else(|| vir_invalid("VIR_LIMIT_FUNCTIONS"))
        })?;
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation("session_id")?;
        let response = SuccessorVirImportResponse {
            api_profile: SUCCESSOR_AI_API_PROFILE.to_owned(),
            session_id: parsed.value.session_id.clone(),
            semantic_context: identity.semantic_context.clone(),
            selection: identity.selection.clone(),
            source_ir_schema: SUCCESSOR_VIR_SCHEMA.to_owned(),
            source_ir_hash: vir.hash().as_str().to_owned(),
            unit_count,
            function_count,
            helper_only: true,
        };
        let response_bytes = encode_response(&response)?;
        let session = self
            .sessions
            .get_mut(&parsed.value.session_id)
            .ok_or_else(|| unknown_session("session_id"))?;
        session.source =
            SuccessorSessionSourceState::VirImported(Box::new(ImportedSuccessorVir { vir }));
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    pub fn handle_vc_generate(&mut self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorVcGenerateRequest>(input, VC_CANONICAL_JSON_BYTES_MAX)?;
        let prepared = self.validate_vc_generate(&parsed.value)?;
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation("session_id")?;
        let response_bytes = encode_response(&prepared.response)?;
        let session = self
            .sessions
            .get_mut(&parsed.value.session_id)
            .ok_or_else(|| unknown_session("session_id"))?;
        session.source = SuccessorSessionSourceState::VcGenerated(Box::new(prepared.state));
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    fn validate_vc_generate(
        &self,
        request: &SuccessorVcGenerateRequest,
    ) -> Result<PreparedSuccessorVcGeneration, SuccessorApiError> {
        validate_api_profile(&request.api_profile)?;
        validate_generate_scalars(request)?;
        let identity = self.validate_bound_identity(
            &request.api_profile,
            &request.session_id,
            &request.semantic_context,
            &request.selection,
        )?;
        let imported = match &self
            .sessions
            .get(&request.session_id)
            .ok_or_else(|| unknown_session("session_id"))?
            .source
        {
            SuccessorSessionSourceState::VirImported(imported) => imported,
            _ => return Err(session_state("session_id")),
        };
        if request.source_ir_hash != imported.vir.hash().as_str() {
            return Err(context_mismatch("source_ir_hash"));
        }
        let record = self
            .store
            .get(&request.source_manifest_hash)
            .ok_or_else(|| {
                SuccessorApiError::new(
                    SuccessorValidationPhase::Artifact,
                    SuccessorApiErrorCode::SourceContextUnknown,
                    Some("source_manifest_hash"),
                    None::<String>,
                )
            })?;
        let manifest = record.source_manifest.manifest();
        if request.input_set_hash != manifest.input_set_hash().as_str() {
            return Err(SuccessorApiError::new(
                SuccessorValidationPhase::Artifact,
                SuccessorApiErrorCode::SourceManifestHash,
                Some("input_set_hash"),
                None::<String>,
            ));
        }
        if record.vir.canonical_bytes() != imported.vir.canonical_bytes()
            || manifest.semantic_context() != identity.semantic_context()
            || manifest.selection() != identity.selection()
            || record.source_map.map().semantic_context() != identity.semantic_context()
            || record.compiled_vc_profile_contract.profile_entry_sha256()
                != identity.semantic_context().profile_entry_sha256()
        {
            return Err(context_mismatch("semantic_context"));
        }
        let vc = generate_successor_vc(SuccessorVcSource {
            registry: &self.registry,
            vir: &record.vir,
            manifest: &record.source_manifest,
            profile_contract: &record.vc_profile_contract,
        })
        .map_err(|error| vc_invalid(error.code().as_str()))?;
        let document = vc.document();
        if document.schema() != SUCCESSOR_VC_SCHEMA
            || document.source_ir_schema() != request.source_ir_schema
            || document.source_ir_hash().as_str() != request.source_ir_hash
            || document.source_manifest_schema() != request.source_manifest_schema
            || document.source_manifest_hash().as_str() != request.source_manifest_hash
            || document.input_set_hash().as_str() != request.input_set_hash
            || document.semantic_context() != identity.semantic_context()
        {
            return Err(vc_invalid("VC_SOURCE_LINKAGE"));
        }
        let (function_count, member_count, group_count) = vc_counts(document.functions())?;
        let vc_value =
            serde_json::from_slice(vc.canonical_bytes()).map_err(|_| vc_invalid("VC_SHAPE"))?;
        let response = SuccessorVcGenerateResponse {
            context: SuccessorVcContextResponse::from_parts(
                request.session_id.clone(),
                identity,
                document,
            ),
            function_count,
            member_count,
            group_count,
            helper_only: true,
            vc: vc_value,
        };
        let state = SuccessorVcSessionState {
            imported: imported.as_ref().clone(),
            source_manifest_hash: request.source_manifest_hash.clone(),
            input_set_hash: request.input_set_hash.clone(),
            vc,
            targets: BTreeMap::new(),
            next_target_index: 0,
        };
        Ok(PreparedSuccessorVcGeneration { state, response })
    }

    pub fn handle_vc_list(&self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorVcContextRequest>(input, API_SMALL_REQUEST_BYTES_MAX)?;
        let state = self.validate_vc_context(&parsed.value)?;
        parsed.require_canonical()?;
        let members = state
            .vc
            .document()
            .functions()
            .iter()
            .flat_map(|function| {
                function.members.iter().map(|member| VcMemberSummary {
                    member_id: member.id.clone(),
                    function_id: member.function_id.clone(),
                    kind: member.kind,
                    group_id: member.group_id.clone(),
                })
            })
            .collect();
        encode_response(&SuccessorVcListResponse {
            context: self.context_response(&parsed.value.session_id, state)?,
            members,
            helper_only: true,
        })
    }

    pub fn handle_vc_start_proof(&mut self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorVcStartProofRequest>(input, API_SMALL_REQUEST_BYTES_MAX)?;
        let (target_term, next_index, context) = {
            let state = self.validate_vc_context(&parsed.value.context)?;
            let term = resolve_target_term(state.vc.document().functions(), &parsed.value.target)
                .map_err(|_| target_unknown("target"))?;
            let next = state
                .next_target_index
                .checked_add(1)
                .ok_or_else(|| session_state("target"))?;
            let context = self.context_response(&parsed.value.context.session_id, state)?;
            (term, next, context)
        };
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation("target")?;
        let target_id = format!("t{next_index}");
        let response = SuccessorVcStartProofResponse {
            context,
            target: parsed.value.target.clone(),
            target_id: target_id.clone(),
            helper_only: true,
        };
        let response_bytes = encode_response(&response)?;
        let proof_session = self
            .proof_service
            .session_mut(&parsed.value.context.session_id)
            .ok_or_else(|| unknown_session("session_id"))?;
        let target_term_id =
            materialize_target(proof_session, &target_term).map_err(|_| vc_invalid("VC_TERM"))?;
        let state = match self
            .sessions
            .get_mut(&parsed.value.context.session_id)
            .map(|session| &mut session.source)
        {
            Some(SuccessorSessionSourceState::VcGenerated(state)) => state,
            _ => return Err(session_state("session_id")),
        };
        state.next_target_index = next_index;
        state.targets.insert(
            target_id,
            SuccessorProofTargetRecord {
                target: parsed.value.target,
                target_term: target_term_id,
                candidates: BTreeMap::new(),
            },
        );
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    pub fn handle_vc_attach_candidate(
        &mut self,
        input: &[u8],
    ) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorVcAttachCandidateRequest>(input, API_SMALL_REQUEST_BYTES_MAX)?;
        validate_target_id(&parsed.value.target_id)?;
        validate_candidate_id(&parsed.value.candidate_id)?;
        let context = {
            let state = self.validate_vc_context(&parsed.value.context)?;
            let target = state
                .targets
                .get(&parsed.value.target_id)
                .ok_or_else(|| target_unknown("target_id"))?;
            if target.candidates.contains_key(&parsed.value.candidate_id) {
                return Err(context_mismatch("candidate_id"));
            }
            self.context_response(&parsed.value.context.session_id, state)?
        };
        if self
            .proof_service
            .session(&parsed.value.context.session_id)
            .and_then(|session| session.proof_node(parsed.value.proof_root))
            .is_none()
        {
            return Err(SuccessorApiError::new(
                SuccessorValidationPhase::Context,
                SuccessorApiErrorCode::UnknownProof,
                Some("proof_root"),
                None::<String>,
            ));
        }
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation("candidate_id")?;
        let response = SuccessorVcAttachCandidateResponse {
            context,
            target_id: parsed.value.target_id.clone(),
            candidate_id: parsed.value.candidate_id.clone(),
            proof_root: parsed.value.proof_root,
            helper_only: true,
        };
        let response_bytes = encode_response(&response)?;
        let state = self.require_vc_state_mut(&parsed.value.context.session_id)?;
        let target = state
            .targets
            .get_mut(&parsed.value.target_id)
            .ok_or_else(|| target_unknown("target_id"))?;
        target
            .candidates
            .insert(parsed.value.candidate_id, parsed.value.proof_root);
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    pub fn handle_vc_check_candidate(&self, input: &[u8]) -> Result<Vec<u8>, SuccessorApiError> {
        let parsed =
            parse_request::<SuccessorVcCheckCandidateRequest>(input, API_SMALL_REQUEST_BYTES_MAX)?;
        validate_target_id(&parsed.value.target_id)?;
        if parsed.value.mode != CHECK_MODE || parsed.value.candidates.is_empty() {
            return Err(scalar("mode"));
        }
        let state = self.validate_vc_context(&parsed.value.context)?;
        let target = state
            .targets
            .get(&parsed.value.target_id)
            .ok_or_else(|| target_unknown("target_id"))?;
        let mut seen = BTreeSet::new();
        for candidate in &parsed.value.candidates {
            validate_candidate_id(&candidate.candidate_id)?;
            if !seen.insert(candidate.candidate_id.as_str()) {
                return Err(context_mismatch("candidates"));
            }
            match target.candidates.get(&candidate.candidate_id) {
                Some(root) if *root == candidate.proof_root => {}
                Some(_) => return Err(context_mismatch("proof_root")),
                None => {
                    return Err(SuccessorApiError::new(
                        SuccessorValidationPhase::Context,
                        SuccessorApiErrorCode::CandidateUnknown,
                        Some("candidate_id"),
                        None::<String>,
                    ))
                }
            }
        }
        parsed.require_canonical()?;
        let session = self
            .proof_service
            .session(&parsed.value.context.session_id)
            .ok_or_else(|| unknown_session("session_id"))?;
        let results = parsed
            .value
            .candidates
            .iter()
            .map(|candidate| {
                let diagnostic =
                    check_candidate_root(session, candidate.proof_root, target.target_term);
                VcCandidateResult {
                    candidate_id: candidate.candidate_id.clone(),
                    proof_root: candidate.proof_root,
                    helper_status: if diagnostic.is_none() {
                        HelperStatus::Valid
                    } else {
                        HelperStatus::Invalid
                    },
                    diagnostic,
                }
            })
            .collect();
        encode_response(&SuccessorVcCheckCandidateResponse {
            context: self.context_response(&parsed.value.context.session_id, state)?,
            target_id: parsed.value.target_id,
            mode: parsed.value.mode,
            results,
            helper_only: true,
        })
    }

    fn validate_identity(
        &self,
        api_profile: &str,
        semantic_context: &Value,
        selection: &Value,
    ) -> Result<SuccessorSessionIdentity, SuccessorApiError> {
        validate_api_profile(api_profile)?;
        self.validate_semantic_identity(semantic_context, selection)
    }

    fn validate_bound_identity(
        &self,
        api_profile: &str,
        session_id: &SessionId,
        semantic_context: &Value,
        selection: &Value,
    ) -> Result<&SuccessorSessionIdentity, SuccessorApiError> {
        validate_api_profile(api_profile)?;
        validate_session_id(session_id.as_str())?;
        let bound = self
            .sessions
            .get(session_id)
            .ok_or_else(|| unknown_session("session_id"))?;
        if self.proof_service.session(session_id).is_none() {
            return Err(unknown_session("session_id"));
        }
        let requested = self.validate_semantic_identity(semantic_context, selection)?;
        if bound.identity != requested {
            return Err(context_mismatch("semantic_context"));
        }
        Ok(&bound.identity)
    }

    fn validate_vc_context(
        &self,
        request: &SuccessorVcContextRequest,
    ) -> Result<&SuccessorVcSessionState, SuccessorApiError> {
        validate_api_profile(&request.api_profile)?;
        validate_vc_context_scalars(request)?;
        self.validate_bound_identity(
            &request.api_profile,
            &request.session_id,
            &request.semantic_context,
            &request.selection,
        )?;
        let state = match &self
            .sessions
            .get(&request.session_id)
            .ok_or_else(|| unknown_session("session_id"))?
            .source
        {
            SuccessorSessionSourceState::VcGenerated(state) => state,
            _ => return Err(session_state("session_id")),
        };
        if request.source_ir_hash != state.imported.vir.hash().as_str()
            || request.source_manifest_hash != state.source_manifest_hash
            || request.input_set_hash != state.input_set_hash
            || request.vc_hash != state.vc.hash().as_str()
        {
            return Err(context_mismatch("source_ir_hash"));
        }
        Ok(state)
    }

    fn context_response(
        &self,
        session_id: &SessionId,
        state: &SuccessorVcSessionState,
    ) -> Result<SuccessorVcContextResponse, SuccessorApiError> {
        let identity = self
            .sessions
            .get(session_id)
            .map(|session| &session.identity)
            .ok_or_else(|| unknown_session("session_id"))?;
        Ok(SuccessorVcContextResponse::from_parts(
            session_id.clone(),
            identity,
            state.vc.document(),
        ))
    }

    fn require_vc_state_mut(
        &mut self,
        session_id: &SessionId,
    ) -> Result<&mut SuccessorVcSessionState, SuccessorApiError> {
        match self
            .sessions
            .get_mut(session_id)
            .map(|session| &mut session.source)
        {
            Some(SuccessorSessionSourceState::VcGenerated(state)) => Ok(state),
            _ => Err(session_state("session_id")),
        }
    }

    fn next_mutation(&self, field: &'static str) -> Result<u64, SuccessorApiError> {
        self.mutation_count
            .checked_add(1)
            .ok_or_else(|| session_state(field))
    }

    fn validate_semantic_identity(
        &self,
        semantic_context: &Value,
        selection: &Value,
    ) -> Result<SuccessorSessionIdentity, SuccessorApiError> {
        let semantic_context = validate_registry_semantic_context(&self.registry, semantic_context)
            .map_err(|_| context_mismatch("semantic_context"))?;
        let selection =
            validate_registry_selection_envelope(&self.registry, &semantic_context, selection)
                .map_err(|_| context_mismatch("selection"))?;
        Ok(SuccessorSessionIdentity {
            semantic_context,
            selection,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorStartSessionRequest {
    pub api_profile: String,
    pub module_name: String,
    pub proof_profile: ProofProfile,
    pub semantic_context: Value,
    pub selection: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorStartSessionResponse {
    pub api_profile: String,
    pub session_id: SessionId,
    pub module_name: String,
    pub proof_profile: ProofProfile,
    pub status: SessionStatus,
    pub semantic_context: SemanticContext,
    pub selection: SelectionEnvelope,
    pub helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVirImportRequest {
    pub api_profile: String,
    pub session_id: SessionId,
    pub semantic_context: Value,
    pub selection: Value,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub vir: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVirImportResponse {
    pub api_profile: String,
    pub session_id: SessionId,
    pub semantic_context: SemanticContext,
    pub selection: SelectionEnvelope,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub unit_count: u64,
    pub function_count: u64,
    pub helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcGenerateRequest {
    pub api_profile: String,
    pub session_id: SessionId,
    pub semantic_context: Value,
    pub selection: Value,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub source_manifest_schema: String,
    pub source_manifest_hash: String,
    pub input_set_hash: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcGenerateResponse {
    #[serde(flatten)]
    pub context: SuccessorVcContextResponse,
    pub function_count: u64,
    pub member_count: u64,
    pub group_count: u64,
    pub helper_only: bool,
    pub vc: Value,
}

struct PreparedSuccessorVcGeneration {
    state: SuccessorVcSessionState,
    response: SuccessorVcGenerateResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcContextRequest {
    pub api_profile: String,
    pub session_id: SessionId,
    pub semantic_context: Value,
    pub selection: Value,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub source_manifest_schema: String,
    pub source_manifest_hash: String,
    pub input_set_hash: String,
    pub source_vc_schema: String,
    pub vc_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVcContextResponse {
    pub api_profile: String,
    pub session_id: SessionId,
    pub semantic_context: SemanticContext,
    pub selection: SelectionEnvelope,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub source_manifest_schema: String,
    pub source_manifest_hash: String,
    pub input_set_hash: String,
    pub source_vc_schema: String,
    pub vc_hash: String,
}

impl SuccessorVcContextResponse {
    fn from_parts(
        session_id: SessionId,
        identity: &SuccessorSessionIdentity,
        document: &mpk_vc::successor_vc::SuccessorVcDocument,
    ) -> Self {
        Self {
            api_profile: SUCCESSOR_AI_API_PROFILE.to_owned(),
            session_id,
            semantic_context: identity.semantic_context.clone(),
            selection: identity.selection.clone(),
            source_ir_schema: document.source_ir_schema().to_owned(),
            source_ir_hash: document.source_ir_hash().as_str().to_owned(),
            source_manifest_schema: document.source_manifest_schema().to_owned(),
            source_manifest_hash: document.source_manifest_hash().as_str().to_owned(),
            input_set_hash: document.input_set_hash().as_str().to_owned(),
            source_vc_schema: document.schema().to_owned(),
            vc_hash: document.vc_hash().as_str().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcListResponse {
    #[serde(flatten)]
    pub context: SuccessorVcContextResponse,
    pub members: Vec<VcMemberSummary>,
    pub helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcStartProofRequest {
    #[serde(flatten)]
    pub context: SuccessorVcContextRequest,
    pub target: VcProofTarget,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcStartProofResponse {
    #[serde(flatten)]
    pub context: SuccessorVcContextResponse,
    pub target: VcProofTarget,
    pub target_id: String,
    pub helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcAttachCandidateRequest {
    #[serde(flatten)]
    pub context: SuccessorVcContextRequest,
    pub target_id: String,
    pub candidate_id: String,
    pub proof_root: ApiProofId,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcAttachCandidateResponse {
    #[serde(flatten)]
    pub context: SuccessorVcContextResponse,
    pub target_id: String,
    pub candidate_id: String,
    pub proof_root: ApiProofId,
    pub helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcCheckCandidateRequest {
    #[serde(flatten)]
    pub context: SuccessorVcContextRequest,
    pub target_id: String,
    pub mode: String,
    pub candidates: Vec<VcCandidateBinding>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorVcCheckCandidateResponse {
    #[serde(flatten)]
    pub context: SuccessorVcContextResponse,
    pub target_id: String,
    pub mode: String,
    pub results: Vec<VcCandidateResult>,
    pub helper_only: bool,
}

struct ParsedRequest<T> {
    value: T,
    canonical_transport: bool,
}

impl<T> ParsedRequest<T> {
    fn require_canonical(&self) -> Result<(), SuccessorApiError> {
        if self.canonical_transport {
            Ok(())
        } else {
            Err(SuccessorApiError::new(
                SuccessorValidationPhase::CanonicalTransport,
                SuccessorApiErrorCode::CanonicalTransport,
                None,
                None::<String>,
            ))
        }
    }
}

fn parse_request<T: DeserializeOwned>(
    input: &[u8],
    embedded_bytes_max: u64,
) -> Result<ParsedRequest<T>, SuccessorApiError> {
    let transport_max = embedded_bytes_max
        .checked_add(API_ENVELOPE_OVERHEAD)
        .ok_or_else(json_invalid)?;
    if u64::try_from(input.len()).unwrap_or(u64::MAX) > transport_max
        || !input.ends_with(b"\n")
        || input.starts_with(&[0xef, 0xbb, 0xbf])
    {
        return Err(json_invalid());
    }
    let body = &input[..input.len() - 1];
    let strict = parse_strict_json(
        body,
        StrictJsonLimits::new(
            transport_max,
            transport_max,
            API_WRAPPER_LEVELS,
            API_STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| json_invalid())?;
    let canonical = canonical_json_bytes(&strict).map_err(|_| json_invalid())?;
    let value = serde_json::from_slice(&canonical).map_err(|_| {
        SuccessorApiError::new(
            SuccessorValidationPhase::Shape,
            SuccessorApiErrorCode::Shape,
            None,
            None::<String>,
        )
    })?;
    let canonical_transport = input.len() == canonical.len() + 1
        && input[..canonical.len()] == canonical
        && input[canonical.len()] == b'\n';
    Ok(ParsedRequest {
        value,
        canonical_transport,
    })
}

fn canonical_embedded_value(value: &Value, maximum: u64) -> Result<Vec<u8>, SuccessorApiError> {
    let serialized = serde_json::to_vec(value).map_err(|_| vir_invalid("VIR_SHAPE"))?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(maximum, maximum, API_WRAPPER_LEVELS, API_STRING_BYTES_MAX),
    )
    .map_err(|_| vir_invalid("VIR_JSON_INVALID"))?;
    canonical_json_bytes(&strict).map_err(|_| vir_invalid("VIR_CANONICAL"))
}

fn encode_response<T: Serialize>(response: &T) -> Result<Vec<u8>, SuccessorApiError> {
    let serialized = serde_json::to_vec(response).map_err(|_| vc_invalid("RESPONSE_SHAPE"))?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(
            API_TRANSPORT_BYTES_MAX,
            API_TRANSPORT_BYTES_MAX,
            API_WRAPPER_LEVELS,
            API_STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| json_invalid())?;
    let mut canonical = canonical_json_bytes(&strict).map_err(|_| json_invalid())?;
    canonical.push(b'\n');
    Ok(canonical)
}

fn validate_api_profile(value: &str) -> Result<(), SuccessorApiError> {
    if value == SUCCESSOR_AI_API_PROFILE {
        Ok(())
    } else {
        Err(SuccessorApiError::new(
            SuccessorValidationPhase::Route,
            SuccessorApiErrorCode::ApiProfile,
            Some("api_profile"),
            None::<String>,
        ))
    }
}

fn validate_session_id(value: &str) -> Result<(), SuccessorApiError> {
    let valid = value.strip_prefix('s').is_some_and(|digits| {
        !digits.is_empty()
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok_and(|number| number > 0)
    });
    if valid {
        Ok(())
    } else {
        Err(scalar("session_id"))
    }
}

fn validate_sha256(value: &str, field: &'static str) -> Result<(), SuccessorApiError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(scalar(field))
    }
}

fn validate_generate_scalars(
    request: &SuccessorVcGenerateRequest,
) -> Result<(), SuccessorApiError> {
    if request.source_ir_schema != SUCCESSOR_VIR_SCHEMA {
        return Err(SuccessorApiError::new(
            SuccessorValidationPhase::Shape,
            SuccessorApiErrorCode::VirSchema,
            Some("source_ir_schema"),
            None::<String>,
        ));
    }
    if request.source_manifest_schema != SUCCESSOR_SOURCE_MANIFEST_SCHEMA {
        return Err(SuccessorApiError::new(
            SuccessorValidationPhase::Shape,
            SuccessorApiErrorCode::SourceManifestSchema,
            Some("source_manifest_schema"),
            None::<String>,
        ));
    }
    validate_session_id(request.session_id.as_str())?;
    validate_sha256(&request.source_ir_hash, "source_ir_hash")?;
    validate_sha256(&request.source_manifest_hash, "source_manifest_hash")?;
    validate_sha256(&request.input_set_hash, "input_set_hash")
}

fn validate_vc_context_scalars(
    request: &SuccessorVcContextRequest,
) -> Result<(), SuccessorApiError> {
    if request.source_ir_schema != SUCCESSOR_VIR_SCHEMA
        || request.source_manifest_schema != SUCCESSOR_SOURCE_MANIFEST_SCHEMA
        || request.source_vc_schema != SUCCESSOR_VC_SCHEMA
    {
        return Err(context_mismatch("source_ir_schema"));
    }
    validate_session_id(request.session_id.as_str())?;
    validate_sha256(&request.source_ir_hash, "source_ir_hash")?;
    validate_sha256(&request.source_manifest_hash, "source_manifest_hash")?;
    validate_sha256(&request.input_set_hash, "input_set_hash")?;
    validate_sha256(&request.vc_hash, "vc_hash")
}

fn vc_counts(functions: &[VcFunction]) -> Result<(u64, u64, u64), SuccessorApiError> {
    let function_count = u64::try_from(functions.len()).map_err(|_| vc_invalid("VC_LIMIT"))?;
    let (member_count, group_count) =
        functions
            .iter()
            .try_fold((0_u64, 0_u64), |(members, groups), function| {
                Ok::<_, SuccessorApiError>((
                    members
                        .checked_add(u64::try_from(function.members.len()).unwrap_or(u64::MAX))
                        .ok_or_else(|| vc_invalid("VC_LIMIT"))?,
                    groups
                        .checked_add(u64::try_from(function.groups.len()).unwrap_or(u64::MAX))
                        .ok_or_else(|| vc_invalid("VC_LIMIT"))?,
                ))
            })?;
    Ok((function_count, member_count, group_count))
}

fn checked_count(count: usize, field: &'static str) -> Result<u64, SuccessorApiError> {
    u64::try_from(count).map_err(|_| {
        SuccessorApiError::new(
            SuccessorValidationPhase::Artifact,
            SuccessorApiErrorCode::VirInvalid,
            Some(field),
            Some("VIR_LIMIT_COUNT"),
        )
    })
}

fn validate_candidate_id(candidate_id: &str) -> Result<(), SuccessorApiError> {
    if !candidate_id.is_empty()
        && candidate_id.len() <= 256
        && candidate_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
    {
        Ok(())
    } else {
        Err(scalar("candidate_id"))
    }
}

fn validate_target_id(target_id: &str) -> Result<(), SuccessorApiError> {
    let valid = target_id.strip_prefix('t').is_some_and(|digits| {
        !digits.is_empty()
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok_and(|number| number > 0)
    });
    if valid {
        Ok(())
    } else {
        Err(scalar("target_id"))
    }
}

fn map_session_error(error: crate::session::ApiError) -> SuccessorApiError {
    match error.code {
        crate::session::ApiErrorCode::InvalidModuleName => scalar("module_name"),
        crate::session::ApiErrorCode::SessionLimitExceeded => session_state("session_id"),
        _ => session_state("session_id"),
    }
}

fn json_invalid() -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Transport,
        SuccessorApiErrorCode::JsonInvalid,
        None,
        None::<String>,
    )
}

fn scalar(field: &'static str) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Scalar,
        SuccessorApiErrorCode::Scalar,
        Some(field),
        None::<String>,
    )
}

fn unknown_session(field: &'static str) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Session,
        SuccessorApiErrorCode::UnknownSession,
        Some(field),
        None::<String>,
    )
}

fn session_state(field: &'static str) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Session,
        SuccessorApiErrorCode::SessionState,
        Some(field),
        None::<String>,
    )
}

fn context_mismatch(field: &'static str) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Context,
        SuccessorApiErrorCode::ContextMismatch,
        Some(field),
        None::<String>,
    )
}

fn target_unknown(field: &'static str) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Context,
        SuccessorApiErrorCode::TargetUnknown,
        Some(field),
        None::<String>,
    )
}

fn vir_invalid(detail: impl Into<String>) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Artifact,
        SuccessorApiErrorCode::VirInvalid,
        Some("vir"),
        Some(detail),
    )
}

fn source_manifest_invalid(detail: impl Into<String>) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Artifact,
        SuccessorApiErrorCode::SourceManifestInvalid,
        None,
        Some(detail),
    )
}

fn vc_invalid(detail: impl Into<String>) -> SuccessorApiError {
    SuccessorApiError::new(
        SuccessorValidationPhase::Artifact,
        SuccessorApiErrorCode::VcInvalid,
        None,
        Some(detail),
    )
}
