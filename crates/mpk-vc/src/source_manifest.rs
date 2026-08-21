//! Strict source-manifest v0 lifecycle model and validation.

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, StrictJsonError, StrictJsonLimits, StrictJsonValue,
};
use crate::hash::{hash_canonical_json, HashDomain};
use crate::release_bundle::{
    CompilerIdentity, ToolchainComponent, ValidatedReleaseRegistry, RELEASE_REGISTRY_ID,
    RELEASE_REGISTRY_SCHEMA,
};
use crate::semantic_profile::{
    validate_semantic_context, PointerWidth, SemanticParameters, SemanticProfile, SourceLanguage,
};
use crate::source_map::{
    captured_input_matches, is_portable_normalized_path, CapturedInput, InputKind, SourceOrigin,
    ValidatedSourceMap, NORMALIZED_PATH_BYTES_MAX,
};
use crate::vir::{LowercaseSha256, VirModule, VIR_SCHEMA_VERSION};
use crate::vir_canonical::vir_hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const SOURCE_MANIFEST_SCHEMA_VERSION: &str = "mpk.source_manifest.v0";
pub const SOURCE_MANIFEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MANIFEST-0.1");
pub const INPUT_SET_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-INPUT-SET-0.1");
pub const SOURCE_MANIFEST_CANONICAL_BYTES_MAX: u64 = 4_194_304;
pub const SOURCE_MANIFEST_GO_INPUTS_MAX: u64 = 32_768;
pub const SOURCE_MANIFEST_RUST_INPUTS_MAX: u64 = 512;
pub const SOURCE_MANIFEST_UNITS_MAX: u64 = 256;
pub const SOURCE_MANIFEST_TOOLCHAIN_COMPONENTS_MAX: u64 = 8_192;
pub const SOURCE_MANIFEST_CFG_ENTRIES_MAX: u64 = 16_384;

