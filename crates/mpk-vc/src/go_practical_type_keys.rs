//! Private source-type key components with exact original declaration syntax
//! membership. Neither that membership nor hash/shape/closure checks establish
//! Go source typing, a registered capture context or source/program authority.
//!
//! Version two retains embedded-field identity and raw tag bytes. The earlier
//! private v1 draft cannot distinguish an embedded field from an explicit
//! field with the same name/type/tag and is never accepted by this importer.
use crate::canonical_json::{canonical_json_bytes, parse_strict_json, StrictJsonLimits};
use crate::hash::{hash_canonical_json, sha256_raw_file_bytes, HashDomain};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod compiler_inputs;
mod contract_inputs;
pub use compiler_inputs::{
    import_go_source_compiler_inputs, parse_go_executable_build_info_text,
    read_go_executable_build_info, GoCompilerFileBuffer, GoExecutableBuildInfo, GoExecutableModule,
    GoSourceCompilerInputError, GoSourceCompilerInputs,
};
mod export_inputs;
mod module_observations;
mod observation_json;
pub use export_inputs::{
    import_go_source_export_inputs, GoSourceExportInputError, GoSourceExportInputs,
};
pub use module_observations::{
    derive_go_source_graph_manifest, derive_go_source_module_manifest, import_go_module_sums,
    import_go_source_module_graph_inputs, import_go_source_module_observations,
    inspect_go_module_archive, GoModuleArchiveInventory, GoModuleArchiveMember,
    GoModuleReplacement, GoModuleRequirement, GoModuleRetraction, GoModuleSums,
    GoSourceGraphManifest, GoSourceGraphModule, GoSourceModuleData, GoSourceModuleDataError,
    GoSourceModuleGraphError, GoSourceModuleGraphInputs, GoSourceModuleManifest,
    GoSourceModuleManifestError, GoSourceModuleObservationError, GoSourceModuleObservations,
};
mod lex;
pub use contract_inputs::{
    import_go_source_contract_inputs, GoSourceContractDerivationError, GoSourceContractInputError,
    GoSourceContractInputs,
};
mod parse;
mod selection_inputs;
mod snapshot_inputs;
pub use snapshot_inputs::{
    import_go_source_capture_inputs, import_go_source_snapshot_inputs, GoSourceCaptureInputs,
    GoSourceSnapshotEntry, GoSourceSnapshotError, GoSourceSnapshotInputs,
    GoSourceSnapshotLookupError,
};
mod source_build;
mod source_constants;
pub use source_build::{
    derive_go_source_build_match, GoSourceBuildError, GoSourceBuildErrorKind, GoSourceBuildMatch,
};
mod source_declarations;
mod source_dependencies;
mod source_inputs;
pub use selection_inputs::{
    import_go_source_selection_inputs, GoSourcePhysicalRoot, GoSourceSelectionInputError,
    GoSourceSelectionInputs,
};
mod source_languages;
pub use source_languages::{
    derive_go_source_languages, GoSourceFileLanguage, GoSourceLanguageError,
    GoSourceLanguageErrorKind, GoSourceLanguages,
};
mod source_overlays;
mod source_package;
pub use source_package::{
    derive_go_source_embedded_files, derive_go_source_package_files, GoSourceAuxiliaryKind,
    GoSourceEmbeddedFiles, GoSourcePackageError, GoSourcePackageErrorKind, GoSourcePackageFiles,
};
mod source_scalar_typing;
mod source_scopes;
mod source_signatures;
mod source_types;
mod validate;
pub use lex::{lex_go_source, GoLexError, GoLexErrorKind, GoLexToken, GoLexedSource};
pub use parse::{
    parse_go_source, GoParseError, GoParseErrorKind, GoParsedSource, GoSyntaxEdge, GoSyntaxNode,
};
pub use source_constants::{
    evaluate_go_constant, evaluate_go_constant_binding, evaluate_go_constant_declaration,
    GoConstantError, GoConstantKind, GoConstantScalarType, GoEvaluatedConstant,
};
pub use source_declarations::{GoBoundTypeDeclaration, GoSourceTypeDeclarations};
pub use source_inputs::{import_go_source_inputs, GoSourceInputs};
pub use source_overlays::{
    bind_go_source_overlay, derive_go_source_contract_module, derive_go_source_overlay_selection,
    derive_go_typed_source_overlays, GoBoundOverlayLoop, GoBoundSourceOverlay,
    GoCapturedFunctionOverlay, GoContractExpression, GoContractExpressionReference,
    GoContractExpressionType, GoContractHeaderBinding, GoContractObservation, GoContractPhase,
    GoContractTypingError, GoContractTypingErrorKind, GoSourceContractModule,
    GoSourceContractOrigin, GoSourceFunctionContract, GoSourceFunctionLoopContract,
    GoSourceFunctionSelector, GoSourceLoopCandidateOrigin, GoSourceOverlay, GoSourceOverlayError,
    GoSourceOverlayErrorKind, GoSourceOverlaySelection, GoSourceReceiverSelector,
    GoTypedOverlayLoop, GoTypedSourceOverlay, GoTypedSourceOverlaySet,
};
pub use source_scalar_typing::{
    check_go_assignment, check_go_assignment_statement, check_go_function_body,
    check_go_return_statement, check_go_scalar_assignment, derive_go_source_call_sites,
    derive_go_source_entry_layout, derive_go_source_entry_layouts,
    derive_go_source_error_selection, derive_go_source_selection,
    derive_go_source_string_operations, derive_go_source_value_domain, execute_go_source,
    infer_go_binding, infer_go_results, infer_go_scalar_binding, infer_go_scalar_results,
    infer_go_scalar_value, infer_go_value, GoAssignmentTarget, GoAssignmentTyping, GoBodyBranch,
    GoBodyConstant, GoBodyErrorKind, GoBodyExpression, GoBodyStatement, GoBodyTyping,
    GoBodyTypingError, GoBoundErrorEntry, GoErrorBoxing, GoErrorBoxingProjection,
    GoErrorEntryConstraint, GoErrorEntryStep, GoErrorImplementation, GoMethodCallHeader,
    GoResolvedErrorEntryStep, GoReturnTyping, GoScalarCallHeader, GoScalarTyping,
    GoScalarTypingError, GoSliceBuiltinHeader, GoSourceCallArgument, GoSourceCallArgumentValue,
    GoSourceCallSite, GoSourceCallTarget, GoSourceDomainError, GoSourceDomainErrorKind,
    GoSourceEntryBinding, GoSourceEntryBindingOrigin, GoSourceEntryLayout, GoSourceEntrySlot,
    GoSourceErrorCall, GoSourceErrorDispatch, GoSourceErrorSelection, GoSourceErrorSelectionError,
    GoSourceExecution, GoSourceExecutionError, GoSourceExecutionErrorKind, GoSourceExecutionEvent,
    GoSourceExecutionEventKind, GoSourceExecutionStatus, GoSourceMethod, GoSourceRuntimeData,
    GoSourceRuntimeFault, GoSourceRuntimeSlice, GoSourceRuntimeValue, GoSourceScalarValue,
    GoSourceSelection, GoSourceSelectionCall, GoSourceSelectionDeclaration,
    GoSourceSelectionDependency, GoSourceSelectionError, GoSourceSelectionErrorKind,
    GoSourceSelectionRef, GoSourceSelectionRole, GoSourceStringOperation,
    GoSourceStringOperationKind, GoSourceTypedValue, GoSourceValueDomain, GoSourceValueProperties,
    GoValueTyping,
};
pub use source_scopes::{
    build_go_source_scopes, GoNameLookup, GoSourceNameBinding, GoSourceNameKind,
    GoSourceScopeError, GoSourceScopes,
};
pub use source_signatures::{
    derive_go_declared_binding_type, derive_go_source_signature, GoDeclaredSourceBindingType,
    GoSourceSignature, GoSourceSignatureError, GoSourceSignatureRole, GoSourceSignatureSlot,
};
pub use source_types::{
    check_go_source_type_declarations, check_go_source_type_declarations_with_scopes,
    derive_go_source_type, derive_go_source_type_declaration, GoDerivedSourceTypes,
    GoSourceTypeError,
};

