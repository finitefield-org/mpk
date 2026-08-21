//! Staged VIR import state for AI API v1.

use std::collections::BTreeMap;

use mpk_vc::{
    canonical_vir_json, import_vir_json, validate_vir, SourceManifestStage,
    ValidatedSourceManifest, ValidatedSourceMap, VirImportError, VirModule,
    VIR_INPUT_JSON_BYTES_MAX, VIR_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::session::{ApiService, SessionId, StartSessionRequest, StartSessionResponse};
use crate::v1_router::{
    encode_response, parse_request, validate_session_id, validate_sha256, ParsedRequest,
    V1ApiError, V1ErrorCode, V1ValidationPhase,
};

#[derive(Clone, Debug)]
pub(crate) struct ValidatedFrontendArtifacts {
    pub(crate) vir: VirModule,
    pub(crate) source_map: ValidatedSourceMap,
    pub(crate) source_manifest: ValidatedSourceManifest,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedFrontendArtifactRecord {
    pub(crate) vir: VirModule,
    pub(crate) vir_bytes: Vec<u8>,
    pub(crate) source_map: ValidatedSourceMap,
    pub(crate) source_manifest: ValidatedSourceManifest,
}

impl ValidatedFrontendArtifactRecord {
    fn from_frontend_success(artifacts: ValidatedFrontendArtifacts) -> Result<Self, V1ApiError> {
        validate_vir(&artifacts.vir).map_err(|error| source_manifest_invalid(error.code()))?;
        let vir_bytes = canonical_vir_json(&artifacts.vir)
            .map_err(|_| source_manifest_invalid("VIR_CANONICAL"))?;
        let manifest = artifacts.source_manifest.manifest();
        if artifacts.source_manifest.stage() != SourceManifestStage::Frontend
            || manifest.vc_hash.is_some()
            || manifest.vir_hash != artifacts.vir.vir_hash.as_str()
            || manifest.source_map_hash != artifacts.source_map.hash().as_str()
            || artifacts.source_map.map().source_ir_schema != VIR_SCHEMA_VERSION
            || artifacts.source_map.map().source_ir_hash != artifacts.vir.vir_hash.as_str()
        {
            return Err(source_manifest_invalid("SOURCE_MANIFEST_ARTIFACT_LINKAGE"));
        }
        Ok(Self {
            vir: artifacts.vir,
            vir_bytes,
            source_map: artifacts.source_map,
            source_manifest: artifacts.source_manifest,
        })
    }

    fn same_capability(&self, other: &Self) -> bool {
        self.vir_bytes == other.vir_bytes
            && self.source_map.canonical_bytes() == other.source_map.canonical_bytes()
            && self.source_manifest.canonical_bytes() == other.source_manifest.canonical_bytes()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ValidatedFrontendArtifactStore {
    records: BTreeMap<String, ValidatedFrontendArtifactRecord>,
}

impl ValidatedFrontendArtifactStore {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    /// Builds the immutable store only from capabilities produced by the
    /// successful frontend protocol path. Request JSON has no access to this
    /// constructor or to a record insertion operation.
    pub(crate) fn from_frontend_successes(
        artifacts: impl IntoIterator<Item = ValidatedFrontendArtifacts>,
    ) -> Result<Self, V1ApiError> {
        let mut records = BTreeMap::<String, ValidatedFrontendArtifactRecord>::new();
        for artifacts in artifacts {
            let record = ValidatedFrontendArtifactRecord::from_frontend_success(artifacts)?;
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
        Ok(Self { records })
    }

    pub(crate) fn get(&self, manifest_hash: &str) -> Option<&ValidatedFrontendArtifactRecord> {
        self.records.get(manifest_hash)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ImportedVir {
    pub(crate) module: VirModule,
    pub(crate) canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(crate) enum SessionSourceState {
    Empty,
    VirImported(ImportedVir),
    VcGenerated(Box<crate::vc_api::VcSessionState>),
}

impl SessionSourceState {
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::VirImported(_) => "vir_imported",
            Self::VcGenerated(_) => "vc_generated",
        }
    }
}

#[derive(Debug)]
pub(crate) struct V1ApiService {
    pub(crate) legacy: ApiService,
    pub(crate) store: ValidatedFrontendArtifactStore,
    pub(crate) sources: BTreeMap<SessionId, SessionSourceState>,
    mutation_count: u64,
}

impl V1ApiService {
    pub(crate) fn new(store: ValidatedFrontendArtifactStore) -> Self {
        Self {
            legacy: ApiService::new(),
            store,
            sources: BTreeMap::new(),
            mutation_count: 0,
        }
    }

    pub(crate) fn start_session(
        &mut self,
        request: StartSessionRequest,
    ) -> Result<StartSessionResponse, crate::session::ApiError> {
        let response = self.legacy.start_session(request)?;
        self.sources
            .insert(response.session_id.clone(), SessionSourceState::Empty);
        Ok(response)
    }

    pub(crate) const fn mutation_count(&self) -> u64 {
        self.mutation_count
    }

    pub(crate) fn commit_mutation_count(&mut self, mutation_count: u64) {
        self.mutation_count = mutation_count;
    }

    pub(crate) fn source_state(&self, session_id: &SessionId) -> Option<&SessionSourceState> {
        self.sources.get(session_id)
    }

    pub(crate) fn handle_vir_import(&mut self, input: &[u8]) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VirImportRequest>(input, VIR_INPUT_JSON_BYTES_MAX)?;
        let response = self.validate_vir_import(&parsed)?;
        parsed.require_canonical()?;
        let next_mutation = self.mutation_count.checked_add(1).ok_or_else(|| {
            V1ApiError::new(
                V1ValidationPhase::Session,
                V1ErrorCode::SessionState,
                None,
                None::<String>,
            )
        })?;
        let response_bytes = encode_response(&response.response)?;
        let state = ImportedVir {
            module: response.validated,
            canonical_bytes: response.vir_bytes,
        };
        self.sources.insert(
            parsed.value.session_id.clone(),
            SessionSourceState::VirImported(state),
        );
        self.mutation_count = next_mutation;
        Ok(response_bytes)
    }

    fn validate_vir_import(
        &self,
        parsed: &ParsedRequest<VirImportRequest>,
    ) -> Result<PreparedVirImport, V1ApiError> {
        let request = &parsed.value;
        if request.source_ir_schema != VIR_SCHEMA_VERSION {
            return Err(V1ApiError::new(
                V1ValidationPhase::Shape,
                V1ErrorCode::VirSchema,
                Some("source_ir_schema"),
                None::<String>,
            ));
        }
        validate_session_id(request.session_id.as_str())?;
        validate_sha256(&request.source_ir_hash, "source_ir_hash")?;
        if self.legacy.session(&request.session_id).is_none() {
            return Err(V1ApiError::inherited(
                V1ValidationPhase::Session,
                V1ErrorCode::UnknownSession,
                format!("API session {} does not exist", request.session_id),
                "session_id",
            ));
        }
        if !matches!(
            self.sources.get(&request.session_id),
            Some(SessionSourceState::Empty)
        ) {
            return Err(V1ApiError::new(
                V1ValidationPhase::Session,
                V1ErrorCode::SessionState,
                Some("session_id"),
                None::<String>,
            ));
        }

        let vir_bytes = canonical_value_bytes(&request.vir)?;
        let validated = import_vir_json(&vir_bytes).map_err(map_vir_import_error)?;
        if request.source_ir_hash != validated.vir_hash.as_str() {
            return Err(V1ApiError::new(
                V1ValidationPhase::Artifact,
                V1ErrorCode::VirHash,
                Some("source_ir_hash"),
                None::<String>,
            ));
        }
        let unit_count = checked_count(validated.units.len(), "vir")?;
        let function_count = validated.units.iter().try_fold(0_u64, |count, unit| {
            count
                .checked_add(u64::try_from(unit.functions.len()).unwrap_or(u64::MAX))
                .ok_or_else(|| {
                    V1ApiError::new(
                        V1ValidationPhase::Artifact,
                        V1ErrorCode::VirInvalid,
                        None,
                        Some("VIR_LIMIT_FUNCTIONS"),
                    )
                })
        })?;
        let response = VirImportResponse {
            session_id: request.session_id.clone(),
            source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
            source_ir_hash: validated.vir_hash.as_str().to_owned(),
            source_language: validated.source_language,
            semantic_profile: validated.semantic_profile,
            semantic_parameters: validated.semantic_parameters.clone(),
            unit_count,
            function_count,
            helper_only: true,
        };
        Ok(PreparedVirImport {
            validated,
            vir_bytes,
            response,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VirImportRequest {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) vir: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VirImportResponse {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) source_language: mpk_vc::SourceLanguage,
    pub(crate) semantic_profile: mpk_vc::SemanticProfile,
    pub(crate) semantic_parameters: mpk_vc::SemanticParameters,
    pub(crate) unit_count: u64,
    pub(crate) function_count: u64,
    pub(crate) helper_only: bool,
}

struct PreparedVirImport {
    validated: VirModule,
    vir_bytes: Vec<u8>,
    response: VirImportResponse,
}

fn canonical_value_bytes(value: &Value) -> Result<Vec<u8>, V1ApiError> {
    let serialized = serde_json::to_vec(value).map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Artifact,
            V1ErrorCode::VirInvalid,
            Some("vir"),
            Some("VIR_SHAPE"),
        )
    })?;
    let strict = mpk_vc::parse_strict_json(
        &serialized,
        mpk_vc::StrictJsonLimits::new(
            VIR_INPUT_JSON_BYTES_MAX,
            VIR_INPUT_JSON_BYTES_MAX,
            mpk_vc::VIR_JSON_NESTING_MAX,
            mpk_vc::VIR_STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Artifact,
            V1ErrorCode::VirInvalid,
            Some("vir"),
            Some("VIR_JSON_INVALID"),
        )
    })?;
    mpk_vc::canonical_json_bytes(&strict).map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Artifact,
            V1ErrorCode::VirInvalid,
            Some("vir"),
            Some("VIR_CANONICAL"),
        )
    })
}

fn map_vir_import_error(error: VirImportError) -> V1ApiError {
    let detail = match &error {
        VirImportError::StrictJson(_) => "VIR_JSON_INVALID",
        VirImportError::CanonicalJson(_) => "VIR_CANONICAL",
        VirImportError::InvalidShape(_) => "VIR_SHAPE",
        VirImportError::UnsupportedSchema { .. } => "VIR_SCHEMA",
        VirImportError::SemanticProfile(_) => "VIR_SEMANTIC_PROFILE",
        VirImportError::Validation(error) => error.code(),
        VirImportError::NonemptyModifies { .. } => "VIR_MODIFIES_NONEMPTY",
    };
    V1ApiError::new(
        V1ValidationPhase::Artifact,
        V1ErrorCode::VirInvalid,
        Some("vir"),
        Some(detail),
    )
}

fn checked_count(count: usize, field: &'static str) -> Result<u64, V1ApiError> {
    u64::try_from(count).map_err(|_| {
        V1ApiError::new(
            V1ValidationPhase::Artifact,
            V1ErrorCode::VirInvalid,
            Some(field),
            Some("VIR_LIMIT_COUNT"),
        )
    })
}

fn source_manifest_invalid(detail: impl Into<String>) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Artifact,
        V1ErrorCode::SourceManifestInvalid,
        None,
        Some(detail),
    )
}