const SOURCE_MANIFEST_JSON_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    268_435_456,
    SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
    256,
    1_048_576,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceManifestStage {
    Frontend,
    Certificate,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    pub schema: String,
    pub source_language: SourceLanguage,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub selection: ManifestSelection,
    pub limit_profile: String,
    pub release_registry: ReleaseRegistryIdentity,
    pub toolchain: ToolchainIdentity,
    pub frontend: FrontendIdentity,
    pub units: Vec<ManifestUnit>,
    pub target: TargetIdentity,
    pub inputs: Vec<InputEntry>,
    pub input_set_hash: String,
    pub vir_hash: String,
    pub source_map_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vc_hash: Option<String>,
    pub source_manifest_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ManifestSelection {
    Go(GoSelection),
    Rust(RustSelection),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GoSelection {
    pub package: String,
    pub function: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RustSelection {
    pub package: String,
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub kind: RustUnitKind,
    pub function: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RustUnitKind {
    Lib,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseRegistryIdentity {
    pub schema: String,
    pub id: String,
    pub registry_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendIdentity {
    pub bundle_id: String,
    pub name: String,
    pub version: String,
    pub binary_sha256: String,
    pub subordinate_binaries: Vec<SubordinateIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubordinateIdentity {
    pub name: String,
    pub version: String,
    pub binary_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainIdentity {
    pub bundle_id: String,
    pub distribution_sha256: String,
    pub components: Vec<ComponentIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ComponentIdentity {
    Executable {
        name: String,
        release: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        commit_hash: Option<String>,
        binary_sha256: String,
    },
    Content {
        name: String,
        release: String,
        content_sha256: String,
    },
}

impl ComponentIdentity {
    fn name(&self) -> &str {
        match self {
            Self::Executable { name, .. } | Self::Content { name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestUnit {
    pub identity: String,
    pub name: String,
    pub kind: ManifestUnitKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestUnitKind {
    Package,
    Lib,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetIdentity {
    pub id: String,
    pub pointer_width: PointerWidth,
    pub language_configuration: LanguageConfiguration,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum LanguageConfiguration {
    Go {
        compiler: String,
        cgo_enabled: bool,
        go111module: String,
        module_mode: String,
        workspace_mode: String,
        tests: bool,
        build_tags: Vec<String>,
        environment_profile_id: String,
        argument_profile_id: String,
    },
    Rust {
        edition: String,
        crate_type: String,
        enabled_features: Vec<String>,
        prelude: RustPrelude,
        locked: bool,
        offline: bool,
        default_features: bool,
        overflow_checks: bool,
        panic: String,
        debug_assertions: bool,
        rustc_opt_level: i64,
        mir_opt_level: i64,
        jobs: i64,
        message_format: String,
        target_allowlist_id: String,
        environment_profile_id: String,
        argument_profile_id: String,
        cfg: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RustPrelude {
    Std,
    Core,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputEntry {
    pub kind: InputKind,
    pub normalized_path: String,
    pub size_bytes: i64,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceManifestValidationContext<'a> {
    pub vir: &'a VirModule,
    pub source_map: &'a ValidatedSourceMap,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub release_registry: &'a ValidatedReleaseRegistry,
    /// Exact final compiler-session configuration. Required for Rust; Go v0
    /// has a single closed configuration and may omit this repetition.
    pub expected_language_configuration: Option<&'a LanguageConfiguration>,
}

/// Projection supplied by a VC validator after it has recomputed `vc_hash`.
/// Requiring all repeated identities prevents finalization from a hash alone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedVcIdentity {
    input_set_hash: LowercaseSha256,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    semantic_profile: SemanticProfile,
    semantic_parameters: SemanticParameters,
    vc_hash: LowercaseSha256,
}

impl ValidatedVcIdentity {
    pub fn new(
        input_set_hash: String,
        source_ir_schema: String,
        source_ir_hash: String,
        semantic_profile: SemanticProfile,
        semantic_parameters: SemanticParameters,
        recomputed_vc_hash: String,
    ) -> Result<Self, SourceManifestError> {
        validate_semantic_context(
            semantic_profile.source_language(),
            semantic_profile,
            &semantic_parameters,
        )
        .map_err(|error| {
            invalid_profile(SourceManifestValidationPhase::Scalar, error.to_string())
        })?;
        Ok(Self {
            input_set_hash: parse_hash(&input_set_hash)?,
            source_ir_schema,
            source_ir_hash: parse_hash(&source_ir_hash)?,
            semantic_profile,
            semantic_parameters,
            vc_hash: parse_hash(&recomputed_vc_hash)?,
        })
    }

    pub fn vc_hash(&self) -> &LowercaseSha256 {
        &self.vc_hash
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSourceManifest {
    manifest: SourceManifest,
    stage: SourceManifestStage,
    canonical_bytes: Vec<u8>,
    hash: LowercaseSha256,
}

impl ValidatedSourceManifest {
    pub fn manifest(&self) -> &SourceManifest {
        &self.manifest
    }

    pub const fn stage(&self) -> SourceManifestStage {
        self.stage
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceManifestValidationPhase {
    Transport,
    Shape,
    Scalar,
    Order,
    Semantic,
    Release,
    Inputs,
    Artifacts,
    VcLinkage,
    CanonicalSize,
    Hash,
}

impl SourceManifestValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Order => "order",
            Self::Semantic => "semantic",
            Self::Release => "release",
            Self::Inputs => "inputs",
            Self::Artifacts => "artifacts",
            Self::VcLinkage => "vc_linkage",
            Self::CanonicalSize => "canonical_size",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceManifestErrorCode {
    JsonDuplicateKey,
    JsonInvalid,
    Schema,
    Shape,
    Stage,
    Path,
    Order,
    Profile,
    Selection,
    Units,
    Release,
    InputKind,
    InputBytes,
    InputSetHash,
    IrLinkage,
    SourceMapLinkage,
    VcLinkage,
    LifecycleMutation,
    Limit,
    Hash,
}

impl SourceManifestErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JsonDuplicateKey => "SOURCE_MANIFEST_JSON_DUPLICATE_KEY",
            Self::JsonInvalid => "SOURCE_MANIFEST_JSON_INVALID",
            Self::Schema => "SOURCE_MANIFEST_SCHEMA",
            Self::Shape => "SOURCE_MANIFEST_SHAPE",
            Self::Stage => "SOURCE_MANIFEST_STAGE",
            Self::Path => "SOURCE_MANIFEST_PATH",
            Self::Order => "SOURCE_MANIFEST_ORDER",
            Self::Profile => "SOURCE_MANIFEST_PROFILE",
            Self::Selection => "SOURCE_MANIFEST_SELECTION",
            Self::Units => "SOURCE_MANIFEST_UNITS",
            Self::Release => "SOURCE_MANIFEST_RELEASE",
            Self::InputKind => "SOURCE_MANIFEST_INPUT_KIND",
            Self::InputBytes => "SOURCE_MANIFEST_INPUT_BYTES",
            Self::InputSetHash => "SOURCE_MANIFEST_INPUT_SET_HASH",
            Self::IrLinkage => "SOURCE_MANIFEST_IR_LINKAGE",
            Self::SourceMapLinkage => "SOURCE_MANIFEST_SOURCE_MAP_LINKAGE",
            Self::VcLinkage => "SOURCE_MANIFEST_VC_LINKAGE",
            Self::LifecycleMutation => "SOURCE_MANIFEST_LIFECYCLE_MUTATION",
            Self::Limit => "SOURCE_MANIFEST_LIMIT",
            Self::Hash => "SOURCE_MANIFEST_HASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceManifestError {
    pub phase: SourceManifestValidationPhase,
    pub code: SourceManifestErrorCode,
    pub detail: String,
}

impl SourceManifestError {
    fn new(
        phase: SourceManifestValidationPhase,
        code: SourceManifestErrorCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SourceManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code.as_str(),
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for SourceManifestError {}

pub fn import_frontend_source_manifest_json(
    input: &[u8],
    context: SourceManifestValidationContext<'_>,
) -> Result<ValidatedSourceManifest, SourceManifestError> {
    import_source_manifest(input, SourceManifestStage::Frontend, context, None)
}

pub fn import_certificate_source_manifest_json(
    input: &[u8],
    context: SourceManifestValidationContext<'_>,
    vc: &ValidatedVcIdentity,
) -> Result<ValidatedSourceManifest, SourceManifestError> {
    import_source_manifest(input, SourceManifestStage::Certificate, context, Some(vc))
}

/// Finalizes only validated canonical frontend-stage bytes by adding a checked VC hash.
pub fn attach_vc_hash(
    canonical_frontend_bytes: &[u8],
    context: SourceManifestValidationContext<'_>,
    vc: &ValidatedVcIdentity,
) -> Result<ValidatedSourceManifest, SourceManifestError> {
    let frontend = import_frontend_source_manifest_json(canonical_frontend_bytes, context)?;
    if frontend.canonical_bytes != canonical_frontend_bytes {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::VcLinkage,
            SourceManifestErrorCode::LifecycleMutation,
            "frontend-stage bytes are not their exact canonical re-encoding",
        ));
    }
    validate_vc_linkage(frontend.manifest(), context.vir, vc)?;

    let mut certificate = frontend.manifest.clone();
    certificate.vc_hash = Some(vc.vc_hash.as_str().to_owned());
    certificate.source_manifest_hash = "0".repeat(64);
    certificate.source_manifest_hash = source_manifest_hash(&certificate)?.as_str().to_owned();
    let bytes = canonical_source_manifest_json(&certificate)?;
    import_certificate_source_manifest_json(&bytes, context, vc)
}

/// Validates an externally carried lifecycle transition and rejects any mutation
/// beyond `vc_hash` plus the lifecycle self-hash.
pub fn validate_source_manifest_transition(
    canonical_frontend_bytes: &[u8],
    certificate_bytes: &[u8],
    context: SourceManifestValidationContext<'_>,
    vc: &ValidatedVcIdentity,
) -> Result<ValidatedSourceManifest, SourceManifestError> {
    let frontend = import_frontend_source_manifest_json(canonical_frontend_bytes, context)?;
    if frontend.canonical_bytes != canonical_frontend_bytes {
        return Err(lifecycle_mutation("frontend manifest is not canonical"));
    }
    let certificate_value = parse_manifest_transport(certificate_bytes)?;
    ensure_stage(&certificate_value, SourceManifestStage::Certificate)?;
    let frontend_value = parse_manifest_transport(canonical_frontend_bytes)?;
    let frontend_payload = frontend_value
        .clone_without_fields(&["source_manifest_hash"])
        .map_err(|error| lifecycle_mutation(error.to_string()))?;
    let certificate_payload = certificate_value
        .clone_without_fields(&["source_manifest_hash", "vc_hash"])
        .map_err(|error| lifecycle_mutation(error.to_string()))?;
    if frontend_payload != certificate_payload {
        return Err(lifecycle_mutation(
            "certificate assembly changed a frontend-stage member",
        ));
    }
    import_certificate_source_manifest_json(certificate_bytes, context, vc)
}

fn import_source_manifest(
    input: &[u8],
    stage: SourceManifestStage,
    context: SourceManifestValidationContext<'_>,
    vc: Option<&ValidatedVcIdentity>,
) -> Result<ValidatedSourceManifest, SourceManifestError> {
    let strict = parse_manifest_transport(input)?;
    validate_transport_counts(&strict)?;
    ensure_stage(&strict, stage)?;
    let canonical = canonical_json_bytes(&strict).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Transport,
            SourceManifestErrorCode::JsonInvalid,
            error.to_string(),
        )
    })?;
    let manifest: SourceManifest = serde_json::from_slice(&canonical).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Shape,
            SourceManifestErrorCode::Shape,
            error.to_string(),
        )
    })?;

    validate_shape(&manifest)?;
    validate_scalars(&manifest)?;
    validate_order(&manifest)?;
    validate_semantics(
        &manifest,
        context.vir,
        context.expected_language_configuration,
    )?;
    validate_release(&manifest, context.release_registry)?;
    validate_inputs(&manifest, context.captured_inputs)?;
    validate_artifacts(
        &manifest,
        context.vir,
        context.source_map,
        context.captured_inputs,
    )?;
    if stage == SourceManifestStage::Certificate {
        validate_vc_linkage(
            &manifest,
            context.vir,
            vc.ok_or_else(|| {
                SourceManifestError::new(
                    SourceManifestValidationPhase::VcLinkage,
                    SourceManifestErrorCode::VcLinkage,
                    "certificate-stage validation requires a validated VC identity",
                )
            })?,
        )?;
    }
    validate_source_manifest_canonical_size(canonical.len() as u64)?;
    let hash = recompute_manifest_hash_from_value(&strict)?;
    if manifest.source_manifest_hash != hash.as_str() {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Hash,
            SourceManifestErrorCode::Hash,
            "source_manifest_hash does not match the lifecycle payload",
        ));
    }

    Ok(ValidatedSourceManifest {
        manifest,
        stage,
        canonical_bytes: canonical,
        hash,
    })
}

pub fn canonical_source_manifest_json(
    manifest: &SourceManifest,
) -> Result<Vec<u8>, SourceManifestError> {
    let strict = strict_manifest_value(manifest)?;
    canonical_json_bytes(&strict).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::CanonicalSize,
            SourceManifestErrorCode::Limit,
            error.to_string(),
        )
    })
}

pub fn input_set_hash(inputs: &[InputEntry]) -> Result<LowercaseSha256, SourceManifestError> {
    let bytes = serde_json::to_vec(inputs).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Inputs,
            SourceManifestErrorCode::InputSetHash,
            error.to_string(),
        )
    })?;
    let strict =
        parse_strict_json(&bytes, SOURCE_MANIFEST_JSON_LIMITS).map_err(map_transport_error)?;
    let digest = hash_canonical_json(INPUT_SET_HASH_DOMAIN, &strict).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Inputs,
            SourceManifestErrorCode::InputSetHash,
            error.to_string(),
        )
    })?;
    parse_hash(&digest.to_hex())
}

pub fn source_manifest_hash(
    manifest: &SourceManifest,
) -> Result<LowercaseSha256, SourceManifestError> {
    let strict = strict_manifest_value(manifest)?;
    recompute_manifest_hash_from_value(&strict)
}

pub fn validate_source_manifest_canonical_size(count: u64) -> Result<(), SourceManifestError> {
    if count > SOURCE_MANIFEST_CANONICAL_BYTES_MAX {
        Err(SourceManifestError::new(
            SourceManifestValidationPhase::CanonicalSize,
            SourceManifestErrorCode::Limit,
            "source-manifest canonical byte limit exceeded",
        ))
    } else {
        Ok(())
    }
}

pub fn validate_source_manifest_input_count(
    source_language: SourceLanguage,
    count: u64,
) -> Result<(), SourceManifestError> {
    let maximum = match source_language {
        SourceLanguage::Go => SOURCE_MANIFEST_GO_INPUTS_MAX,
        SourceLanguage::Rust => SOURCE_MANIFEST_RUST_INPUTS_MAX,
    };
    if count > maximum {
        Err(SourceManifestError::new(
            SourceManifestValidationPhase::Transport,
            SourceManifestErrorCode::Limit,
            "source-manifest input count exceeds the language limit",
        ))
    } else {
        Ok(())
    }
}

pub fn validate_manifest_normalized_path(path: &str) -> Result<(), SourceManifestError> {
    if path.len() > NORMALIZED_PATH_BYTES_MAX || !is_portable_normalized_path(path) {
        Err(SourceManifestError::new(
            SourceManifestValidationPhase::Scalar,
            SourceManifestErrorCode::Path,
            format!("nonportable normalized path {path:?}"),
        ))
    } else {
        Ok(())
    }
}

pub fn validate_language_configuration(
    source_language: SourceLanguage,
    semantic_profile: SemanticProfile,
    configuration: &LanguageConfiguration,
) -> Result<(), SourceManifestError> {
    if semantic_profile.source_language() != source_language {
        return Err(invalid_profile(
            SourceManifestValidationPhase::Semantic,
            "language/profile mismatch",
        ));
    }
    if !matches!(
        (source_language, configuration),
        (SourceLanguage::Go, LanguageConfiguration::Go { .. })
            | (SourceLanguage::Rust, LanguageConfiguration::Rust { .. })
    ) {
        return Err(shape(
            "language configuration uses the wrong closed language branch",
        ));
    }
    match (source_language, semantic_profile, configuration) {
        (
            SourceLanguage::Go,
            SemanticProfile::GoFixedV0,
            LanguageConfiguration::Go {
                compiler,
                cgo_enabled,
                go111module,
                module_mode,
                workspace_mode,
                tests,
                build_tags,
                environment_profile_id,
                argument_profile_id,
            },
        ) if compiler == "gc"
            && !cgo_enabled
            && go111module == "on"
            && module_mode == "readonly"
            && workspace_mode == "off"
            && !tests
            && build_tags.is_empty()
            && environment_profile_id == "mpk.go.frontend_environment.v0"
            && argument_profile_id == "mpk.go.frontend_arguments.v0" =>
        {
            Ok(())
        }
        (
            SourceLanguage::Rust,
            SemanticProfile::RustCheckedV0,
            LanguageConfiguration::Rust {
                edition,
                crate_type,
                enabled_features,
                locked,
                offline,
                default_features,
                overflow_checks,
                panic,
                debug_assertions,
                rustc_opt_level,
                mir_opt_level,
                jobs,
                message_format,
                target_allowlist_id,
                environment_profile_id,
                argument_profile_id,
                cfg,
                ..
            },
        ) if edition == "2021"
            && crate_type == "lib"
            && enabled_features.is_empty()
            && *locked
            && *offline
            && !default_features
            && *overflow_checks
            && panic == "abort"
            && !debug_assertions
            && *rustc_opt_level == 0
            && *mir_opt_level == 0
            && *jobs == 1
            && message_format == "json"
            && target_allowlist_id == "mpk.rust.targets.v0"
            && environment_profile_id == "mpk.rust.frontend_environment.v0"
            && argument_profile_id == "mpk.rust.frontend_arguments.v0"
            && cfg.len() <= SOURCE_MANIFEST_CFG_ENTRIES_MAX as usize
            && cfg.iter().all(|value| valid_cfg(value)) =>
        {
            Ok(())
        }
        _ => Err(invalid_profile(
            SourceManifestValidationPhase::Semantic,
            "language configuration does not match the selected profile",
        )),
    }
}

pub fn validate_component_identity(
    component: &ComponentIdentity,
    source_language: SourceLanguage,
    compiler_release: Option<&str>,
    rustc_commit: Option<&str>,
) -> Result<(), SourceManifestError> {
    validate_component_shape(component, source_language)?;
    validate_component_scalars(component)?;
    match component {
        ComponentIdentity::Executable {
            name,
            release,
            commit_hash,
            ..
        } => {
            if source_language == SourceLanguage::Rust && name == "rustc" {
                let Some(commit) = commit_hash.as_deref() else {
                    return Err(shape("rustc executable identity requires commit_hash"));
                };
                if compiler_release.is_some_and(|expected| expected != release)
                    || rustc_commit.is_some_and(|expected| expected != commit)
                {
                    return Err(release_error("rustc compiler identity mismatch"));
                }
            }
        }
        ComponentIdentity::Content { .. } => {}
    }
    Ok(())
}

fn parse_manifest_transport(input: &[u8]) -> Result<StrictJsonValue, SourceManifestError> {
    parse_strict_json(input, SOURCE_MANIFEST_JSON_LIMITS).map_err(map_transport_error)
}

fn map_transport_error(error: StrictJsonError) -> SourceManifestError {
    let code = if matches!(error, StrictJsonError::DuplicateObjectName { .. }) {
        SourceManifestErrorCode::JsonDuplicateKey
    } else {
        SourceManifestErrorCode::JsonInvalid
    };
    SourceManifestError::new(
        SourceManifestValidationPhase::Transport,
        code,
        error.to_string(),
    )
}

fn ensure_stage(
    value: &StrictJsonValue,
    expected: SourceManifestStage,
) -> Result<(), SourceManifestError> {
    let has_vc_hash = value.get("vc_hash").is_some();
    if has_vc_hash != (expected == SourceManifestStage::Certificate) {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Shape,
            SourceManifestErrorCode::Stage,
            "vc_hash presence does not match the requested lifecycle stage",
        ));
    }
    Ok(())
}

fn validate_transport_counts(value: &StrictJsonValue) -> Result<(), SourceManifestError> {
    let language = match value
        .get("source_language")
        .and_then(StrictJsonValue::as_str)
    {
        Some("go") => Some(SourceLanguage::Go),
        Some("rust") => Some(SourceLanguage::Rust),
        _ => None,
    };
    if let Some(inputs) = value.get("inputs").and_then(StrictJsonValue::as_array) {
        let maximum = match language {
            Some(SourceLanguage::Rust) => SOURCE_MANIFEST_RUST_INPUTS_MAX,
            Some(SourceLanguage::Go) | None => SOURCE_MANIFEST_GO_INPUTS_MAX,
        };
        if inputs.len() as u64 > maximum {
            return Err(limit_transport(
                "source-manifest input count exceeds the language limit",
            ));
        }
        if language == Some(SourceLanguage::Rust) {
            let source_count = inputs
                .iter()
                .filter(|input| {
                    input.get("kind").and_then(StrictJsonValue::as_str) == Some("source")
                })
                .count() as u64;
            if source_count > 256 {
                return Err(limit_transport("Rust compiled-source input limit exceeded"));
            }
        }
    }
    validate_other_transport_counts(value, language)
}

fn validate_other_transport_counts(
    value: &StrictJsonValue,
    language: Option<SourceLanguage>,
) -> Result<(), SourceManifestError> {
    validate_array_limit(
        value.get("units"),
        SOURCE_MANIFEST_UNITS_MAX,
        "manifest unit limit exceeded",
    )?;
    validate_array_limit(
        value
            .get("toolchain")
            .and_then(|toolchain| toolchain.get("components")),
        SOURCE_MANIFEST_TOOLCHAIN_COMPONENTS_MAX,
        "toolchain component limit exceeded",
    )?;
    let subordinate_maximum = match language {
        Some(SourceLanguage::Go) => 0,
        Some(SourceLanguage::Rust) | None => 1,
    };
    validate_array_limit(
        value
            .get("frontend")
            .and_then(|frontend| frontend.get("subordinate_binaries")),
        subordinate_maximum,
        "frontend subordinate limit exceeded",
    )?;
    validate_array_limit(
        value
            .get("target")
            .and_then(|target| target.get("language_configuration"))
            .and_then(|configuration| configuration.get("cfg")),
        SOURCE_MANIFEST_CFG_ENTRIES_MAX,
        "Rust cfg entry limit exceeded",
    )
}

fn validate_array_limit(
    value: Option<&StrictJsonValue>,
    maximum: u64,
    detail: &str,
) -> Result<(), SourceManifestError> {
    if value
        .and_then(StrictJsonValue::as_array)
        .is_some_and(|array| array.len() as u64 > maximum)
    {
        Err(limit_transport(detail))
    } else {
        Ok(())
    }
}

fn validate_shape(manifest: &SourceManifest) -> Result<(), SourceManifestError> {
    if manifest.schema != SOURCE_MANIFEST_SCHEMA_VERSION {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Shape,
            SourceManifestErrorCode::Schema,
            "unsupported source-manifest schema",
        ));
    }
    for component in &manifest.toolchain.components {
        validate_component_shape(component, manifest.source_language)?;
    }
    if manifest.units.is_empty() || manifest.inputs.is_empty() {
        return Err(shape("units and inputs must be nonempty"));
    }
    Ok(())
}

