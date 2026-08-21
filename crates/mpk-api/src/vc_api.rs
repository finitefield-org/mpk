//! Staged VIR-bound VC workflow for AI API v1.

use std::collections::{BTreeMap, BTreeSet};

use mpk_cert::encode::ProofNode;
use mpk_core::{TermId, TermNode};
use mpk_vc::{
    canonical_vir_json, generate_vc_v1, group_body, member_theorem_type, ValidatedVcDocument,
    VcDocument, VcFunction, VcTerm, VcTypeTerm, VC_CANONICAL_JSON_BYTES_MAX, VC_SCHEMA_VERSION,
    VIR_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::diagnostics::RepairDiagnostic;
use crate::proof_api::ApiProofId;
use crate::session::{ApiSession, SessionId};
use crate::term_api::ApiTermId;
use crate::v1_router::{
    encode_response, parse_request, validate_session_id, validate_sha256, ParsedRequest,
    V1ApiError, V1ErrorCode, V1ValidationPhase,
};
use crate::vir_api::{ImportedVir, SessionSourceState, V1ApiService};

const SOURCE_MANIFEST_SCHEMA: &str = "mpk.source_manifest.v0";
const CHECK_MODE: &str = "fail_fast_per_candidate";

#[derive(Clone, Debug)]
pub(crate) struct VcSessionState {
    pub(crate) imported: ImportedVir,
    pub(crate) source_manifest_schema: String,
    pub(crate) frontend_source_manifest_hash: String,
    pub(crate) input_set_hash: String,
    pub(crate) vc: ValidatedVcDocument,
    pub(crate) targets: BTreeMap<String, ProofTargetRecord>,
    pub(crate) next_target_index: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ProofTargetRecord {
    pub(crate) target: VcProofTarget,
    pub(crate) target_term: ApiTermId,
    pub(crate) candidates: BTreeMap<String, ApiProofId>,
}

impl V1ApiService {
    pub(crate) fn handle_vc_generate(&mut self, input: &[u8]) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VcGenerateRequest>(input, VC_CANONICAL_JSON_BYTES_MAX)?;
        let prepared = self.validate_vc_generate(&parsed)?;
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation()?;
        let response = encode_response(&prepared.response)?;
        self.sources.insert(
            parsed.value.session_id.clone(),
            SessionSourceState::VcGenerated(Box::new(prepared.state)),
        );
        self.set_mutation_count(next_mutation);
        Ok(response)
    }

    fn validate_vc_generate(
        &self,
        parsed: &ParsedRequest<VcGenerateRequest>,
    ) -> Result<PreparedVcGeneration, V1ApiError> {
        let request = &parsed.value;
        validate_generate_scalars(request)?;
        let imported = match self.require_source_state(&request.session_id)? {
            SessionSourceState::VirImported(imported) => imported,
            _ => return Err(session_state("session_id")),
        };
        if request.source_ir_schema != imported.module.schema
            || request.source_ir_hash != imported.module.vir_hash.as_str()
        {
            return Err(context_mismatch("source_ir_hash"));
        }
        let record = self
            .store
            .get(&request.frontend_source_manifest_hash)
            .ok_or_else(|| {
                V1ApiError::new(
                    V1ValidationPhase::Artifact,
                    V1ErrorCode::SourceContextUnknown,
                    Some("frontend_source_manifest_hash"),
                    None::<String>,
                )
            })?;
        let manifest = record.source_manifest.manifest();
        if request.input_set_hash != manifest.input_set_hash {
            return Err(V1ApiError::new(
                V1ValidationPhase::Artifact,
                V1ErrorCode::SourceManifestHash,
                Some("input_set_hash"),
                None::<String>,
            ));
        }
        let stored_vir = canonical_vir_json(&record.vir)
            .map_err(|_| source_manifest_invalid("VIR_CANONICAL"))?;
        if stored_vir != imported.canonical_bytes
            || record.vir.vir_hash != imported.module.vir_hash
            || record.source_map.map().source_ir_hash != imported.module.vir_hash.as_str()
            || manifest.vir_hash != imported.module.vir_hash.as_str()
        {
            return Err(context_mismatch("source_ir_hash"));
        }

        // `generate_vc_v1` revalidates the VIR, the frontend-stage manifest
        // capability, every repeated source identity, and the complete output.
        let vc = generate_vc_v1(&record.vir, &record.source_manifest)
            .map_err(|error| vc_invalid(error.code()))?;
        let document = vc.document();
        if document.schema != VC_SCHEMA_VERSION
            || document.source_ir_schema != request.source_ir_schema
            || document.source_ir_hash != request.source_ir_hash
            || document.input_set_hash != request.input_set_hash
            || document.semantic_profile != imported.module.semantic_profile
            || document.semantic_parameters != imported.module.semantic_parameters
        {
            return Err(vc_invalid("VC_SOURCE_LINKAGE"));
        }
        let (function_count, member_count, group_count) = vc_counts(document)?;
        let vc_value =
            serde_json::from_slice(vc.canonical_bytes()).map_err(|_| vc_invalid("VC_SHAPE"))?;
        let response = VcGenerateResponse {
            session_id: request.session_id.clone(),
            source_ir_schema: request.source_ir_schema.clone(),
            source_ir_hash: request.source_ir_hash.clone(),
            source_manifest_schema: request.source_manifest_schema.clone(),
            frontend_source_manifest_hash: request.frontend_source_manifest_hash.clone(),
            input_set_hash: request.input_set_hash.clone(),
            source_vc_schema: VC_SCHEMA_VERSION.to_owned(),
            vc_hash: vc.hash().as_str().to_owned(),
            function_count,
            member_count,
            group_count,
            helper_only: true,
            vc: vc_value,
        };
        let state = VcSessionState {
            imported: imported.clone(),
            source_manifest_schema: request.source_manifest_schema.clone(),
            frontend_source_manifest_hash: request.frontend_source_manifest_hash.clone(),
            input_set_hash: request.input_set_hash.clone(),
            vc,
            targets: BTreeMap::new(),
            next_target_index: 0,
        };
        Ok(PreparedVcGeneration { state, response })
    }

    pub(crate) fn handle_vc_list(&self, input: &[u8]) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VcListRequest>(input, 1_048_576)?;
        let state = self.validate_vc_context(&parsed.value)?;
        parsed.require_canonical()?;
        let members = state
            .vc
            .document()
            .functions
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
        encode_response(&VcListResponse {
            session_id: parsed.value.session_id,
            source_ir_schema: parsed.value.source_ir_schema,
            source_ir_hash: parsed.value.source_ir_hash,
            input_set_hash: parsed.value.input_set_hash,
            source_vc_schema: parsed.value.source_vc_schema,
            vc_hash: parsed.value.vc_hash,
            members,
            helper_only: true,
        })
    }

    pub(crate) fn handle_vc_start_proof(&mut self, input: &[u8]) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VcStartProofRequest>(input, 1_048_576)?;
        let (target_term, next_index) = {
            let state = self.validate_vc_context(&parsed.value.context)?;
            let term = resolve_target_term(state.vc.document(), &parsed.value.target)?;
            (term, state.next_target_index.checked_add(1))
        };
        let next_index = next_index.ok_or_else(|| session_state("target"))?;
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation()?;
        let target_id = format!("t{next_index}");
        let response = encode_response(&VcStartProofResponse {
            context: parsed.value.context.clone(),
            target: parsed.value.target.clone(),
            target_id: target_id.clone(),
            helper_only: true,
        })?;
        let target_term_id = {
            let session = self
                .legacy
                .session_mut(&parsed.value.context.session_id)
                .ok_or_else(|| unknown_session(&parsed.value.context.session_id))?;
            materialize_target(session, &target_term)?
        };
        let state = self.require_vc_state_mut(&parsed.value.context.session_id)?;
        state.next_target_index = next_index;
        state.targets.insert(
            target_id.clone(),
            ProofTargetRecord {
                target: parsed.value.target.clone(),
                target_term: target_term_id,
                candidates: BTreeMap::new(),
            },
        );
        self.set_mutation_count(next_mutation);
        Ok(response)
    }

    pub(crate) fn handle_vc_attach_candidate(
        &mut self,
        input: &[u8],
    ) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VcAttachCandidateRequest>(input, 1_048_576)?;
        validate_target_id(&parsed.value.target_id)?;
        validate_candidate_id(&parsed.value.candidate_id)?;
        {
            let state = self.validate_vc_context(&parsed.value.context)?;
            let target = state
                .targets
                .get(&parsed.value.target_id)
                .ok_or_else(|| target_unknown("target_id"))?;
            if target.candidates.contains_key(&parsed.value.candidate_id) {
                return Err(context_mismatch("candidate_id"));
            }
        }
        if self
            .legacy
            .session(&parsed.value.context.session_id)
            .and_then(|session| session.proof_node(parsed.value.proof_root))
            .is_none()
        {
            return Err(V1ApiError::inherited(
                V1ValidationPhase::Context,
                V1ErrorCode::UnknownProof,
                format!(
                    "proof id {} is not registered in this API session",
                    parsed.value.proof_root.as_u32()
                ),
                "proof_root",
            ));
        }
        parsed.require_canonical()?;
        let next_mutation = self.next_mutation()?;
        let response = encode_response(&VcAttachCandidateResponse {
            context: parsed.value.context.clone(),
            target_id: parsed.value.target_id.clone(),
            candidate_id: parsed.value.candidate_id.clone(),
            proof_root: parsed.value.proof_root,
            helper_only: true,
        })?;
        let state = self.require_vc_state_mut(&parsed.value.context.session_id)?;
        let target = state
            .targets
            .get_mut(&parsed.value.target_id)
            .ok_or_else(|| target_unknown("target_id"))?;
        target
            .candidates
            .insert(parsed.value.candidate_id.clone(), parsed.value.proof_root);
        self.set_mutation_count(next_mutation);
        Ok(response)
    }

    pub(crate) fn handle_vc_check_candidate(&self, input: &[u8]) -> Result<Vec<u8>, V1ApiError> {
        let parsed = parse_request::<VcCheckCandidateRequest>(input, 1_048_576)?;
        validate_target_id(&parsed.value.target_id)?;
        if parsed.value.mode != CHECK_MODE || parsed.value.candidates.is_empty() {
            return Err(V1ApiError::new(
                V1ValidationPhase::Scalar,
                V1ErrorCode::Scalar,
                Some("mode"),
                None::<String>,
            ));
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
                    return Err(V1ApiError::new(
                        V1ValidationPhase::Context,
                        V1ErrorCode::CandidateUnknown,
                        Some("candidate_id"),
                        None::<String>,
                    ))
                }
            }
        }
        parsed.require_canonical()?;
        let session = self
            .legacy
            .session(&parsed.value.context.session_id)
            .ok_or_else(|| unknown_session(&parsed.value.context.session_id))?;
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
        encode_response(&VcCheckCandidateResponse {
            context: parsed.value.context,
            target_id: parsed.value.target_id,
            mode: parsed.value.mode,
            results,
            helper_only: true,
        })
    }

    pub(crate) fn target_term_id(
        &self,
        session_id: &SessionId,
        target_id: &str,
    ) -> Option<ApiTermId> {
        match self.sources.get(session_id)? {
            SessionSourceState::VcGenerated(state) => state
                .targets
                .get(target_id)
                .map(|target| target.target_term),
            _ => None,
        }
    }

    pub(crate) fn retained_context(&self, session_id: &SessionId) -> Option<(&str, &str)> {
        match self.sources.get(session_id)? {
            SessionSourceState::VcGenerated(state) => Some((
                state.source_manifest_schema.as_str(),
                state.frontend_source_manifest_hash.as_str(),
            )),
            _ => None,
        }
    }

    pub(crate) fn target_binding(
        &self,
        session_id: &SessionId,
        target_id: &str,
    ) -> Option<&VcProofTarget> {
        match self.sources.get(session_id)? {
            SessionSourceState::VcGenerated(state) => {
                state.targets.get(target_id).map(|target| &target.target)
            }
            _ => None,
        }
    }

    fn validate_vc_context<C: VcContext>(
        &self,
        request: &C,
    ) -> Result<&VcSessionState, V1ApiError> {
        validate_context_scalars(request)?;
        let state = match self.require_source_state(request.session_id())? {
            SessionSourceState::VcGenerated(state) => state,
            _ => return Err(session_state("session_id")),
        };
        if request.source_ir_schema() != state.imported.module.schema
            || request.source_ir_hash() != state.imported.module.vir_hash.as_str()
            || request.input_set_hash() != state.input_set_hash
            || request.source_vc_schema() != state.vc.document().schema
            || request.vc_hash() != state.vc.hash().as_str()
        {
            return Err(context_mismatch("source_ir_hash"));
        }
        Ok(state)
    }

    fn require_source_state(
        &self,
        session_id: &SessionId,
    ) -> Result<&SessionSourceState, V1ApiError> {
        if self.legacy.session(session_id).is_none() {
            return Err(unknown_session(session_id));
        }
        self.sources
            .get(session_id)
            .ok_or_else(|| session_state("session_id"))
    }

    fn require_vc_state_mut(
        &mut self,
        session_id: &SessionId,
    ) -> Result<&mut VcSessionState, V1ApiError> {
        match self.sources.get_mut(session_id) {
            Some(SessionSourceState::VcGenerated(state)) => Ok(state),
            _ => Err(session_state("session_id")),
        }
    }

    fn next_mutation(&self) -> Result<u64, V1ApiError> {
        self.mutation_count()
            .checked_add(1)
            .ok_or_else(|| session_state("session_id"))
    }

    fn set_mutation_count(&mut self, mutation_count: u64) {
        self.commit_mutation_count(mutation_count);
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcGenerateRequest {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) source_manifest_schema: String,
    pub(crate) frontend_source_manifest_hash: String,
    pub(crate) input_set_hash: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcGenerateResponse {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) source_manifest_schema: String,
    pub(crate) frontend_source_manifest_hash: String,
    pub(crate) input_set_hash: String,
    pub(crate) source_vc_schema: String,
    pub(crate) vc_hash: String,
    pub(crate) function_count: u64,
    pub(crate) member_count: u64,
    pub(crate) group_count: u64,
    pub(crate) helper_only: bool,
    pub(crate) vc: Value,
}