const SOURCE_SCHEMA: &str = "mpk.go.source_object.v1";
const SOURCE_DOMAIN: HashDomain = HashDomain::new("MPK-GO-SOURCE-OBJECT-1.0");
const PACKAGE_SCHEMA: &str = "mpk.go.type_package.v2";
const PACKAGE_DOMAIN: HashDomain = HashDomain::new("MPK-GO-TYPE-PACKAGE-2.0");
const DECL_SCHEMA: &str = "mpk.go.type_declaration.v2";
const DECL_DOMAIN: HashDomain = HashDomain::new("MPK-GO-TYPE-DECLARATION-2.0");
const TYPE_SCHEMA: &str = "mpk.go.source_type.v2";
const TYPE_DOMAIN: HashDomain = HashDomain::new("MPK-GO-SOURCE-TYPE-2.0");
const ROOT_SCHEMA: &str = "mpk.go.type_components.v2";
const ROOT_DOMAIN: HashDomain = HashDomain::new("MPK-GO-TYPE-COMPONENTS-2.0");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoTypeKeyError {
    Json,
    Shape,
    Identity,
    Source,
    Reference,
    Order,
    Limit,
    Cycle,
}
impl std::fmt::Display for GoTypeKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go type key component error: {self:?}")
    }
}
impl std::error::Error for GoTypeKeyError {}
type Result<T> = std::result::Result<T, GoTypeKeyError>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoTypeSourceObject {
    pub schema: String,
    pub id: String,
    pub capability: String,
    pub path: String,
    pub kind: String,
    pub raw_sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoTypeSourceBuffer {
    object: GoTypeSourceObject,
    bytes: Vec<u8>,
}
impl GoTypeSourceBuffer {
    pub fn new(capability: &str, path: &str, kind: &str, raw: &[u8]) -> Result<Self> {
        let mut object = GoTypeSourceObject {
            schema: SOURCE_SCHEMA.into(),
            id: String::new(),
            capability: capability.into(),
            path: path.into(),
            kind: kind.into(),
            raw_sha256: sha256_raw_file_bytes(raw).to_hex(),
            bytes: raw.len() as u64,
        };
        validate::source_shape(&object)?;
        object.id = identity(&object, SOURCE_DOMAIN, "id")?;
        Ok(Self {
            object,
            bytes: raw.to_vec(),
        })
    }
    pub fn import(record: &[u8], raw: &[u8]) -> Result<Self> {
        let object: GoTypeSourceObject = decode(record)?;
        let expected = Self::new(&object.capability, &object.path, &object.kind, raw)?;
        if expected.object != object {
            return Err(GoTypeKeyError::Identity);
        }
        Ok(expected)
    }
    pub fn object(&self) -> &GoTypeSourceObject {
        &self.object
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoTypePackageKey {
    pub schema: String,
    pub id: String,
    pub module_source_object_id: String,
    pub import_path: String,
    pub package_name: String,
    pub source_objects: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoTypeDeclarationKey {
    pub schema: String,
    pub id: String,
    pub package_id: String,
    pub source_object_id: String,
    pub kind: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub name: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoTypeFieldKey {
    pub name: String,
    pub unexported_package_id: Option<String>,
    pub type_id: String,
    pub tag_bytes_hex: String,
    pub embedded: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum GoSourceTypeKey {
    #[serde(rename = "builtin")]
    Builtin {
        schema: String,
        id: String,
        name: String,
        width: Option<u8>,
        signed: Option<bool>,
    },
    #[serde(rename = "nominal")]
    Nominal {
        schema: String,
        id: String,
        declaration_id: String,
    },
    #[serde(rename = "array")]
    Array {
        schema: String,
        id: String,
        length: u64,
        element: String,
    },
    #[serde(rename = "slice")]
    Slice {
        schema: String,
        id: String,
        element: String,
    },
    #[serde(rename = "struct")]
    Struct {
        schema: String,
        id: String,
        fields: Vec<GoTypeFieldKey>,
    },
}
impl GoSourceTypeKey {
    pub fn id(&self) -> &str {
        match self {
            Self::Builtin { id, .. }
            | Self::Nominal { id, .. }
            | Self::Array { id, .. }
            | Self::Slice { id, .. }
            | Self::Struct { id, .. } => id,
        }
    }
    fn schema(&self) -> &str {
        match self {
            Self::Builtin { schema, .. }
            | Self::Nominal { schema, .. }
            | Self::Array { schema, .. }
            | Self::Slice { schema, .. }
            | Self::Struct { schema, .. } => schema,
        }
    }
    fn edges(&self) -> Vec<&str> {
        match self {
            Self::Array { element, .. } | Self::Slice { element, .. } => vec![element],
            Self::Struct { fields, .. } => fields.iter().map(|f| f.type_id.as_str()).collect(),
            _ => vec![],
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoUnderlyingKey {
    pub type_id: String,
    pub underlying_type_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoAliasKey {
    pub declaration_id: String,
    pub target_type_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireComponents {
    schema: String,
    source_objects: Vec<GoTypeSourceObject>,
    packages: Vec<GoTypePackageKey>,
    declarations: Vec<GoTypeDeclarationKey>,
    types: Vec<GoSourceTypeKey>,
    underlying: Vec<GoUnderlyingKey>,
    aliases: Vec<GoAliasKey>,
    package_roots: Vec<String>,
    type_roots: Vec<String>,
    content_sha256: String,
}

/// Internally consistent candidate keys, owned bytes and exact declaration
/// syntax membership. No semantic context, typing witness or program admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoTypeKeyComponents {
    wire: WireComponents,
    canonical: Vec<u8>,
    sources: BTreeMap<String, GoTypeSourceBuffer>,
    source_declarations: GoSourceTypeDeclarations,
}
impl GoTypeKeyComponents {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn content_hash(&self) -> &str {
        &self.wire.content_sha256
    }
    pub fn types(&self) -> &[GoSourceTypeKey] {
        &self.wire.types
    }
    pub fn declarations(&self) -> &[GoTypeDeclarationKey] {
        &self.wire.declarations
    }
    pub fn aliases(&self) -> &[GoAliasKey] {
        &self.wire.aliases
    }
    pub fn type_roots(&self) -> &[String] {
        &self.wire.type_roots
    }
    pub fn source(&self, id: &str) -> Option<&GoTypeSourceBuffer> {
        self.sources.get(id)
    }
    /// Original syntax membership, not name/type resolution or a capture proof.
    pub fn source_declarations(&self) -> &GoSourceTypeDeclarations {
        &self.source_declarations
    }
}
pub fn import_go_type_key_components(
    record: &[u8],
    sources: &[GoTypeSourceBuffer],
) -> Result<GoTypeKeyComponents> {
    let wire: WireComponents = decode(record)?;
    validate::components(&wire, sources)?;
    let source_declarations = source_declarations::bind(&wire, sources)?;
    let value = crate::canonical_json::parse_strict_json(record, limits())
        .map_err(|_| GoTypeKeyError::Json)?;
    let canonical = canonical_json_bytes(&value).map_err(|_| GoTypeKeyError::Json)?;
    Ok(GoTypeKeyComponents {
        wire,
        canonical,
        sources: sources
            .iter()
            .map(|s| (s.object.id.clone(), s.clone()))
            .collect(),
        source_declarations,
    })
}
fn limits() -> StrictJsonLimits {
    StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216)
}
fn decode<T: serde::de::DeserializeOwned + Serialize>(record: &[u8]) -> Result<T> {
    let original = parse_strict_json(record, limits()).map_err(|_| GoTypeKeyError::Json)?;
    let value: T = serde_json::from_slice(record).map_err(|_| GoTypeKeyError::Shape)?;
    let encoded = serde_json::to_vec(&value).map_err(|_| GoTypeKeyError::Shape)?;
    let expected = parse_strict_json(&encoded, limits()).map_err(|_| GoTypeKeyError::Json)?;
    // Option fields are required nullable fields, not defaulted omissions.
    if canonical_json_bytes(&original).map_err(|_| GoTypeKeyError::Json)?
        != canonical_json_bytes(&expected).map_err(|_| GoTypeKeyError::Json)?
    {
        return Err(GoTypeKeyError::Shape);
    }
    Ok(value)
}
fn identity<T: Serialize>(record: &T, domain: HashDomain, field: &str) -> Result<String> {
    let bytes = serde_json::to_vec(record).map_err(|_| GoTypeKeyError::Shape)?;
    let value = parse_strict_json(&bytes, limits()).map_err(|_| GoTypeKeyError::Json)?;
    let body = value
        .clone_without_fields(&[field])
        .map_err(|_| GoTypeKeyError::Shape)?;
    Ok(hash_canonical_json(domain, &body)
        .map_err(|_| GoTypeKeyError::Identity)?
        .to_hex())
}