fn validate_scalars(manifest: &SourceManifest) -> Result<(), SourceManifestError> {
    for hash in [
        &manifest.release_registry.registry_sha256,
        &manifest.toolchain.distribution_sha256,
        &manifest.frontend.binary_sha256,
        &manifest.input_set_hash,
        &manifest.vir_hash,
        &manifest.source_map_hash,
        &manifest.source_manifest_hash,
    ] {
        parse_hash(hash)?;
    }
    if let Some(vc_hash) = &manifest.vc_hash {
        parse_hash(vc_hash)?;
    }
    for subordinate in &manifest.frontend.subordinate_binaries {
        parse_hash(&subordinate.binary_sha256)?;
    }
    for component in &manifest.toolchain.components {
        validate_component_scalars(component)?;
    }
    let mut folded_paths = BTreeMap::new();
    for input in &manifest.inputs {
        validate_manifest_normalized_path(&input.normalized_path)?;
        let folded = input.normalized_path.to_ascii_lowercase();
        if let Some(existing) = folded_paths.insert(folded, input.normalized_path.as_str()) {
            if existing != input.normalized_path {
                return Err(SourceManifestError::new(
                    SourceManifestValidationPhase::Scalar,
                    SourceManifestErrorCode::Path,
                    "input paths collide under ASCII case folding",
                ));
            }
        }
        if !(0..=4_294_967_296).contains(&input.size_bytes) {
            return Err(SourceManifestError::new(
                SourceManifestValidationPhase::Scalar,
                SourceManifestErrorCode::InputBytes,
                "input size is outside the v0 range",
            ));
        }
        parse_hash(&input.sha256)?;
    }
    Ok(())
}

