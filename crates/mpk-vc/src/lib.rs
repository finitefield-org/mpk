//! Untrusted program-IR importers and verification-condition data model.
//!
//! `mpk-vc` does not accept proofs. It imports canonical VIR and prepares
//! versioned verification-condition data for later certificate emission.

#![forbid(unsafe_code)]

pub mod call_wp;
pub mod canonical_json;
#[doc(hidden)]
pub mod csharp_practical_registry;
#[doc(hidden)]
pub mod csharp_practical_vir_model;
pub mod expr_encode;
pub mod grouping;
pub mod hash;
mod java_profile;
pub mod java_release;
mod java_source_artifacts;
pub mod program_encode;
pub mod program_wp;
mod release_bundle;
pub mod release_bundle_v1;
pub mod safety_check;
pub mod semantic_profile;
pub mod semantic_profile_registry;
pub mod source_manifest;
pub mod source_map;
pub mod successor_source_artifacts;
pub mod successor_vc;
pub mod type_encode;
pub mod vc;
pub mod vc_canonical;
pub mod vc_skeleton;
pub mod verification_limits;
pub mod vir;
pub mod vir_canonical;
pub mod vir_validate;

pub use call_wp::{
    program_declaration_name, CallWpError, ProgramCallDependencies, ProgramDeclarationKind,
    VC_FUNCTION_DECLARATION_PREFIX,
};
pub use canonical_json::{
    canonical_json_bytes, canonical_json_bytes_bounded, compare_utf16_code_units,
    normalize_unordered_set_by, normalize_unordered_utf8_strings, parse_strict_json,
    scan_strict_json, serialize_json_bounded, BoundedJsonSerializeError, CanonicalJsonError,
    ObjectFieldsError, StrictJsonError, StrictJsonEvent, StrictJsonLimits, StrictJsonObserver,
    StrictJsonPathSegment, StrictJsonValue, StrictJsonValueKind, UnorderedSetError,
    MAX_SAFE_JSON_INTEGER, MAX_SUPPORTED_JSON_DEPTH, MIN_SAFE_JSON_INTEGER,
};