struct PreparedVcGeneration {
    state: VcSessionState,
    response: VcGenerateResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcListRequest {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) input_set_hash: String,
    pub(crate) source_vc_schema: String,
    pub(crate) vc_hash: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcListResponse {
    pub(crate) session_id: SessionId,
    pub(crate) source_ir_schema: String,
    pub(crate) source_ir_hash: String,
    pub(crate) input_set_hash: String,
    pub(crate) source_vc_schema: String,
    pub(crate) vc_hash: String,
    pub(crate) members: Vec<VcMemberSummary>,
    pub(crate) helper_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcMemberSummary {
    pub(crate) member_id: String,
    pub(crate) function_id: String,
    pub(crate) kind: mpk_vc::VcMemberKind,
    pub(crate) group_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcStartProofRequest {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target: VcProofTarget,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcStartProofResponse {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target: VcProofTarget,
    pub(crate) target_id: String,
    pub(crate) helper_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum VcProofTarget {
    Member { id: String },
    Group { id: String },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcAttachCandidateRequest {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target_id: String,
    pub(crate) candidate_id: String,
    pub(crate) proof_root: ApiProofId,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcAttachCandidateResponse {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target_id: String,
    pub(crate) candidate_id: String,
    pub(crate) proof_root: ApiProofId,
    pub(crate) helper_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcCheckCandidateRequest {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target_id: String,
    pub(crate) mode: String,
    pub(crate) candidates: Vec<VcCandidateBinding>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcCandidateBinding {
    pub(crate) candidate_id: String,
    pub(crate) proof_root: ApiProofId,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcCheckCandidateResponse {
    #[serde(flatten)]
    pub(crate) context: VcListRequest,
    pub(crate) target_id: String,
    pub(crate) mode: String,
    pub(crate) results: Vec<VcCandidateResult>,
    pub(crate) helper_only: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HelperStatus {
    Valid,
    Invalid,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcCandidateResult {
    pub(crate) candidate_id: String,
    pub(crate) proof_root: ApiProofId,
    pub(crate) helper_status: HelperStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) diagnostic: Option<RepairDiagnostic>,
}

trait VcContext {
    fn session_id(&self) -> &SessionId;
    fn source_ir_schema(&self) -> &str;
    fn source_ir_hash(&self) -> &str;
    fn input_set_hash(&self) -> &str;
    fn source_vc_schema(&self) -> &str;
    fn vc_hash(&self) -> &str;
}

impl VcContext for VcListRequest {
    fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    fn source_ir_schema(&self) -> &str {
        &self.source_ir_schema
    }

    fn source_ir_hash(&self) -> &str {
        &self.source_ir_hash
    }

    fn input_set_hash(&self) -> &str {
        &self.input_set_hash
    }

    fn source_vc_schema(&self) -> &str {
        &self.source_vc_schema
    }

    fn vc_hash(&self) -> &str {
        &self.vc_hash
    }
}

fn validate_generate_scalars(request: &VcGenerateRequest) -> Result<(), V1ApiError> {
    if request.source_ir_schema != VIR_SCHEMA_VERSION {
        return Err(V1ApiError::new(
            V1ValidationPhase::Shape,
            V1ErrorCode::VirSchema,
            Some("source_ir_schema"),
            None::<String>,
        ));
    }
    if request.source_manifest_schema != SOURCE_MANIFEST_SCHEMA {
        return Err(V1ApiError::new(
            V1ValidationPhase::Shape,
            V1ErrorCode::SourceManifestSchema,
            Some("source_manifest_schema"),
            None::<String>,
        ));
    }
    validate_session_id(request.session_id.as_str())?;
    validate_sha256(&request.source_ir_hash, "source_ir_hash")?;
    validate_sha256(
        &request.frontend_source_manifest_hash,
        "frontend_source_manifest_hash",
    )?;
    validate_sha256(&request.input_set_hash, "input_set_hash")
}

fn validate_context_scalars<C: VcContext>(request: &C) -> Result<(), V1ApiError> {
    if request.source_ir_schema() != VIR_SCHEMA_VERSION
        || request.source_vc_schema() != VC_SCHEMA_VERSION
    {
        return Err(context_mismatch("source_ir_schema"));
    }
    validate_session_id(request.session_id().as_str())?;
    validate_sha256(request.source_ir_hash(), "source_ir_hash")?;
    validate_sha256(request.input_set_hash(), "input_set_hash")?;
    validate_sha256(request.vc_hash(), "vc_hash")
}

fn vc_counts(document: &VcDocument) -> Result<(u64, u64, u64), V1ApiError> {
    let functions = u64::try_from(document.functions.len()).map_err(|_| vc_invalid("VC_LIMIT"))?;
    let (members, groups) =
        document
            .functions
            .iter()
            .try_fold((0_u64, 0_u64), |(members, groups), function| {
                Ok::<_, V1ApiError>((
                    members
                        .checked_add(u64::try_from(function.members.len()).unwrap_or(u64::MAX))
                        .ok_or_else(|| vc_invalid("VC_LIMIT"))?,
                    groups
                        .checked_add(u64::try_from(function.groups.len()).unwrap_or(u64::MAX))
                        .ok_or_else(|| vc_invalid("VC_LIMIT"))?,
                ))
            })?;
    Ok((functions, members, groups))
}

fn resolve_target_term(
    document: &VcDocument,
    target: &VcProofTarget,
) -> Result<VcTerm, V1ApiError> {
    match target {
        VcProofTarget::Member { id } => {
            let matches = document.functions.iter().flat_map(|function| {
                function
                    .members
                    .iter()
                    .filter(move |member| member.id == *id)
                    .map(move |member| (function, member))
            });
            let (function, member) = exactly_one(matches)?;
            let body = bind_function_variables(member_theorem_type(member), function)?;
            Ok(wrap_function_parameters(function, body))
        }
        VcProofTarget::Group { id } => {
            let matches = document.functions.iter().flat_map(|function| {
                function
                    .groups
                    .iter()
                    .filter(move |group| group.id == *id)
                    .map(move |group| (function, group))
            });
            let (function, group) = exactly_one(matches)?;
            let body = group_body(function, group).map_err(|_| vc_invalid("VC_GROUP_SET"))?;
            let body = bind_function_variables(body, function)?;
            Ok(wrap_function_parameters(function, body))
        }
    }
}

fn exactly_one<'a, T>(mut values: impl Iterator<Item = T> + 'a) -> Result<T, V1ApiError> {
    let first = values.next().ok_or_else(|| target_unknown("target"))?;
    if values.next().is_some() {
        return Err(target_unknown("target"));
    }
    Ok(first)
}

fn wrap_function_parameters(function: &VcFunction, body: VcTerm) -> VcTerm {
    function
        .parameters
        .iter()
        .rev()
        .fold(body, |body, parameter| VcTerm::Forall {
            binder_type: parameter.r#type.clone(),
            body: Box::new(body),
        })
}

fn bind_function_variables(term: VcTerm, function: &VcFunction) -> Result<VcTerm, V1ApiError> {
    let parameter_indices = function
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            let reverse = function
                .parameters
                .len()
                .checked_sub(index + 1)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| vc_invalid("VC_TERM_DEPTH"))?;
            Ok((parameter.id.as_str(), reverse))
        })
        .collect::<Result<BTreeMap<_, _>, V1ApiError>>()?;
    bind_named_variables(term, &parameter_indices, 0)
}

fn bind_named_variables(
    term: VcTerm,
    parameter_indices: &BTreeMap<&str, u32>,
    local_depth: u32,
) -> Result<VcTerm, V1ApiError> {
    match term {
        VcTerm::Var { name } => {
            let index = parameter_indices
                .get(name.as_str())
                .copied()
                .ok_or_else(|| vc_invalid("VC_TERM_VARIABLE"))?
                .checked_add(local_depth)
                .ok_or_else(|| vc_invalid("VC_TERM_DEPTH"))?;
            Ok(VcTerm::Bound { index })
        }
        VcTerm::Apply { function, args } => Ok(VcTerm::Apply {
            function,
            args: args
                .into_iter()
                .map(|argument| bind_named_variables(argument, parameter_indices, local_depth))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        VcTerm::Convert { value, target } => Ok(VcTerm::Convert {
            value: Box::new(bind_named_variables(
                *value,
                parameter_indices,
                local_depth,
            )?),
            target,
        }),
        VcTerm::Forall { binder_type, body } => Ok(VcTerm::Forall {
            binder_type,
            body: Box::new(bind_named_variables(
                *body,
                parameter_indices,
                local_depth
                    .checked_add(1)
                    .ok_or_else(|| vc_invalid("VC_TERM_DEPTH"))?,
            )?),
        }),
        other => Ok(other),
    }
}

fn materialize_target(session: &mut ApiSession, target: &VcTerm) -> Result<ApiTermId, V1ApiError> {
    materialize_vc_term(session, target)
}

fn materialize_vc_term(session: &mut ApiSession, term: &VcTerm) -> Result<ApiTermId, V1ApiError> {
    match term {
        VcTerm::Var { .. } => Err(vc_invalid("VC_TERM_VARIABLE")),
        VcTerm::Bound { index } => {
            let term = session.terms_mut().var(*index);
            register_core_term(session, term)
        }
        VcTerm::Constant { name } => materialize_constant(session, name),
        VcTerm::BitVecLiteral {
            value,
            width,
            signed,
        } => materialize_constant(
            session,
            &format!(
                "V1.Literal.{}.{}.{}",
                if *signed { "Signed" } else { "Unsigned" },
                width,
                value.replace('-', "Neg")
            ),
        ),
        VcTerm::Apply { function, args } if function == "Std.Logic.Imp" && args.len() == 2 => {
            let domain = materialize_vc_term(session, &args[0])?;
            let body = materialize_vc_term(session, &args[1])?;
            let domain = require_core_term(session, domain)?;
            let body = require_core_term(session, body)?;
            let term = session.terms_mut().pi(domain, body);
            register_core_term(session, term)
        }
        VcTerm::Apply { function, args } => {
            let function = materialize_constant(session, function)?;
            let function = require_core_term(session, function)?;
            let arguments = args
                .iter()
                .map(|argument| {
                    materialize_vc_term(session, argument)
                        .and_then(|term| require_core_term(session, term))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let term = session.terms_mut().app(function, arguments);
            register_core_term(session, term)
        }
        VcTerm::Convert { value, target } => {
            let function = materialize_constant(session, "V1.Term.Convert")?;
            let function = require_core_term(session, function)?;
            let value = materialize_vc_term(session, value)?;
            let value = require_core_term(session, value)?;
            let target = materialize_vc_type(session, target)?;
            let target = require_core_term(session, target)?;
            let term = session.terms_mut().app(function, vec![value, target]);
            register_core_term(session, term)
        }
        VcTerm::Forall { binder_type, body } => {
            let domain = materialize_vc_type(session, binder_type)?;
            let body = materialize_vc_term(session, body)?;
            let domain = require_core_term(session, domain)?;
            let body = require_core_term(session, body)?;
            let term = session.terms_mut().pi(domain, body);
            register_core_term(session, term)
        }
    }
}

fn materialize_vc_type(
    session: &mut ApiSession,
    term: &VcTypeTerm,
) -> Result<ApiTermId, V1ApiError> {
    match term {
        VcTypeTerm::Constant { name } => materialize_constant(session, name),
        VcTypeTerm::Apply { function, args } => {
            let function = materialize_constant(session, function)?;
            let function = require_core_term(session, function)?;
            let arguments = args
                .iter()
                .map(|argument| {
                    materialize_vc_type(session, argument)
                        .and_then(|term| require_core_term(session, term))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let term = session.terms_mut().app(function, arguments);
            register_core_term(session, term)
        }
        VcTypeTerm::NatLiteral { value } => {
            materialize_constant(session, &format!("V1.Type.Nat.{value}"))
        }
        VcTypeTerm::StringLiteral { value } => {
            let encoded = value
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            materialize_constant(session, &format!("V1.Type.String.X{encoded}"))
        }
    }
}

fn materialize_constant(session: &mut ApiSession, name: &str) -> Result<ApiTermId, V1ApiError> {
    let global = match session.environment().resolve(name) {
        Ok(Some(global)) => global,
        Ok(None) => {
            let level = session.levels().zero();
            let sort = session.terms_mut().sort(level);
            session
                .environment_mut()
                .register_axiom(name, sort)
                .map_err(|_| vc_invalid("VC_TERM_NAME"))?
        }
        Err(_) => return Err(vc_invalid("VC_TERM_NAME")),
    };
    let term = session.terms_mut().constant(global, Vec::new());
    register_core_term(session, term)
}

fn register_core_term(session: &mut ApiSession, term: TermId) -> Result<ApiTermId, V1ApiError> {
    session
        .register_term_id(term)
        .map_err(|_| vc_invalid("VC_TERM_ID"))
}

fn require_core_term(session: &ApiSession, term: ApiTermId) -> Result<TermId, V1ApiError> {
    session
        .core_term_id(term)
        .ok_or_else(|| vc_invalid("VC_TERM_ID"))
}

fn check_candidate_root(
    session: &ApiSession,
    proof_root: ApiProofId,
    target: ApiTermId,
) -> Option<RepairDiagnostic> {
    let Some(target_core) = session.core_term_id(target) else {
        return Some(RepairDiagnostic {
            ok: false,
            error_code: Some("DEF_EQ_HEAD_MISMATCH".to_owned()),
            node_id: proof_root,
            expected_type_id: Some(target),
            actual_type_id: None,
            expected_head: None,
            actual_head: None,
            context_summary: Vec::new(),
            repair_hints: vec!["inspect-node".to_owned()],
        });
    };
    match check_proof_against(session, proof_root, target_core) {
        Ok(()) => None,
        Err(actual) => Some(RepairDiagnostic {
            ok: false,
            error_code: Some("DEF_EQ_HEAD_MISMATCH".to_owned()),
            node_id: proof_root,
            expected_type_id: Some(target),
            actual_type_id: actual.and_then(|term| u32::try_from(term.index()).ok().map(ApiTermId)),
            expected_head: term_head(session, target_core),
            actual_head: actual.and_then(|term| term_head(session, term)),
            context_summary: Vec::new(),
            repair_hints: vec!["intro".to_owned(), "conv".to_owned()],
        }),
    }
}

fn check_proof_against(
    session: &ApiSession,
    proof_id: ApiProofId,
    expected: TermId,
) -> Result<(), Option<TermId>> {
    let node = session.proof_node(proof_id).ok_or(None)?;
    let stated = proof_expected_type(node);
    let stated = core_by_index(session, stated).ok_or(None)?;
    if stated != expected {
        return Err(Some(stated));
    }
    match node {
        ProofNode::Refl { term, .. } => {
            let TermNode::App {
                function,
                arguments,
            } = session.terms().node(expected)
            else {
                return Err(Some(stated));
            };
            let reflected = core_by_index(session, *term).ok_or(None)?;
            if global_name(session, *function).as_deref() == Some("Std.Eq")
                && arguments.as_slice() == [reflected, reflected]
            {
                Ok(())
            } else {
                Err(Some(stated))
            }
        }
        ProofNode::Intro {
            domain_type,
            body_proof,
            ..
        } => {
            let TermNode::Pi { ty, body } = session.terms().node(expected) else {
                return Err(Some(stated));
            };
            if core_by_index(session, *domain_type) != Some(*ty) {
                return Err(core_by_index(session, *domain_type));
            }
            check_proof_against(session, ApiProofId(*body_proof), *body)
        }
        // The staged owner currently has complete, read-only target checking
        // for the frozen v1 recipes. Other node kinds must go through their
        // unchanged declaration/check workflow and cannot be upgraded to a
        // helper-valid VC candidate merely by repeating the target ID.
        _ => Err(Some(stated)),
    }
}

fn proof_expected_type(node: &ProofNode) -> u32 {
    match node {
        ProofNode::Exact { expected_type, .. }
        | ProofNode::Apply { expected_type, .. }
        | ProofNode::Intro { expected_type, .. }
        | ProofNode::LetProof { expected_type, .. }
        | ProofNode::Refl { expected_type, .. }
        | ProofNode::Rewrite { expected_type, .. }
        | ProofNode::EqRec { expected_type, .. }
        | ProofNode::Constructor { expected_type, .. }
        | ProofNode::Recursor { expected_type, .. }
        | ProofNode::Conv { expected_type, .. }
        | ProofNode::Theory { expected_type, .. } => *expected_type,
    }
}

fn core_by_index(session: &ApiSession, raw: u32) -> Option<TermId> {
    session
        .terms()
        .iter_topological()
        .find_map(|(term, _)| (term.index() == usize::try_from(raw).ok()?).then_some(term))
}

fn global_name(session: &ApiSession, term: TermId) -> Option<String> {
    let TermNode::Const { global, .. } = session.terms().node(term) else {
        return None;
    };
    session
        .environment()
        .lookup(*global)
        .map(|declaration| declaration.name().as_str().to_owned())
}

fn term_head(session: &ApiSession, term: TermId) -> Option<String> {
    match session.terms().node(term) {
        TermNode::Sort(_) => Some("sort".to_owned()),
        TermNode::Var(index) => Some(format!("var:{index}")),
        TermNode::Const { .. } => global_name(session, term),
        TermNode::App { function, .. } => term_head(session, *function),
        TermNode::Lam { .. } => Some("lam".to_owned()),
        TermNode::Pi { .. } => Some("pi".to_owned()),
        TermNode::Let { .. } => Some("let".to_owned()),
    }
}

fn validate_candidate_id(candidate_id: &str) -> Result<(), V1ApiError> {
    if !candidate_id.is_empty()
        && candidate_id.len() <= 256
        && candidate_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
    {
        Ok(())
    } else {
        Err(V1ApiError::new(
            V1ValidationPhase::Scalar,
            V1ErrorCode::Scalar,
            Some("candidate_id"),
            None::<String>,
        ))
    }
}

fn validate_target_id(target_id: &str) -> Result<(), V1ApiError> {
    let valid = target_id.strip_prefix('t').is_some_and(|digits| {
        !digits.is_empty()
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok_and(|number| number > 0)
    });
    if valid {
        Ok(())
    } else {
        Err(V1ApiError::new(
            V1ValidationPhase::Scalar,
            V1ErrorCode::Scalar,
            Some("target_id"),
            None::<String>,
        ))
    }
}

fn unknown_session(session_id: &SessionId) -> V1ApiError {
    V1ApiError::inherited(
        V1ValidationPhase::Session,
        V1ErrorCode::UnknownSession,
        format!("API session {session_id} does not exist"),
        "session_id",
    )
}

fn session_state(field: &'static str) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Session,
        V1ErrorCode::SessionState,
        Some(field),
        None::<String>,
    )
}

fn context_mismatch(field: &'static str) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Context,
        V1ErrorCode::ContextMismatch,
        Some(field),
        None::<String>,
    )
}

fn target_unknown(field: &'static str) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Context,
        V1ErrorCode::TargetUnknown,
        Some(field),
        None::<String>,
    )
}

fn source_manifest_invalid(detail: impl Into<String>) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Artifact,
        V1ErrorCode::SourceManifestInvalid,
        None,
        Some(detail),
    )
}

fn vc_invalid(detail: impl Into<String>) -> V1ApiError {
    V1ApiError::new(
        V1ValidationPhase::Artifact,
        V1ErrorCode::VcInvalid,
        None,
        Some(detail),
    )
}