fn validate_order(manifest: &SourceManifest) -> Result<(), SourceManifestError> {
    strictly_increasing_by(
        &manifest.frontend.subordinate_binaries,
        |item| item.name.as_bytes(),
        "frontend subordinate identities",
    )?;
    strictly_increasing_by(
        &manifest.toolchain.components,
        |item| item.name().as_bytes(),
        "toolchain component identities",
    )?;
    strictly_increasing_by(&manifest.units, |unit| unit.identity.as_bytes(), "units")?;
    for pair in manifest.inputs.windows(2) {
        let left = (
            pair[0].normalized_path.as_bytes(),
            input_kind_name(pair[0].kind).as_bytes(),
        );
        let right = (
            pair[1].normalized_path.as_bytes(),
            input_kind_name(pair[1].kind).as_bytes(),
        );
        if left >= right || pair[0].normalized_path == pair[1].normalized_path {
            return Err(order_error("inputs are duplicated or not canonical"));
        }
    }
    match &manifest.target.language_configuration {
        LanguageConfiguration::Go { build_tags, .. } => {
            strictly_increasing_strings(build_tags, "Go build tags")?;
        }
        LanguageConfiguration::Rust {
            enabled_features,
            cfg,
            ..
        } => {
            strictly_increasing_strings(enabled_features, "Rust features")?;
            strictly_increasing_strings(cfg, "Rust cfg")?;
        }
    }
    Ok(())
}

