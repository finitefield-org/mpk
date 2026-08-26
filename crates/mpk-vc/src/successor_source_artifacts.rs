//! Inactive successor source-artifact models and hash domains.
//!
//! These parsers are available only through explicit staging calls. They do
//! not discover a registry, accept a compatibility representation, or alter
//! any active VIR, source-map, source-manifest, frontend, VC, or checker path.
//! Profile-specific producer conformance remains with the later profile
//! owners; this module seals the common successor shapes, identities, hashes,
//! ordering, and cross-artifact links they must consume.

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, serialize_json_bounded, StrictJsonLimits,
    StrictJsonValue,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};
use crate::semantic_profile::PointerWidth;
use crate::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_registry_semantic_context,
    validate_semantic_context_linkage, CompiledSemanticProfile, SelectionEnvelope, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use crate::source_manifest::{
    input_set_hash, FrontendIdentity, InputEntry, ManifestUnitKind, ReleaseRegistryIdentity,
    ToolchainIdentity, SOURCE_MANIFEST_CANONICAL_BYTES_MAX, SOURCE_MANIFEST_GO_INPUTS_MAX,
    SOURCE_MANIFEST_RUST_INPUTS_MAX, SOURCE_MANIFEST_TOOLCHAIN_COMPONENTS_MAX,
    SOURCE_MANIFEST_UNITS_MAX,
};
use crate::source_map::{
    validate_normalized_path, CapturedInput, InputKind, SourceMapEntry, SourceOrigin,
    SourceReference, SyntheticPermission, SOURCE_MAP_CANONICAL_BYTES_MAX, SOURCE_MAP_ENTRIES_MAX,
};
use crate::vir::{
    LowercaseSha256, VirBinding, VirBlock, VirConstDecl, VirContractExpr, VirFeature,
    VirInstruction, VirLoopContract, VirPanicPolicy, VirStructDecl, VirTermination,
    VIR_INPUT_JSON_BYTES_MAX, VIR_JSON_NESTING_MAX, VIR_STRING_BYTES_MAX,
};
use crate::vir_validate::VIR_CANONICAL_JSON_BYTES_MAX;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const SUCCESSOR_VIR_SCHEMA: &str = "mpk.vir.v1";
pub const SUCCESSOR_SOURCE_MAP_SCHEMA: &str = "mpk.source_map.v1";
pub const SUCCESSOR_SOURCE_MANIFEST_SCHEMA: &str = "mpk.source_manifest.v1";
pub const SUCCESSOR_RELEASE_REGISTRY_SCHEMA: &str = "mpk.release.bundle_registry.v1";
pub const SUCCESSOR_RELEASE_REGISTRY_ID: &str = "mpk.release.registry.v1";

pub const SUCCESSOR_CONTRACT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CONTRACT-1.0");
pub const SUCCESSOR_VIR_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-VIR-1.0");
pub const SUCCESSOR_SOURCE_MAP_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MAP-1.0");
pub const SUCCESSOR_SOURCE_MANIFEST_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SOURCE-MANIFEST-1.0");

const VIR_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_JSON_NESTING_MAX,
    VIR_STRING_BYTES_MAX,
);
const SOURCE_MAP_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, SOURCE_MAP_CANONICAL_BYTES_MAX, 256, 1_048_576);
const SOURCE_MANIFEST_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    268_435_456,
    SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
    256,
    1_048_576,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorArtifactKind {
    Contract,
    Vir,
    SourceMap,
    SourceManifest,
}

impl SuccessorArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::Vir => "vir",
            Self::SourceMap => "source_map",
            Self::SourceManifest => "source_manifest",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorArtifactValidationPhase {
    Transport,
    Shape,
    Identity,
    Order,
    Linkage,
    CanonicalSize,
    Hash,
}

impl SuccessorArtifactValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Identity => "identity",
            Self::Order => "order",
            Self::Linkage => "linkage",
            Self::CanonicalSize => "canonical_size",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorArtifactErrorCode {
    Json,
    Shape,
    Schema,
    SemanticContext,
    Selection,
    Order,
    Linkage,
    Limit,
    Hash,
}

impl SuccessorArtifactErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "SUCCESSOR_ARTIFACT_JSON",
            Self::Shape => "SUCCESSOR_ARTIFACT_SHAPE",
            Self::Schema => "SUCCESSOR_ARTIFACT_SCHEMA",
            Self::SemanticContext => "SUCCESSOR_SEMANTIC_CONTEXT",
            Self::Selection => "SUCCESSOR_SELECTION",
            Self::Order => "SUCCESSOR_ARTIFACT_ORDER",
            Self::Linkage => "SUCCESSOR_ARTIFACT_LINKAGE",
            Self::Limit => "SUCCESSOR_ARTIFACT_LIMIT",
            Self::Hash => "SUCCESSOR_ARTIFACT_HASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorArtifactError {
    artifact: SuccessorArtifactKind,
    phase: SuccessorArtifactValidationPhase,
    code: SuccessorArtifactErrorCode,
}

impl SuccessorArtifactError {
    pub const fn artifact(&self) -> SuccessorArtifactKind {
        self.artifact
    }

    pub const fn phase(&self) -> SuccessorArtifactValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorArtifactErrorCode {
        self.code
    }
}

impl fmt::Display for SuccessorArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} for {} at {}",
            self.code.as_str(),
            self.artifact.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for SuccessorArtifactError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVirModule {
    schema: String,
    semantic_context: SemanticContext,
    units: Vec<SuccessorVirUnit>,
    vir_hash: LowercaseSha256,
}

impl SuccessorVirModule {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn units(&self) -> &[SuccessorVirUnit] {
        &self.units
    }

