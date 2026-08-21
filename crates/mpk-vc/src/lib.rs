//! Untrusted program-IR importers and verification-condition data model.
//!
//! `mpk-vc` does not accept proofs. The released path still imports GIR while
//! the internal VIR foundation is built for the atomic cutover; both prepare
//! theorem-obligation data for later certificate emission.

#![forbid(unsafe_code)]

pub mod canonical_json;
pub mod expr_encode;
pub mod gir;
pub mod hash;
pub mod loops;
pub mod obligation_emit;
pub mod policy_obligation;
pub mod policy_theory_goal;
pub mod release_bundle;
pub mod safety;
pub mod semantic_profile;
pub mod type_encode;
pub mod vc;
pub mod vir;
pub mod vir_canonical;
pub mod vir_validate;
pub mod wp;
pub mod wp_branch;

pub use canonical_json::{
    canonical_json_bytes, compare_utf16_code_units, normalize_unordered_set_by,
    normalize_unordered_utf8_strings, parse_strict_json, CanonicalJsonError, ObjectFieldsError,
    StrictJsonError, StrictJsonLimits, StrictJsonValue, UnorderedSetError, MAX_SAFE_JSON_INTEGER,
    MAX_SUPPORTED_JSON_DEPTH, MIN_SAFE_JSON_INTEGER,
};

pub use expr_encode::{
    encode_contract_expr, encode_gir_value, encode_instruction_expr, ExprContext, ExprEncodeError,
    ExprEncoder, MpkExprTerm, MpkExprType, STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_FALSE,
    STD_BOOL_NOT, STD_BOOL_OR, STD_BOOL_TRUE, STD_EQ,
};
pub use gir::{
    import_gir_json, GirBinding, GirBlock, GirContractExpr, GirContracts, GirField, GirFieldType,
    GirFunction, GirImportError, GirInstruction, GirInstructionKind, GirIntLiteral,
    GirLoopContract, GirModule, GirPackage, GirRejectedFeature, GirTerminator, GirTerminatorKind,
    GirType, GirTypeKind, GirValue, GIR_SCHEMA_VERSION,
};
pub use hash::{
    hash_canonical_inventory, hash_canonical_json, hash_domain_separated_raw,
    sha256_raw_file_bytes, HashDomain, HashError, Sha256Digest,
};
pub use loops::{generate_loop_vcs, LoopVcGenerator};
pub use obligation_emit::{
    core_declaration_name, emit_theorem_obligations, theorem_type_for_obligation,
    CoreTheoremDeclarationSkeleton, ObligationEmitError, ObligationEmitter, VcCertificateSkeleton,
    STD_LOGIC_IMP, VC_CERT_SKELETON_SCHEMA_VERSION, VC_DECLARATION_PREFIX,
};
pub use policy_obligation::{
    classify_payment_policy_obligation, classify_payment_policy_obligations,
    PaymentPolicyClassificationOutcome, PaymentPolicyClassificationReport,
    PaymentPolicyClassifierPropertyStatus, PaymentPolicyEvidenceLabel,
    PaymentPolicyObligationClassification, PaymentPolicyObligationPattern, UnsupportedPropertyCode,
    UnsupportedPropertyDiagnostic, PAYMENT_OBLIGATION_CLASSIFICATION_SCHEMA,
};
pub use policy_theory_goal::{
    policy_theory_goal_from_obligation, PolicyBoolGoal, PolicyBoolTautology,
    PolicyBoolTautologyReason, PolicyLinearGoal, PolicyLinearInequality, PolicyLinearTerm,
    PolicyTheoryGoal, PolicyTheoryGoalError, PolicyTheoryGoalErrorKind, PolicyTheoryGoalKind,
    MAX_POLICY_LINEAR_VARIABLES,
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
pub use safety::{generate_safety_vcs, SafetyVcGenerator};
pub use semantic_profile::{
    validate_semantic_context, validate_semantic_parameters, GoFixedParameters, OverflowMode,
    PanicMode, PointerWidth, PointerWidthError, RustCheckedParameters, SemanticContext,
    SemanticParameters, SemanticProfile, SemanticProfileError, SourceLanguage,
};
pub use type_encode::{
    encode_gir_type, MpkTypeTerm, TypeEncodeError, TypeEncoder, STD_GO_BASE_ARRAY,
    STD_GO_BASE_ARRAY_LENGTH, STD_GO_BASE_BOOL, STD_GO_BASE_INT16, STD_GO_BASE_INT32,
    STD_GO_BASE_INT64, STD_GO_BASE_INT8, STD_GO_BASE_STRUCT_FIELD, STD_GO_BASE_STRUCT_FIELD_TYPE,
    STD_GO_BASE_STRUCT_SHAPE, STD_GO_BASE_STRUCT_VALUE, STD_GO_BASE_UINT16, STD_GO_BASE_UINT32,
    STD_GO_BASE_UINT64, STD_GO_BASE_UINT8,
};
pub use vc::{VcModule, VcObligation, VcObligationKind};
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
    validate_vir_struct_decl_fragment, validate_vir_type_fragment, VirSafetyOperation,
    VirValidationError,
};
pub use wp::{generate_straight_line_vcs, WpError, WpGenerator};
pub use wp_branch::{generate_branch_vcs, BranchWpGenerator};