fn validate_semantics(
    manifest: &SourceManifest,
    vir: &VirModule,
    expected_language_configuration: Option<&LanguageConfiguration>,
) -> Result<(), SourceManifestError> {
    validate_semantic_context(
        manifest.source_language,
        manifest.semantic_profile,
        &manifest.semantic_parameters,
    )
    .map_err(|error| invalid_profile(SourceManifestValidationPhase::Semantic, error.to_string()))?;
    if manifest.source_language != vir.source_language
        || manifest.semantic_profile != vir.semantic_profile
        || manifest.semantic_parameters != vir.semantic_parameters
        || manifest.target.id != manifest.semantic_parameters.target_id()
        || manifest.target.pointer_width != manifest.semantic_parameters.pointer_width()
    {
        return Err(invalid_profile(
            SourceManifestValidationPhase::Semantic,
            "manifest, target, and VIR semantic identities differ",
        ));
    }
    validate_language_configuration(
        manifest.source_language,
        manifest.semantic_profile,
        &manifest.target.language_configuration,
    )?;
    match (manifest.source_language, expected_language_configuration) {
        (SourceLanguage::Rust, Some(expected))
            if expected == &manifest.target.language_configuration => {}
        (SourceLanguage::Rust, _) => {
            return Err(invalid_profile(
                SourceManifestValidationPhase::Semantic,
                "Rust manifest requires the exact final compiler-session configuration",
            ));
        }
        (SourceLanguage::Go, Some(expected))
            if expected != &manifest.target.language_configuration =>
        {
            return Err(invalid_profile(
                SourceManifestValidationPhase::Semantic,
                "expected language configuration differs from the manifest",
            ));
        }
        (SourceLanguage::Go, _) => {}
    }

    let expected_kind = match manifest.source_language {
        SourceLanguage::Go => ManifestUnitKind::Package,
        SourceLanguage::Rust => ManifestUnitKind::Lib,
    };
    if manifest.units.len() != vir.units.len()
        || manifest
            .units
            .iter()
            .zip(&vir.units)
            .any(|(manifest, vir)| {
                manifest.identity != vir.id
                    || manifest.name != vir.name
                    || manifest.kind != expected_kind
            })
    {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Semantic,
            SourceManifestErrorCode::Units,
            "manifest units do not exactly project VIR units",
        ));
    }

    match (&manifest.selection, manifest.source_language) {
        (ManifestSelection::Go(selection), SourceLanguage::Go) => {
            let Some(unit) = vir.units.iter().find(|unit| unit.id == selection.package) else {
                return Err(selection_error());
            };
            if unit
                .functions
                .iter()
                .filter(|function| function.id == selection.function)
                .count()
                != 1
            {
                return Err(selection_error());
            }
        }
        (ManifestSelection::Rust(selection), SourceLanguage::Rust) => {
            if vir.units.len() != 1
                || vir.units[0].id != selection.crate_name
                || vir.units[0].name != selection.package
                || vir.units[0]
                    .functions
                    .iter()
                    .filter(|function| function.id == selection.function)
                    .count()
                    != 1
            {
                return Err(selection_error());
            }
        }
        _ => return Err(selection_error()),
    }
    Ok(())
}