pub use expr_encode::{
    MpkExprTerm, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_FALSE, STD_BOOL_IF, STD_BOOL_NOT,
    STD_BOOL_OR, STD_BOOL_TRUE, STD_EQ,
};
pub use grouping::{
    conjoin_terms, group_body, imply, member_theorem_type, GroupingError,
    STD_BOOL_AND as VC_STD_BOOL_AND, STD_BOOL_TRUE as VC_STD_BOOL_TRUE, STD_LOGIC_IMP_V1,
};
pub use hash::{
    hash_canonical_inventory, hash_canonical_json, hash_domain_separated_raw,
    sha256_raw_file_bytes, HashDomain, HashError, Sha256Digest,
};
pub use program_encode::{
    encode_vir_contract_expr, encode_vir_instruction_expr, encode_vir_value,
    evaluate_total_bitvector_operation, ProgramExprContext, ProgramExprEncodeError,
    ProgramExprEncoder, TotalBitVectorResult,
};
pub use program_wp::{
    generate_program_vcs, ProgramVcFunction, ProgramVcMember, ProgramVcMemberKind, ProgramVcModule,
    ProgramWpError, ProgramWpGenerator, VC_ASSUMPTIONS_PER_MEMBER_MAX,
    VC_EXPRESSION_NODES_PER_DOCUMENT_MAX, VC_EXPRESSION_NODES_PER_MEMBER_MAX,
    VC_MEMBERS_PER_DOCUMENT_MAX, VC_MEMBERS_PER_FUNCTION_MAX, VC_MEMBER_EXPRESSION_DEPTH_MAX,
};
pub use release_bundle::{
    registry_build_constants, validate_release_limit, validate_release_registry, BundleInventory,
    BundleRegistry, CompilerIdentity, ExecutableRecord, ExecutableRuntime, ExecutionHostProfile,
    FrontendBundle, InterpreterMount, InventoryFile, InventoryScope, LibraryMount,
    NativeRuntimeLayoutProfile, NativeRuntimeSelection, RegistryBuildConstants,
    ReleaseRegistryError, ReleaseRegistryErrorCode, ReleaseSelectionError, ReleaseSelectionRequest,
    ReleaseTuple, ReleaseValidationPhase, ResolvedRelease, RuntimeLibrary, TargetLibrary,
    ToolchainBundle, ToolchainComponent, ValidatedReleaseRegistry, BUNDLE_CONTENT_HASH_DOMAIN,
    BUNDLE_DECLARED_BYTES_MAX, BUNDLE_DESCRIPTORS_MAX, BUNDLE_FILE_BYTES_MAX,
    BUNDLE_INVENTORY_SCHEMA, BUNDLE_REGISTRY_HASH_DOMAIN, EXECUTION_HOST_PROFILES_MAX,
    FRONTEND_BUNDLE_SCHEMA, NATIVE_RUNTIME_LAYOUT_PROFILES_MAX, PORTABLE_PATH_BYTES_MAX,
    REGISTRY_CANONICAL_BYTES_MAX, REGISTRY_TRANSPORT_BYTES_MAX, RELEASE_JSON_NESTING_MAX,
    RELEASE_REGISTRY_ID, RELEASE_REGISTRY_SCHEMA, RELEASE_STRING_BYTES_MAX, RELEASE_TUPLES_MAX,
    SERIALIZED_INVENTORY_ENTRIES_MAX, TOOLCHAIN_BUNDLE_SCHEMA, TOOLCHAIN_COMPONENTS_MAX,
    UNIQUE_BUNDLE_FILES_MAX,
};
pub use safety_check::{
    encode_instruction_safety, required_safety_checks, validate_safety_check_sequence,
    EncodedSafetyPredicate, SafetyCheckError, SafetyEvidenceRoute, SafetyObligationKind,
    VirSafetyOperation, SAFETY_BITVEC_THEORY_FORMAT, SAFETY_GROUPED_CERTIFICATE_FOUNDATION,
    SAFETY_OBLIGATION_KIND_COMPONENT,
};
pub use semantic_profile::{
    validate_semantic_context, validate_semantic_parameters, GoFixedParameters, OverflowMode,
    PanicMode, PointerWidth, PointerWidthError, RustCheckedParameters, SemanticContext,
    SemanticParameters, SemanticProfile, SemanticProfileError, SourceLanguage,
};
pub use source_manifest::{
    attach_vc_hash, canonical_source_manifest_json, import_certificate_source_manifest_json,
    import_frontend_source_manifest_json, input_set_hash, source_manifest_hash,
    validate_component_identity, validate_language_configuration,
    validate_manifest_normalized_path, validate_source_manifest_canonical_size,
    validate_source_manifest_input_count, validate_source_manifest_transition, ComponentIdentity,
    FrontendIdentity, GoSelection, InputEntry, LanguageConfiguration, ManifestSelection,
    ManifestUnit, ManifestUnitKind, ReleaseRegistryIdentity, RustPrelude, RustSelection,
    RustUnitKind, SourceManifest, SourceManifestError, SourceManifestErrorCode,
    SourceManifestStage, SourceManifestValidationContext, SourceManifestValidationPhase,
    SubordinateIdentity, TargetIdentity, ToolchainIdentity, ValidatedSourceManifest,
    ValidatedVcIdentity, INPUT_SET_HASH_DOMAIN, SOURCE_MANIFEST_CANONICAL_BYTES_MAX,
    SOURCE_MANIFEST_CFG_ENTRIES_MAX, SOURCE_MANIFEST_GO_INPUTS_MAX, SOURCE_MANIFEST_HASH_DOMAIN,
    SOURCE_MANIFEST_RUST_INPUTS_MAX, SOURCE_MANIFEST_SCHEMA_VERSION,
    SOURCE_MANIFEST_TOOLCHAIN_COMPONENTS_MAX, SOURCE_MANIFEST_UNITS_MAX,
};
pub use source_map::{
    import_source_map_json, source_map_hash, validate_normalized_path,
    validate_source_map_canonical_size, validate_source_map_entry_count, CapturedInput, InputKind,
    SourceInputKind, SourceMap, SourceMapEntry, SourceMapError, SourceMapErrorCode,
    SourceMapValidationContext, SourceMapValidationPhase, SourceOrigin, SourceReference,
    SyntheticPermission, ValidatedSourceMap, NORMALIZED_PATH_BYTES_MAX,
    SOURCE_MAP_CANONICAL_BYTES_MAX, SOURCE_MAP_ENTRIES_MAX, SOURCE_MAP_HASH_DOMAIN,
    SOURCE_MAP_JSON_NESTING_MAX, SOURCE_MAP_SCHEMA_VERSION, SOURCE_MAP_STRING_BYTES_MAX,
};
pub use type_encode::{
    encode_vir_type, MpkTypeTerm, ProgramTypeEncoder, TypeEncodeError, STD_PROGRAM_BASE_ARRAY,
    STD_PROGRAM_BASE_ARRAY_LENGTH, STD_PROGRAM_BASE_BOOL, STD_PROGRAM_BASE_INT16,
    STD_PROGRAM_BASE_INT32, STD_PROGRAM_BASE_INT64, STD_PROGRAM_BASE_INT8,
    STD_PROGRAM_BASE_STRUCT_FIELD, STD_PROGRAM_BASE_STRUCT_FIELD_TYPE,
    STD_PROGRAM_BASE_STRUCT_SHAPE, STD_PROGRAM_BASE_STRUCT_VALUE, STD_PROGRAM_BASE_UINT16,
    STD_PROGRAM_BASE_UINT32, STD_PROGRAM_BASE_UINT64, STD_PROGRAM_BASE_UINT8,
};
pub use vc::{
    VcBinder, VcDocument, VcFunction, VcGroup, VcGroupKind, VcMember, VcMemberKind,
    VcSourceContext, VcSourceFunction, VcTerm, VcTermConversionError, VcTypeTerm,
    VC_SCHEMA_VERSION, VERIFICATION_LIMIT_PROFILE,
};
pub use vc_canonical::{
    canonical_vc_hash_payload, canonical_vc_json, generate_vc_v1, generate_vc_v1_from_context,
    import_vc_v1_json, vc_hash, ValidatedVcDocument, VcValidationError, VcValidationPhase,
    VC_HASH_DOMAIN,
};
pub use vc_skeleton::{
    canonical_skeleton_json, emit_validated_vc_skeleton_v1, emit_vc_skeleton_v1,
    import_vc_skeleton_v1_json, validate_policy_member_binding, GroupedTheoremDeclaration,
    GroupedTheoremType, PolicyMemberBindingError, ValidatedVcCertificateSkeleton,
    VcCertificateSkeletonV1, VcSkeletonValidationError, VcSkeletonValidationPhase,
    VC_CERT_SKELETON_V1_SCHEMA_VERSION,
};
pub use verification_limits::{
    validate_verification_limit, VerificationLimitError, VerificationLimitId,
    VC_CANONICAL_CERTIFICATE_BYTES_MAX, VC_CANONICAL_JSON_BYTES_MAX,
    VC_CANONICAL_SKELETON_JSON_BYTES_MAX, VC_GENERATED_PROOF_DEPTH_MAX,
    VC_GROUPED_THEOREM_DEPTH_MAX,
};
pub use vir::{
    import_vir_json, ArrayLength, ArrayLengthError, BitVectorWidth, BitVectorWidthError,
    DecimalInteger, DecimalIntegerError, DivRemOperation, LowercaseSha256, LowercaseSha256Error,
    OverflowOperation, VirBinaryOperator, VirBinding, VirBlock, VirConstDecl, VirContract,
    VirContractExpr, VirFeature, VirFunction, VirImportError, VirInstruction, VirInstructionKind,
    VirIntLiteral, VirLiteral, VirLoopContract, VirModule, VirSafetyCheck, VirSafetyCheckKind,
    VirStructDecl, VirStructField, VirTerminator, VirTerminatorKind, VirType, VirUnaryOperator,
    VirUnit, VirValue, VIR_INPUT_JSON_BYTES_MAX, VIR_JSON_NESTING_MAX, VIR_SCHEMA_VERSION,
    VIR_STRING_BYTES_MAX,
};
pub use vir_canonical::{
    canonical_contract_hash_payload, canonical_contract_json, canonical_vir_hash_payload,
    canonical_vir_json, contract_hash, vir_hash, VirCanonicalError, CONTRACT_HASH_DOMAIN,
    VIR_HASH_DOMAIN,
};
pub use vir_validate::{
    validate_safety_checks, validate_vir, validate_vir_const_decl_fragment,
    validate_vir_contract_expr_fragment, validate_vir_limit_count, validate_vir_safety_fragment,
    validate_vir_struct_decl_fragment, validate_vir_type_fragment, VirValidationError,
};