    pub fn vir_hash(&self) -> &LowercaseSha256 {
        &self.vir_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVirUnit {
    id: String,
    name: String,
    type_decls: Vec<VirStructDecl>,
    const_decls: Vec<VirConstDecl>,
    functions: Vec<SuccessorVirFunction>,
}

impl SuccessorVirUnit {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_decls(&self) -> &[VirStructDecl] {
        &self.type_decls
    }

    pub fn const_decls(&self) -> &[VirConstDecl] {
        &self.const_decls
    }

    pub fn functions(&self) -> &[SuccessorVirFunction] {
        &self.functions
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVirFunction {
    id: String,
    unit_id: String,
    name: String,
    params: Vec<VirBinding>,
    results: Vec<VirBinding>,
    locals: Vec<VirBinding>,
    blocks: Vec<VirBlock>,
    contracts: SuccessorVirContract,
    features_used: Vec<VirFeature>,
}

impl SuccessorVirFunction {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn unit_id(&self) -> &str {
        &self.unit_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[VirBinding] {
        &self.params
    }

    pub fn results(&self) -> &[VirBinding] {
        &self.results
    }

    pub fn locals(&self) -> &[VirBinding] {
        &self.locals
    }

    pub fn blocks(&self) -> &[VirBlock] {
        &self.blocks
    }

    pub fn contracts(&self) -> &SuccessorVirContract {
        &self.contracts
    }

    pub fn features_used(&self) -> &[VirFeature] {
        &self.features_used
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorVirContract {
    semantic_context: SemanticContext,
    unit_id: String,
    function_id: String,
    requires: Vec<VirContractExpr>,
    ensures: Vec<VirContractExpr>,
    modifies: Vec<String>,
    panic: VirPanicPolicy,
    termination: VirTermination,
    loops: Vec<VirLoopContract>,
    contract_hash: LowercaseSha256,
}

impl SuccessorVirContract {
    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn unit_id(&self) -> &str {
        &self.unit_id
    }

    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    pub fn requires(&self) -> &[VirContractExpr] {
        &self.requires
    }

    pub fn ensures(&self) -> &[VirContractExpr] {
        &self.ensures
    }

    pub fn modifies(&self) -> &[String] {
        &self.modifies
    }

    pub const fn panic(&self) -> VirPanicPolicy {
        self.panic
    }

    pub const fn termination(&self) -> VirTermination {
        self.termination
    }

    pub fn loops(&self) -> &[VirLoopContract] {
        &self.loops
    }

    pub fn contract_hash(&self) -> &LowercaseSha256 {
        &self.contract_hash
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorVir {
    module: SuccessorVirModule,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorVir {
    pub fn module(&self) -> &SuccessorVirModule {
        &self.module
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.module.vir_hash
    }

    pub fn function(&self, id: &str) -> Option<&SuccessorVirFunction> {
        self.module
            .units
            .iter()
            .flat_map(|unit| &unit.functions)
            .find(|function| function.id == id)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVirModule {
    schema: String,
    semantic_context: Value,
    units: Vec<WireSuccessorVirUnit>,
    vir_hash: LowercaseSha256,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVirUnit {
    id: String,
    name: String,
    type_decls: Vec<VirStructDecl>,
    const_decls: Vec<VirConstDecl>,
    functions: Vec<WireSuccessorVirFunction>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVirFunction {
    id: String,
    unit_id: String,
    name: String,
    params: Vec<VirBinding>,
    results: Vec<VirBinding>,
    locals: Vec<VirBinding>,
    blocks: Vec<VirBlock>,
    contracts: WireSuccessorVirContract,
    features_used: Vec<VirFeature>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorVirContract {
    semantic_context: Value,
    unit_id: String,
    function_id: String,
    requires: Vec<VirContractExpr>,
    ensures: Vec<VirContractExpr>,
    modifies: Vec<String>,
    panic: VirPanicPolicy,
    termination: VirTermination,
    loops: Vec<VirLoopContract>,
    contract_hash: LowercaseSha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorSourceMap {
    schema: String,
    semantic_context: SemanticContext,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    entries: Vec<SourceMapEntry>,
    source_map_hash: LowercaseSha256,
}

impl SuccessorSourceMap {
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

    pub fn entries(&self) -> &[SourceMapEntry] {
        &self.entries
    }

    pub fn source_map_hash(&self) -> &LowercaseSha256 {
        &self.source_map_hash
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorSourceMap {
    map: SuccessorSourceMap,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorSourceMap {
    pub fn map(&self) -> &SuccessorSourceMap {
        &self.map
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.map.source_map_hash
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SuccessorSourceMapValidationContext<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub vir: &'a ValidatedSuccessorVir,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub synthetic_permissions: &'a [SyntheticPermission],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorSourceMap {
    schema: String,
    semantic_context: Value,
    source_ir_schema: String,
    source_ir_hash: LowercaseSha256,
    entries: Vec<SourceMapEntry>,
    source_map_hash: LowercaseSha256,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorSourceManifestStage {
    Frontend,
    Certificate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessorManifestUnitKind {
    Package,
    Lib,
    Compilation,
}

impl From<ManifestUnitKind> for SuccessorManifestUnitKind {
    fn from(value: ManifestUnitKind) -> Self {
        match value {
            ManifestUnitKind::Package => Self::Package,
            ManifestUnitKind::Lib => Self::Lib,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorManifestUnit {
    identity: String,
    name: String,
    kind: SuccessorManifestUnitKind,
}

impl SuccessorManifestUnit {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn kind(&self) -> SuccessorManifestUnitKind {
        self.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorTargetIdentity {
    id: String,
    pointer_width: PointerWidth,
}

impl SuccessorTargetIdentity {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn pointer_width(&self) -> PointerWidth {
        self.pointer_width
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorSourceManifest {
    schema: String,
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    limit_profile: String,
    release_registry: ReleaseRegistryIdentity,
    toolchain: ToolchainIdentity,
    frontend: FrontendIdentity,
    units: Vec<SuccessorManifestUnit>,
    target: SuccessorTargetIdentity,
    inputs: Vec<InputEntry>,
    input_set_hash: LowercaseSha256,
    vir_hash: LowercaseSha256,
    source_map_hash: LowercaseSha256,
    #[serde(skip_serializing_if = "Option::is_none")]
    vc_hash: Option<LowercaseSha256>,
    source_manifest_hash: LowercaseSha256,
}

impl SuccessorSourceManifest {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }

    pub fn limit_profile(&self) -> &str {
        &self.limit_profile
    }

    pub fn release_registry(&self) -> &ReleaseRegistryIdentity {
        &self.release_registry
    }

    pub fn toolchain(&self) -> &ToolchainIdentity {
        &self.toolchain
    }

    pub fn frontend(&self) -> &FrontendIdentity {
        &self.frontend
    }

    pub fn units(&self) -> &[SuccessorManifestUnit] {
        &self.units
    }

    pub fn target(&self) -> &SuccessorTargetIdentity {
        &self.target
    }

    pub fn inputs(&self) -> &[InputEntry] {
        &self.inputs
    }

    pub fn input_set_hash(&self) -> &LowercaseSha256 {
        &self.input_set_hash
    }

    pub fn vir_hash(&self) -> &LowercaseSha256 {
        &self.vir_hash
    }

    pub fn source_map_hash(&self) -> &LowercaseSha256 {
        &self.source_map_hash
    }

    pub fn vc_hash(&self) -> Option<&LowercaseSha256> {
        self.vc_hash.as_ref()
    }

    pub fn source_manifest_hash(&self) -> &LowercaseSha256 {
        &self.source_manifest_hash
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedSuccessorSourceManifest {
    manifest: SuccessorSourceManifest,
    stage: SuccessorSourceManifestStage,
    canonical_bytes: Vec<u8>,
}

impl ValidatedSuccessorSourceManifest {
    pub fn manifest(&self) -> &SuccessorSourceManifest {
        &self.manifest
    }

    pub const fn stage(&self) -> SuccessorSourceManifestStage {
        self.stage
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.manifest.source_manifest_hash
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SuccessorSourceManifestValidationContext<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub vir: &'a ValidatedSuccessorVir,
    pub source_map: &'a ValidatedSuccessorSourceMap,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub expected_release_registry: &'a ReleaseRegistryIdentity,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSuccessorSourceManifest {
    schema: String,
    semantic_context: Value,
    selection: Value,
    limit_profile: String,
    release_registry: ReleaseRegistryIdentity,
    toolchain: ToolchainIdentity,
    frontend: FrontendIdentity,
    units: Vec<SuccessorManifestUnit>,
    target: SuccessorTargetIdentity,
    inputs: Vec<InputEntry>,
    input_set_hash: LowercaseSha256,
    vir_hash: LowercaseSha256,
    source_map_hash: LowercaseSha256,
    vc_hash: Option<LowercaseSha256>,
    source_manifest_hash: LowercaseSha256,
}

pub fn import_successor_vir_json(
    input: &[u8],
    registry: &ValidatedSemanticProfileRegistry,
) -> Result<ValidatedSuccessorVir, SuccessorArtifactError> {
    let (canonical, wire) = parse_wire::<WireSuccessorVirModule>(
        input,
        VIR_LIMITS,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Vir,
    )?;
    if wire.schema != SUCCESSOR_VIR_SCHEMA {
        return Err(failure(
            SuccessorArtifactKind::Vir,
            SuccessorArtifactValidationPhase::Shape,
            SuccessorArtifactErrorCode::Schema,
        ));
    }
    let semantic_context = validate_registry_semantic_context(registry, &wire.semantic_context)
        .map_err(|_| semantic_context_failure(SuccessorArtifactKind::Vir))?;
    if wire.units.is_empty() {
        return Err(shape_failure(SuccessorArtifactKind::Vir));
    }

    let mut units = Vec::with_capacity(wire.units.len());
    let mut prior_unit: Option<String> = None;
    let mut function_ids = BTreeSet::new();
    for unit in wire.units {
        if unit.id.is_empty() || unit.name.is_empty() {
            return Err(shape_failure(SuccessorArtifactKind::Vir));
        }
        if prior_unit
            .as_deref()
            .is_some_and(|prior| prior >= unit.id.as_str())
        {
            return Err(order_failure(SuccessorArtifactKind::Vir));
        }
        prior_unit = Some(unit.id.clone());
        let mut functions = Vec::with_capacity(unit.functions.len());
        for function in unit.functions {
            if function.id.is_empty() || function.name.is_empty() {
                return Err(shape_failure(SuccessorArtifactKind::Vir));
            }
            if function.unit_id != unit.id || !function_ids.insert(function.id.clone()) {
                return Err(linkage_failure(SuccessorArtifactKind::Vir));
            }
            let contract_context =
                validate_registry_semantic_context(registry, &function.contracts.semantic_context)
                    .map_err(|_| semantic_context_failure(SuccessorArtifactKind::Contract))?;
            validate_semantic_context_linkage(&semantic_context, &contract_context)
                .map_err(|_| linkage_failure(SuccessorArtifactKind::Contract))?;
            if function.contracts.unit_id != unit.id
                || function.contracts.function_id != function.id
                || !function.contracts.modifies.is_empty()
            {
                return Err(linkage_failure(SuccessorArtifactKind::Contract));
            }
            let contract = SuccessorVirContract {
                semantic_context: contract_context,
                unit_id: function.contracts.unit_id,
                function_id: function.contracts.function_id,
                requires: function.contracts.requires,
                ensures: function.contracts.ensures,
                modifies: function.contracts.modifies,
                panic: function.contracts.panic,
                termination: function.contracts.termination,
                loops: function.contracts.loops,
                contract_hash: function.contracts.contract_hash,
            };
            let computed = successor_contract_hash(&contract)?;
            if computed != contract.contract_hash {
                return Err(hash_failure(SuccessorArtifactKind::Contract));
            }
            functions.push(SuccessorVirFunction {
                id: function.id,
                unit_id: function.unit_id,
                name: function.name,
                params: function.params,
                results: function.results,
                locals: function.locals,
                blocks: function.blocks,
                contracts: contract,
                features_used: function.features_used,
            });
        }
        units.push(SuccessorVirUnit {
            id: unit.id,
            name: unit.name,
            type_decls: unit.type_decls,
            const_decls: unit.const_decls,
            functions,
        });
    }

    let module = SuccessorVirModule {
        schema: wire.schema,
        semantic_context,
        units,
        vir_hash: wire.vir_hash,
    };
    validate_successor_call_linkage_and_order(&module)?;
    let encoded = canonical_serialize(
        &module,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Vir,
    )?;
    if encoded != canonical {
        return Err(shape_failure(SuccessorArtifactKind::Vir));
    }
    let computed = successor_vir_hash(&module)?;
    if computed != module.vir_hash {
        return Err(hash_failure(SuccessorArtifactKind::Vir));
    }
    Ok(ValidatedSuccessorVir {
        module,
        canonical_bytes: canonical,
    })
}

pub fn successor_contract_hash(
    contract: &SuccessorVirContract,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_serializable_without_field(
        contract,
        "contract_hash",
        SUCCESSOR_CONTRACT_HASH_DOMAIN,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Contract,
    )
}

pub fn successor_vir_hash(
    module: &SuccessorVirModule,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_serializable_without_field(
        module,
        "vir_hash",
        SUCCESSOR_VIR_HASH_DOMAIN,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Vir,
    )
}

pub fn successor_contract_hash_value(
    value: &Value,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_value_without_field(
        value,
        "contract_hash",
        SUCCESSOR_CONTRACT_HASH_DOMAIN,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Contract,
    )
}

pub fn successor_vir_hash_value(value: &Value) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_value_without_field(
        value,
        "vir_hash",
        SUCCESSOR_VIR_HASH_DOMAIN,
        VIR_CANONICAL_JSON_BYTES_MAX,
        SuccessorArtifactKind::Vir,
    )
}

fn validate_successor_call_linkage_and_order(
    module: &SuccessorVirModule,
) -> Result<(), SuccessorArtifactError> {
    let contracts = module
        .units
        .iter()
        .flat_map(|unit| &unit.functions)
        .map(|function| (function.id.as_str(), &function.contracts.contract_hash))
        .collect::<BTreeMap<_, _>>();
    let mut callees_by_caller = BTreeMap::new();
    for function in module.units.iter().flat_map(|unit| &unit.functions) {
        let mut callees = BTreeSet::new();
        for instruction in function.blocks.iter().flat_map(|block| &block.instructions) {
            if let VirInstruction::CallStatic {
                function,
                contract_hash,
                ..
            } = instruction
            {
                if contracts.get(function.as_str()).copied() != Some(contract_hash) {
                    return Err(linkage_failure(SuccessorArtifactKind::Vir));
                }
                callees.insert(function.as_str());
            }
        }
        callees_by_caller.insert(function.id.as_str(), callees);
    }

    let mut callers_by_callee: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut remaining_callees = BTreeMap::new();
    for (caller, callees) in &callees_by_caller {
        remaining_callees.insert(*caller, callees.len());
        for callee in callees {
            callers_by_callee.entry(callee).or_default().push(caller);
        }
    }
    let mut ready = remaining_callees
        .iter()
        .filter_map(|(function, count)| (*count == 0).then_some(*function))
        .collect::<BTreeSet<_>>();
    let mut expected = Vec::with_capacity(remaining_callees.len());
    while let Some(next) = ready.pop_first() {
        expected.push(next);
        for caller in callers_by_callee.get(next).into_iter().flatten() {
            let remaining = remaining_callees
                .get_mut(caller)
                .ok_or_else(|| linkage_failure(SuccessorArtifactKind::Vir))?;
            *remaining = remaining
                .checked_sub(1)
                .ok_or_else(|| linkage_failure(SuccessorArtifactKind::Vir))?;
            if *remaining == 0 {
                ready.insert(caller);
            }
        }
    }
    if expected.len() != contracts.len() {
        return Err(linkage_failure(SuccessorArtifactKind::Vir));
    }
    for unit in &module.units {
        let expected_unit = expected
            .iter()
            .copied()
            .filter(|id| unit.functions.iter().any(|function| function.id == **id))
            .collect::<Vec<_>>();
        let actual = unit
            .functions
            .iter()
            .map(|function| function.id.as_str())
            .collect::<Vec<_>>();
        if actual != expected_unit {
            return Err(order_failure(SuccessorArtifactKind::Vir));
        }
    }
    Ok(())
}

pub fn import_successor_source_map_json(
    input: &[u8],
    context: SuccessorSourceMapValidationContext<'_>,
) -> Result<ValidatedSuccessorSourceMap, SuccessorArtifactError> {
    let (canonical, wire) = parse_wire::<WireSuccessorSourceMap>(
        input,
        SOURCE_MAP_LIMITS,
        SOURCE_MAP_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceMap,
    )?;
    if wire.schema != SUCCESSOR_SOURCE_MAP_SCHEMA || wire.source_ir_schema != SUCCESSOR_VIR_SCHEMA {
        return Err(failure(
            SuccessorArtifactKind::SourceMap,
            SuccessorArtifactValidationPhase::Shape,
            SuccessorArtifactErrorCode::Schema,
        ));
    }
    if wire.entries.len() as u64 > SOURCE_MAP_ENTRIES_MAX {
        return Err(limit_failure(SuccessorArtifactKind::SourceMap));
    }
    let semantic_context =
        validate_registry_semantic_context(context.registry, &wire.semantic_context)
            .map_err(|_| semantic_context_failure(SuccessorArtifactKind::SourceMap))?;
    validate_semantic_context_linkage(context.vir.module().semantic_context(), &semantic_context)
        .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))?;
    if wire.source_ir_hash != *context.vir.hash() {
        return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
    }

    validate_source_map_entries(
        &wire.entries,
        context.vir,
        context.captured_inputs,
        context.synthetic_permissions,
    )?;
    let map = SuccessorSourceMap {
        schema: wire.schema,
        semantic_context,
        source_ir_schema: wire.source_ir_schema,
        source_ir_hash: wire.source_ir_hash,
        entries: wire.entries,
        source_map_hash: wire.source_map_hash,
    };
    let encoded = canonical_serialize(
        &map,
        SOURCE_MAP_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceMap,
    )?;
    if encoded != canonical {
        return Err(shape_failure(SuccessorArtifactKind::SourceMap));
    }
    let computed = successor_source_map_hash(&map)?;
    if computed != map.source_map_hash {
        return Err(hash_failure(SuccessorArtifactKind::SourceMap));
    }
    Ok(ValidatedSuccessorSourceMap {
        map,
        canonical_bytes: canonical,
    })
}

pub fn successor_source_map_hash(
    map: &SuccessorSourceMap,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_serializable_without_field(
        map,
        "source_map_hash",
        SUCCESSOR_SOURCE_MAP_HASH_DOMAIN,
        SOURCE_MAP_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceMap,
    )
}

pub fn successor_source_map_hash_value(
    value: &Value,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_value_without_field(
        value,
        "source_map_hash",
        SUCCESSOR_SOURCE_MAP_HASH_DOMAIN,
        SOURCE_MAP_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceMap,
    )
}

fn validate_source_map_entries(
    entries: &[SourceMapEntry],
    vir: &ValidatedSuccessorVir,
    captured_inputs: &[CapturedInput<'_>],
    synthetic_permissions: &[SyntheticPermission],
) -> Result<(), SuccessorArtifactError> {
    let expected = expected_source_references(vir);
    if entries.len() != expected.len() {
        return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
    }
    let mut observed = BTreeSet::new();
    let mut prior: Option<SourceReferenceOrderKey<'_>> = None;
    for entry in entries {
        let key = source_reference_order_key(&entry.reference)?;
        if prior.as_ref().is_some_and(|previous| previous >= &key)
            || !observed.insert(entry.reference.clone())
            || !expected.contains(&entry.reference)
        {
            return Err(order_failure(SuccessorArtifactKind::SourceMap));
        }
        prior = Some(key);
        match &entry.origin {
            SourceOrigin::Source {
                input_kind,
                normalized_path,
                start,
                end,
            } => {
                if *input_kind != crate::source_map::SourceInputKind::Source {
                    return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
                }
                validate_normalized_path(normalized_path)
                    .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))?;
                let source =
                    unique_captured_input(captured_inputs, InputKind::Source, normalized_path)
                        .ok_or_else(|| linkage_failure(SuccessorArtifactKind::SourceMap))?;
                let start = usize::try_from(*start)
                    .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))?;
                let end = usize::try_from(*end)
                    .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))?;
                let text = std::str::from_utf8(source.bytes)
                    .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))?;
                if start >= end
                    || end > source.bytes.len()
                    || !text.is_char_boundary(start)
                    || !text.is_char_boundary(end)
                {
                    return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
                }
            }
            SourceOrigin::Synthetic { reason } => {
                if !valid_profile_id(reason)
                    || matches!(entry.reference, SourceReference::Function { .. })
                    || !synthetic_permissions.iter().any(|permission| {
                        permission.reference == entry.reference && permission.reason == *reason
                    })
                {
                    return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
                }
            }
        }
    }
    if observed != expected {
        return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
    }
    Ok(())
}

fn expected_source_references(vir: &ValidatedSuccessorVir) -> BTreeSet<SourceReference> {
    let mut expected = BTreeSet::new();
    for unit in vir.module().units() {
        for function in unit.functions() {
            expected.insert(SourceReference::Function {
                unit_id: unit.id().to_owned(),
                function_id: function.id().to_owned(),
            });
            for block in function.blocks() {
                for instruction in &block.instructions {
                    expected.insert(SourceReference::Instruction {
                        unit_id: unit.id().to_owned(),
                        function_id: function.id().to_owned(),
                        block: block.label.clone(),
                        instruction: vir_instruction_id(instruction).to_owned(),
                    });
                }
                expected.insert(SourceReference::Terminator {
                    unit_id: unit.id().to_owned(),
                    function_id: function.id().to_owned(),
                    block: block.label.clone(),
                });
            }
        }
    }
    expected
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct SourceReferenceOrderKey<'a> {
    unit_id: &'a str,
    function_id: &'a str,
    rank: u8,
    block: i64,
    instruction: i64,
}

fn source_reference_order_key(
    reference: &SourceReference,
) -> Result<SourceReferenceOrderKey<'_>, SuccessorArtifactError> {
    match reference {
        SourceReference::Function {
            unit_id,
            function_id,
        } => Ok(SourceReferenceOrderKey {
            unit_id,
            function_id,
            rank: 0,
            block: -1,
            instruction: -1,
        }),
        SourceReference::Instruction {
            unit_id,
            function_id,
            block,
            instruction,
        } => Ok(SourceReferenceOrderKey {
            unit_id,
            function_id,
            rank: 1,
            block: i64::from(parse_dense_id(block, "bb")?),
            instruction: i64::from(parse_dense_id(instruction, "t")?),
        }),
        SourceReference::Terminator {
            unit_id,
            function_id,
            block,
        } => Ok(SourceReferenceOrderKey {
            unit_id,
            function_id,
            rank: 2,
            block: i64::from(parse_dense_id(block, "bb")?),
            instruction: -1,
        }),
    }
}

fn parse_dense_id(value: &str, prefix: &str) -> Result<u32, SuccessorArtifactError> {
    let digits = value
        .strip_prefix(prefix)
        .ok_or_else(|| linkage_failure(SuccessorArtifactKind::SourceMap))?;
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return Err(linkage_failure(SuccessorArtifactKind::SourceMap));
    }
    digits
        .parse()
        .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceMap))
}

fn valid_profile_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let mut need_alphanumeric = true;
    for byte in value.bytes() {
        if need_alphanumeric {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
                return false;
            }
            need_alphanumeric = false;
        } else if matches!(byte, b'.' | b'_' | b'-') {
            need_alphanumeric = true;
        } else if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
            return false;
        }
    }
    !need_alphanumeric
}

fn vir_instruction_id(instruction: &VirInstruction) -> &str {
    match instruction {
        VirInstruction::Const { id, .. }
        | VirInstruction::Copy { id, .. }
        | VirInstruction::BinOp { id, .. }
        | VirInstruction::UnaryOp { id, .. }
        | VirInstruction::Convert { id, .. }
        | VirInstruction::Field { id, .. }
        | VirInstruction::Index { id, .. }
        | VirInstruction::MakeStruct { id, .. }
        | VirInstruction::MakeArray { id, .. }
        | VirInstruction::CallStatic { id, .. } => id,
    }
}

pub fn import_successor_source_manifest_json(
    input: &[u8],
    stage: SuccessorSourceManifestStage,
    context: SuccessorSourceManifestValidationContext<'_>,
) -> Result<ValidatedSuccessorSourceManifest, SuccessorArtifactError> {
    let (canonical, wire) = parse_wire::<WireSuccessorSourceManifest>(
        input,
        SOURCE_MANIFEST_LIMITS,
        SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceManifest,
    )?;
    if wire.schema != SUCCESSOR_SOURCE_MANIFEST_SCHEMA {
        return Err(failure(
            SuccessorArtifactKind::SourceManifest,
            SuccessorArtifactValidationPhase::Shape,
            SuccessorArtifactErrorCode::Schema,
        ));
    }
    if wire.vc_hash.is_some() != (stage == SuccessorSourceManifestStage::Certificate) {
        return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
    }
    let semantic_context =
        validate_registry_semantic_context(context.registry, &wire.semantic_context)
            .map_err(|_| semantic_context_failure(SuccessorArtifactKind::SourceManifest))?;
    validate_semantic_context_linkage(context.vir.module().semantic_context(), &semantic_context)
        .and_then(|_| {
            validate_semantic_context_linkage(
                context.source_map.map().semantic_context(),
                &semantic_context,
            )
        })
        .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceManifest))?;
    let selection =
        validate_registry_selection_envelope(context.registry, &semantic_context, &wire.selection)
            .map_err(|_| selection_failure())?;

    validate_release_registry_identity(&wire.release_registry, context.expected_release_registry)?;
    let profile = context
        .registry
        .lookup(
            semantic_context.source_language(),
            semantic_context.semantic_profile(),
        )
        .ok_or_else(|| semantic_context_failure(SuccessorArtifactKind::SourceManifest))?
        .compiled_profile();
    validate_manifest_order(&wire, profile)?;
    validate_manifest_semantics(&wire, &semantic_context, &selection, profile, context)?;
    validate_manifest_inputs(&wire.inputs, context.captured_inputs, profile)?;

    let recomputed_input_set = input_set_hash(&wire.inputs)
        .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceManifest))?;
    if recomputed_input_set != wire.input_set_hash {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }
    if wire.vir_hash != *context.vir.hash() || wire.source_map_hash != *context.source_map.hash() {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }

    let manifest = SuccessorSourceManifest {
        schema: wire.schema,
        semantic_context,
        selection,
        limit_profile: wire.limit_profile,
        release_registry: wire.release_registry,
        toolchain: wire.toolchain,
        frontend: wire.frontend,
        units: wire.units,
        target: wire.target,
        inputs: wire.inputs,
        input_set_hash: wire.input_set_hash,
        vir_hash: wire.vir_hash,
        source_map_hash: wire.source_map_hash,
        vc_hash: wire.vc_hash,
        source_manifest_hash: wire.source_manifest_hash,
    };
    let encoded = canonical_serialize(
        &manifest,
        SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceManifest,
    )?;
    if encoded != canonical {
        return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
    }
    let computed = successor_source_manifest_hash(&manifest)?;
    if computed != manifest.source_manifest_hash {
        return Err(hash_failure(SuccessorArtifactKind::SourceManifest));
    }
    Ok(ValidatedSuccessorSourceManifest {
        manifest,
        stage,
        canonical_bytes: canonical,
    })
}

pub fn successor_source_manifest_hash(
    manifest: &SuccessorSourceManifest,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_serializable_without_field(
        manifest,
        "source_manifest_hash",
        SUCCESSOR_SOURCE_MANIFEST_HASH_DOMAIN,
        SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceManifest,
    )
}

pub fn successor_source_manifest_hash_value(
    value: &Value,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    hash_value_without_field(
        value,
        "source_manifest_hash",
        SUCCESSOR_SOURCE_MANIFEST_HASH_DOMAIN,
        SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
        SuccessorArtifactKind::SourceManifest,
    )
}

fn validate_release_registry_identity(
    actual: &ReleaseRegistryIdentity,
    expected: &ReleaseRegistryIdentity,
) -> Result<(), SuccessorArtifactError> {
    if actual != expected
        || actual.schema != SUCCESSOR_RELEASE_REGISTRY_SCHEMA
        || actual.id != SUCCESSOR_RELEASE_REGISTRY_ID
        || LowercaseSha256::new(actual.registry_sha256.clone()).is_err()
    {
        Err(linkage_failure(SuccessorArtifactKind::SourceManifest))
    } else {
        Ok(())
    }
}

fn validate_manifest_order(
    manifest: &WireSuccessorSourceManifest,
    profile: CompiledSemanticProfile,
) -> Result<(), SuccessorArtifactError> {
    let input_maximum = match profile {
        CompiledSemanticProfile::GoFixedV0 => SOURCE_MANIFEST_GO_INPUTS_MAX,
        CompiledSemanticProfile::RustCheckedV0 | CompiledSemanticProfile::CSharpScalarV0 => {
            SOURCE_MANIFEST_RUST_INPUTS_MAX
        }
    };
    if manifest.limit_profile != "mpk.vir.limits.v0"
        || manifest.units.is_empty()
        || manifest.inputs.is_empty()
        || manifest.units.len() as u64 > SOURCE_MANIFEST_UNITS_MAX
        || manifest.inputs.len() as u64 > input_maximum
        || manifest.toolchain.components.len() as u64 > SOURCE_MANIFEST_TOOLCHAIN_COMPONENTS_MAX
    {
        return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
    }
    if manifest
        .units
        .windows(2)
        .any(|pair| pair[0].identity >= pair[1].identity)
    {
        return Err(order_failure(SuccessorArtifactKind::SourceManifest));
    }
    if !valid_lower_sha256(&manifest.toolchain.distribution_sha256)
        || !valid_lower_sha256(&manifest.frontend.binary_sha256)
        || manifest
            .frontend
            .subordinate_binaries
            .iter()
            .any(|identity| !valid_lower_sha256(&identity.binary_sha256))
    {
        return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
    }
    if manifest
        .frontend
        .subordinate_binaries
        .windows(2)
        .any(|pair| pair[0].name.as_bytes() >= pair[1].name.as_bytes())
    {
        return Err(order_failure(SuccessorArtifactKind::SourceManifest));
    }
    let mut prior_component: Option<String> = None;
    for component in &manifest.toolchain.components {
        let value = serde_json::to_value(component)
            .map_err(|_| shape_failure(SuccessorArtifactKind::SourceManifest))?;
        let name = value
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| shape_failure(SuccessorArtifactKind::SourceManifest))?;
        let hash = match value.get("kind").and_then(Value::as_str) {
            Some("executable") => value.get("binary_sha256"),
            Some("content") => value.get("content_sha256"),
            _ => None,
        }
        .and_then(Value::as_str)
        .ok_or_else(|| shape_failure(SuccessorArtifactKind::SourceManifest))?;
        if !valid_lower_sha256(hash) {
            return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
        }
        if prior_component
            .as_deref()
            .is_some_and(|prior| prior.as_bytes() >= name.as_bytes())
        {
            return Err(order_failure(SuccessorArtifactKind::SourceManifest));
        }
        prior_component = Some(name.to_owned());
    }
    let mut folded_paths = BTreeMap::new();
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
            return Err(order_failure(SuccessorArtifactKind::SourceManifest));
        }
    }
    for input in &manifest.inputs {
        if !(0..=4_294_967_296).contains(&input.size_bytes) {
            return Err(shape_failure(SuccessorArtifactKind::SourceManifest));
        }
        let folded = input.normalized_path.to_ascii_lowercase();
        if folded_paths
            .insert(folded, input.normalized_path.as_str())
            .is_some_and(|prior| prior != input.normalized_path)
        {
            return Err(order_failure(SuccessorArtifactKind::SourceManifest));
        }
    }
    Ok(())
}

fn valid_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_manifest_semantics(
    manifest: &WireSuccessorSourceManifest,
    semantic_context: &SemanticContext,
    selection: &SelectionEnvelope,
    profile: CompiledSemanticProfile,
    context: SuccessorSourceManifestValidationContext<'_>,
) -> Result<(), SuccessorArtifactError> {
    let parameters = semantic_context.semantic_parameters().value();
    let target_id = parameters
        .get("target_id")
        .and_then(Value::as_str)
        .ok_or_else(|| semantic_context_failure(SuccessorArtifactKind::SourceManifest))?;
    let pointer_width = parameters
        .get("pointer_width")
        .and_then(Value::as_u64)
        .ok_or_else(|| semantic_context_failure(SuccessorArtifactKind::SourceManifest))?;
    if manifest.target.id != target_id
        || u64::from(manifest.target.pointer_width.bits()) != pointer_width
        || manifest.units.len() != context.vir.module().units().len()
    {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }
    let expected_kind = match profile {
        CompiledSemanticProfile::GoFixedV0 => SuccessorManifestUnitKind::Package,
        CompiledSemanticProfile::RustCheckedV0 => SuccessorManifestUnitKind::Lib,
        CompiledSemanticProfile::CSharpScalarV0 => SuccessorManifestUnitKind::Compilation,
    };
    if manifest
        .units
        .iter()
        .zip(context.vir.module().units())
        .any(|(manifest, vir)| {
            manifest.identity != vir.id()
                || manifest.name != vir.name()
                || manifest.kind != expected_kind
        })
    {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }
    validate_manifest_selection(selection, profile, context.vir)
}

fn validate_manifest_selection(
    selection: &SelectionEnvelope,
    profile: CompiledSemanticProfile,
    vir: &ValidatedSuccessorVir,
) -> Result<(), SuccessorArtifactError> {
    let value = selection.value();
    match profile {
        CompiledSemanticProfile::GoFixedV0 => {
            let package = value
                .get("package")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            let function = value
                .get("function")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            if !vir.module().units().iter().any(|unit| {
                unit.id() == package
                    && unit
                        .functions()
                        .iter()
                        .filter(|candidate| candidate.id() == function)
                        .count()
                        == 1
            }) {
                return Err(selection_failure());
            }
        }
        CompiledSemanticProfile::RustCheckedV0 => {
            let package = value
                .get("package")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            let crate_name = value
                .get("crate")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            let function = value
                .get("function")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            if vir.module().units().len() != 1
                || vir.module().units()[0].id() != crate_name
                || vir.module().units()[0].name() != package
                || vir.module().units()[0]
                    .functions()
                    .iter()
                    .filter(|candidate| candidate.id() == function)
                    .count()
                    != 1
            {
                return Err(selection_failure());
            }
        }
        CompiledSemanticProfile::CSharpScalarV0 => {
            let compilation = value
                .get("compilation")
                .and_then(Value::as_str)
                .ok_or_else(selection_failure)?;
            let methods = value
                .get("methods")
                .and_then(Value::as_array)
                .ok_or_else(selection_failure)?;
            if vir.module().units().len() != 1
                || vir.module().units()[0].id() != compilation
                || vir.module().units()[0].name() != compilation
                || methods.iter().any(|method| {
                    method.as_str().is_none_or(|method| {
                        vir.module().units()[0]
                            .functions()
                            .iter()
                            .filter(|candidate| candidate.id() == method)
                            .count()
                            != 1
                    })
                })
            {
                return Err(selection_failure());
            }
        }
    }
    Ok(())
}

fn validate_manifest_inputs(
    inputs: &[InputEntry],
    captured: &[CapturedInput<'_>],
    profile: CompiledSemanticProfile,
) -> Result<(), SuccessorArtifactError> {
    if inputs.len() != captured.len() {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }
    let mut observed = BTreeSet::new();
    for input in inputs {
        validate_normalized_path(&input.normalized_path)
            .map_err(|_| linkage_failure(SuccessorArtifactKind::SourceManifest))?;
        if input.size_bytes < 0
            || !input_kind_allowed(profile, input.kind)
            || !observed.insert((input_kind_name(input.kind), input.normalized_path.as_str()))
        {
            return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
        }
        let captured = unique_captured_input(captured, input.kind, &input.normalized_path)
            .ok_or_else(|| linkage_failure(SuccessorArtifactKind::SourceManifest))?;
        if u64::try_from(input.size_bytes).ok() != u64::try_from(captured.bytes.len()).ok()
            || crate::sha256_raw_file_bytes(captured.bytes).to_hex() != input.sha256
        {
            return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
        }
    }
    if !inputs.iter().any(|input| input.kind == InputKind::Source) {
        return Err(linkage_failure(SuccessorArtifactKind::SourceManifest));
    }
    Ok(())
}

fn input_kind_allowed(profile: CompiledSemanticProfile, kind: InputKind) -> bool {
    match profile {
        CompiledSemanticProfile::GoFixedV0 | CompiledSemanticProfile::RustCheckedV0 => matches!(
            kind,
            InputKind::Source
                | InputKind::Contract
                | InputKind::BuildManifest
                | InputKind::Lockfile
        ),
        CompiledSemanticProfile::CSharpScalarV0 => {
            matches!(kind, InputKind::Source | InputKind::Contract)
        }
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

fn unique_captured_input<'a>(
    inputs: &'a [CapturedInput<'a>],
    kind: InputKind,
    path: &str,
) -> Option<&'a CapturedInput<'a>> {
    let mut matches = inputs
        .iter()
        .filter(|input| input.kind == kind && input.normalized_path == path);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn parse_wire<T: for<'de> Deserialize<'de>>(
    input: &[u8],
    limits: StrictJsonLimits,
    canonical_maximum: u64,
    artifact: SuccessorArtifactKind,
) -> Result<(Vec<u8>, T), SuccessorArtifactError> {
    let strict = parse_strict_json(input, limits).map_err(|_| {
        failure(
            artifact,
            SuccessorArtifactValidationPhase::Transport,
            SuccessorArtifactErrorCode::Json,
        )
    })?;
    let canonical = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(canonical_maximum).map_err(|_| limit_failure(artifact))?,
    )
    .map_err(|_| limit_failure(artifact))?;
    let wire = serde_json::from_slice(&canonical).map_err(|_| shape_failure(artifact))?;
    Ok((canonical, wire))
}

fn canonical_serialize<T: Serialize>(
    value: &T,
    maximum: u64,
    artifact: SuccessorArtifactKind,
) -> Result<Vec<u8>, SuccessorArtifactError> {
    let maximum = usize::try_from(maximum).map_err(|_| limit_failure(artifact))?;
    let serialized = serialize_json_bounded(value, maximum).map_err(|_| limit_failure(artifact))?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(maximum as u64, maximum as u64, 256, 1_048_576),
    )
    .map_err(|_| shape_failure(artifact))?;
    canonical_json_bytes_bounded(&strict, maximum).map_err(|_| limit_failure(artifact))
}

fn hash_serializable_without_field<T: Serialize>(
    value: &T,
    excluded: &str,
    domain: HashDomain,
    maximum: u64,
    artifact: SuccessorArtifactKind,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    let maximum = usize::try_from(maximum).map_err(|_| limit_failure(artifact))?;
    let bytes = serialize_json_bounded(value, maximum).map_err(|_| limit_failure(artifact))?;
    let strict = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(maximum as u64, maximum as u64, 256, 1_048_576),
    )
    .map_err(|_| shape_failure(artifact))?;
    hash_strict_without_field(&strict, excluded, domain, maximum, artifact)
}

fn hash_value_without_field(
    value: &Value,
    excluded: &str,
    domain: HashDomain,
    maximum: u64,
    artifact: SuccessorArtifactKind,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    let maximum = usize::try_from(maximum).map_err(|_| limit_failure(artifact))?;
    let bytes = serialize_json_bounded(value, maximum).map_err(|_| limit_failure(artifact))?;
    let strict = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(maximum as u64, maximum as u64, 256, 1_048_576),
    )
    .map_err(|_| shape_failure(artifact))?;
    hash_strict_without_field(&strict, excluded, domain, maximum, artifact)
}

fn hash_strict_without_field(
    value: &StrictJsonValue,
    excluded: &str,
    domain: HashDomain,
    maximum: usize,
    artifact: SuccessorArtifactKind,
) -> Result<LowercaseSha256, SuccessorArtifactError> {
    let payload = value
        .clone_without_fields(&[excluded])
        .map_err(|_| shape_failure(artifact))?;
    let canonical =
        canonical_json_bytes_bounded(&payload, maximum).map_err(|_| limit_failure(artifact))?;
    let digest =
        hash_domain_separated_raw(domain, &canonical).map_err(|_| hash_failure(artifact))?;
    LowercaseSha256::new(digest.to_hex()).map_err(|_| hash_failure(artifact))
}

const fn failure(
    artifact: SuccessorArtifactKind,
    phase: SuccessorArtifactValidationPhase,
    code: SuccessorArtifactErrorCode,
) -> SuccessorArtifactError {
    SuccessorArtifactError {
        artifact,
        phase,
        code,
    }
}

const fn shape_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::Shape,
        SuccessorArtifactErrorCode::Shape,
    )
}

const fn semantic_context_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::Identity,
        SuccessorArtifactErrorCode::SemanticContext,
    )
}

const fn selection_failure() -> SuccessorArtifactError {
    failure(
        SuccessorArtifactKind::SourceManifest,
        SuccessorArtifactValidationPhase::Identity,
        SuccessorArtifactErrorCode::Selection,
    )
}

const fn order_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::Order,
        SuccessorArtifactErrorCode::Order,
    )
}

const fn linkage_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::Linkage,
        SuccessorArtifactErrorCode::Linkage,
    )
}

const fn limit_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::CanonicalSize,
        SuccessorArtifactErrorCode::Limit,
    )
}

const fn hash_failure(artifact: SuccessorArtifactKind) -> SuccessorArtifactError {
    failure(
        artifact,
        SuccessorArtifactValidationPhase::Hash,
        SuccessorArtifactErrorCode::Hash,
    )
}