fn validate_release(
    manifest: &SourceManifest,
    registry: &ValidatedReleaseRegistry,
) -> Result<(), SourceManifestError> {
    let root = registry.registry();
    if manifest.release_registry.schema != RELEASE_REGISTRY_SCHEMA
        || manifest.release_registry.schema != root.schema
        || manifest.release_registry.id != RELEASE_REGISTRY_ID
        || manifest.release_registry.id != root.id
        || manifest.release_registry.registry_sha256 != registry.registry_digest().to_hex()
    {
        return Err(release_error("release registry projection mismatch"));
    }

    let Some(frontend) = registry.frontend_bundle(&manifest.frontend.bundle_id) else {
        return Err(release_error("unknown frontend bundle"));
    };
    let Some(toolchain) = registry.toolchain_bundle(&manifest.toolchain.bundle_id) else {
        return Err(release_error("unknown toolchain bundle"));
    };
    let language = source_language_name(manifest.source_language);
    let profile = semantic_profile_name(manifest.semantic_profile);
    let pointer_width = i64::from(manifest.target.pointer_width.bits());
    let tuple_matches = root.tuples.iter().any(|tuple| {
        tuple.source_language == language
            && tuple.semantic_profile == profile
            && tuple.target_id == manifest.target.id
            && tuple.pointer_width == pointer_width
            && tuple.limit_profile_id == manifest.limit_profile
            && tuple.frontend_bundle_id == manifest.frontend.bundle_id
            && tuple.toolchain_bundle_id == manifest.toolchain.bundle_id
    });
    if !tuple_matches
        || frontend.source_language != language
        || frontend.limit_profile_id != manifest.limit_profile
        || toolchain.source_language != language
        || manifest.frontend.name != frontend.name
        || manifest.frontend.version != frontend.version
        || manifest.frontend.binary_sha256 != frontend.main.binary_sha256
        || manifest.toolchain.distribution_sha256 != toolchain.distribution_sha256
    {
        return Err(release_error("release tuple or bundle projection mismatch"));
    }

    if manifest.frontend.subordinate_binaries.len() != frontend.subordinate_binaries.len()
        || manifest
            .frontend
            .subordinate_binaries
            .iter()
            .zip(&frontend.subordinate_binaries)
            .any(|(identity, descriptor)| {
                identity.name != descriptor.name
                    || identity.version != descriptor.version
                    || identity.binary_sha256 != descriptor.binary_sha256
            })
    {
        return Err(release_error("frontend subordinate projection mismatch"));
    }

    let (compiler_release, rustc_commit) = match &toolchain.compiler {
        CompilerIdentity::Go { release } => (release.as_str(), None),
        CompilerIdentity::Rust {
            release,
            rustc_commit,
        } => (release.as_str(), Some(rustc_commit.as_str())),
    };
    if manifest.toolchain.components.len() != toolchain.components.len() {
        return Err(release_error(
            "toolchain component projection length mismatch",
        ));
    }
    for (identity, descriptor) in manifest
        .toolchain
        .components
        .iter()
        .zip(&toolchain.components)
    {
        validate_component_identity(
            identity,
            manifest.source_language,
            Some(compiler_release),
            rustc_commit,
        )?;
        let equal = match (identity, descriptor) {
            (
                ComponentIdentity::Executable {
                    name,
                    release,
                    binary_sha256,
                    ..
                },
                ToolchainComponent::Executable {
                    name: expected_name,
                    release: expected_release,
                    binary_sha256: expected_hash,
                    ..
                },
            ) => {
                name == expected_name
                    && release == expected_release
                    && binary_sha256 == expected_hash
            }
            (
                ComponentIdentity::Content {
                    name,
                    release,
                    content_sha256,
                },
                ToolchainComponent::Content {
                    name: expected_name,
                    release: expected_release,
                    content_sha256: expected_hash,
                    ..
                },
            ) => {
                name == expected_name
                    && release == expected_release
                    && content_sha256 == expected_hash
            }
            _ => false,
        };
        if !equal {
            return Err(release_error("toolchain component projection mismatch"));
        }
    }

    let (environment_profile_id, argument_profile_id) =
        configuration_profile_ids(&manifest.target.language_configuration);
    if environment_profile_id != frontend.environment_profile_id
        || argument_profile_id != frontend.argument_profile_id
    {
        return Err(release_error("frontend configuration profile mismatch"));
    }
    Ok(())
}

fn validate_inputs(
    manifest: &SourceManifest,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), SourceManifestError> {
    for input in &manifest.inputs {
        if !profile_input_kind_matches(manifest.source_language, input) {
            return Err(SourceManifestError::new(
                SourceManifestValidationPhase::Inputs,
                SourceManifestErrorCode::InputKind,
                format!(
                    "input kind does not match profile path {:?}",
                    input.normalized_path
                ),
            ));
        }
        if input.size_bytes == 0 && !profile_allows_empty_input(manifest.source_language, input) {
            return Err(SourceManifestError::new(
                SourceManifestValidationPhase::Inputs,
                SourceManifestErrorCode::InputKind,
                "the language profile does not permit this input to be empty",
            ));
        }
    }
    if manifest.inputs.len() != captured_inputs.len() {
        return Err(input_bytes_error(
            "captured input inventory length mismatch",
        ));
    }
    for input in &manifest.inputs {
        let matches: Vec<_> = captured_inputs
            .iter()
            .filter(|captured| captured.normalized_path == input.normalized_path)
            .collect();
        if matches.len() != 1
            || matches[0].kind != input.kind
            || !captured_input_matches(
                matches[0],
                u64::try_from(input.size_bytes).unwrap_or(u64::MAX),
                &input.sha256,
            )
        {
            return Err(input_bytes_error(
                "manifest input does not match immutable captured bytes",
            ));
        }
    }
    let expected = input_set_hash(&manifest.inputs)?;
    if manifest.input_set_hash != expected.as_str() {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Inputs,
            SourceManifestErrorCode::InputSetHash,
            "input_set_hash mismatch",
        ));
    }
    Ok(())
}

fn validate_artifacts(
    manifest: &SourceManifest,
    vir: &VirModule,
    source_map: &ValidatedSourceMap,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), SourceManifestError> {
    let expected_vir_hash = vir_hash(vir).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Artifacts,
            SourceManifestErrorCode::IrLinkage,
            error.to_string(),
        )
    })?;
    if vir.schema != VIR_SCHEMA_VERSION || manifest.vir_hash != expected_vir_hash.as_str() {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Artifacts,
            SourceManifestErrorCode::IrLinkage,
            "manifest VIR hash does not match validated VIR",
        ));
    }
    if manifest.source_map_hash != source_map.hash().as_str()
        || source_map.map().source_ir_schema != vir.schema
        || source_map.map().source_ir_hash != manifest.vir_hash
    {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::Artifacts,
            SourceManifestErrorCode::SourceMapLinkage,
            "manifest source-map linkage mismatch",
        ));
    }
    for entry in &source_map.map().entries {
        let SourceOrigin::Source {
            normalized_path, ..
        } = &entry.origin
        else {
            continue;
        };
        let manifest_input = manifest.inputs.iter().find(|input| {
            input.normalized_path == *normalized_path && input.kind == InputKind::Source
        });
        let captured = captured_inputs.iter().find(|input| {
            input.normalized_path == normalized_path && input.kind == InputKind::Source
        });
        let source_map_input = source_map.captured_source_identity(normalized_path);
        let captures_match = manifest_input
            .zip(captured)
            .zip(source_map_input)
            .is_some_and(|((manifest_input, captured), source_map_input)| {
                source_map_input.size_bytes
                    == u64::try_from(manifest_input.size_bytes).unwrap_or(u64::MAX)
                    && source_map_input.sha256 == manifest_input.sha256
                    && captured_input_matches(
                        captured,
                        source_map_input.size_bytes,
                        &source_map_input.sha256,
                    )
            });
        if !captures_match {
            return Err(SourceManifestError::new(
                SourceManifestValidationPhase::Artifacts,
                SourceManifestErrorCode::SourceMapLinkage,
                "source-map origin does not use the same captured bytes as the manifest",
            ));
        }
    }
    Ok(())
}

fn validate_vc_linkage(
    manifest: &SourceManifest,
    vir: &VirModule,
    vc: &ValidatedVcIdentity,
) -> Result<(), SourceManifestError> {
    let expected_vir_hash = vir_hash(vir).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::VcLinkage,
            SourceManifestErrorCode::VcLinkage,
            error.to_string(),
        )
    })?;
    if vc.input_set_hash.as_str() != manifest.input_set_hash
        || vc.source_ir_schema != VIR_SCHEMA_VERSION
        || vc.source_ir_schema != vir.schema
        || vc.source_ir_hash != expected_vir_hash
        || vc.source_ir_hash.as_str() != manifest.vir_hash
        || vc.semantic_profile != manifest.semantic_profile
        || vc.semantic_parameters != manifest.semantic_parameters
        || manifest
            .vc_hash
            .as_deref()
            .is_some_and(|hash| hash != vc.vc_hash.as_str())
    {
        return Err(SourceManifestError::new(
            SourceManifestValidationPhase::VcLinkage,
            SourceManifestErrorCode::VcLinkage,
            "validated VC identity does not match manifest and VIR",
        ));
    }
    Ok(())
}

fn recompute_manifest_hash_from_value(
    value: &StrictJsonValue,
) -> Result<LowercaseSha256, SourceManifestError> {
    let preimage = value
        .clone_without_fields(&["source_manifest_hash"])
        .map_err(|error| shape(error.to_string()))?;
    let digest = hash_canonical_json(SOURCE_MANIFEST_HASH_DOMAIN, &preimage).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Hash,
            SourceManifestErrorCode::Hash,
            error.to_string(),
        )
    })?;
    parse_hash(&digest.to_hex())
}

fn strict_manifest_value(
    manifest: &SourceManifest,
) -> Result<StrictJsonValue, SourceManifestError> {
    let bytes = serde_json::to_vec(manifest).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Shape,
            SourceManifestErrorCode::Shape,
            error.to_string(),
        )
    })?;
    parse_strict_json(&bytes, SOURCE_MANIFEST_JSON_LIMITS).map_err(map_transport_error)
}

fn parse_hash(value: &str) -> Result<LowercaseSha256, SourceManifestError> {
    LowercaseSha256::new(value.to_owned()).map_err(|error| {
        SourceManifestError::new(
            SourceManifestValidationPhase::Scalar,
            SourceManifestErrorCode::Shape,
            error.to_string(),
        )
    })
}

fn valid_cfg(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1_024
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'/' | b'\\' | b':'))
}

fn valid_rustc_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_component_shape(
    component: &ComponentIdentity,
    source_language: SourceLanguage,
) -> Result<(), SourceManifestError> {
    if let ComponentIdentity::Executable {
        name, commit_hash, ..
    } = component
    {
        if source_language == SourceLanguage::Rust && name == "rustc" {
            if commit_hash.is_none() {
                return Err(shape("rustc executable identity requires commit_hash"));
            }
        } else if commit_hash.is_some() {
            return Err(shape(
                "commit_hash is forbidden outside the rustc component",
            ));
        }
    }
    Ok(())
}

fn validate_component_scalars(component: &ComponentIdentity) -> Result<(), SourceManifestError> {
    match component {
        ComponentIdentity::Executable {
            commit_hash,
            binary_sha256,
            ..
        } => {
            parse_hash(binary_sha256)?;
            if commit_hash
                .as_deref()
                .is_some_and(|commit| !valid_rustc_commit(commit))
            {
                return Err(SourceManifestError::new(
                    SourceManifestValidationPhase::Scalar,
                    SourceManifestErrorCode::Shape,
                    "invalid rustc commit_hash",
                ));
            }
        }
        ComponentIdentity::Content { content_sha256, .. } => {
            parse_hash(content_sha256)?;
        }
    }
    Ok(())
}

fn profile_input_kind_matches(language: SourceLanguage, input: &InputEntry) -> bool {
    let path = input.normalized_path.as_str();
    match language {
        SourceLanguage::Go => match input.kind {
            InputKind::Source => path.ends_with(".go"),
            InputKind::Contract => path.ends_with(".json"),
            InputKind::BuildManifest => {
                matches!(path, "go.mod" | "go.work")
                    || path.ends_with("/go.mod")
                    || path.ends_with("/go.work")
            }
            InputKind::Lockfile => {
                matches!(path, "go.sum" | "go.work.sum")
                    || path.ends_with("/go.sum")
                    || path.ends_with("/go.work.sum")
            }
        },
        SourceLanguage::Rust => match input.kind {
            InputKind::Source => path.ends_with(".rs"),
            InputKind::Contract => path.ends_with(".json"),
            InputKind::BuildManifest => path == "Cargo.toml" || path.ends_with("/Cargo.toml"),
            InputKind::Lockfile => path == "Cargo.lock",
        },
    }
}

fn profile_allows_empty_input(language: SourceLanguage, input: &InputEntry) -> bool {
    matches!(
        (language, input.kind, input.normalized_path.as_str()),
        (
            SourceLanguage::Go,
            InputKind::Lockfile,
            "go.sum" | "go.work.sum"
        )
    ) || (language == SourceLanguage::Go
        && input.kind == InputKind::Lockfile
        && (input.normalized_path.ends_with("/go.sum")
            || input.normalized_path.ends_with("/go.work.sum")))
}

fn configuration_profile_ids(configuration: &LanguageConfiguration) -> (&str, &str) {
    match configuration {
        LanguageConfiguration::Go {
            environment_profile_id,
            argument_profile_id,
            ..
        }
        | LanguageConfiguration::Rust {
            environment_profile_id,
            argument_profile_id,
            ..
        } => (environment_profile_id, argument_profile_id),
    }
}

fn input_kind_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Source => "source",
        InputKind::Contract => "contract",
        InputKind::BuildManifest => "build_manifest",
        InputKind::Lockfile => "lockfile",
    }
}

fn source_language_name(language: SourceLanguage) -> &'static str {
    match language {
        SourceLanguage::Go => "go",
        SourceLanguage::Rust => "rust",
    }
}

fn semantic_profile_name(profile: SemanticProfile) -> &'static str {
    match profile {
        SemanticProfile::GoFixedV0 => "mpk.go.fixed.v0",
        SemanticProfile::RustCheckedV0 => "mpk.rust.checked.v0",
    }
}

fn strictly_increasing_by<'a, T, F>(
    values: &'a [T],
    key: F,
    name: &str,
) -> Result<(), SourceManifestError>
where
    F: Fn(&'a T) -> &'a [u8],
{
    if values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        Err(order_error(format!(
            "{name} are duplicated or not canonical"
        )))
    } else {
        Ok(())
    }
}

fn strictly_increasing_strings(values: &[String], name: &str) -> Result<(), SourceManifestError> {
    if values
        .windows(2)
        .any(|pair| pair[0].as_bytes() >= pair[1].as_bytes())
    {
        Err(order_error(format!(
            "{name} are duplicated or not canonical"
        )))
    } else {
        Ok(())
    }
}

fn shape(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Shape,
        SourceManifestErrorCode::Shape,
        detail,
    )
}

fn invalid_profile(
    phase: SourceManifestValidationPhase,
    detail: impl Into<String>,
) -> SourceManifestError {
    SourceManifestError::new(phase, SourceManifestErrorCode::Profile, detail)
}

fn order_error(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Order,
        SourceManifestErrorCode::Order,
        detail,
    )
}

fn selection_error() -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Semantic,
        SourceManifestErrorCode::Selection,
        "selection does not resolve exactly once in VIR",
    )
}

fn release_error(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Release,
        SourceManifestErrorCode::Release,
        detail,
    )
}

fn input_bytes_error(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Inputs,
        SourceManifestErrorCode::InputBytes,
        detail,
    )
}

fn lifecycle_mutation(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::VcLinkage,
        SourceManifestErrorCode::LifecycleMutation,
        detail,
    )
}

fn limit_transport(detail: impl Into<String>) -> SourceManifestError {
    SourceManifestError::new(
        SourceManifestValidationPhase::Transport,
        SourceManifestErrorCode::Limit,
        detail,
    )
}
