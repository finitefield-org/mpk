//! Untrusted GIR importer and verification-condition data model.
//!
//! `mpk-vc` does not accept proofs. It imports untrusted GIR and prepares
//! theorem-obligation data for later certificate emission.

#![forbid(unsafe_code)]

pub mod expr_encode;
pub mod gir;
pub mod loops;
pub mod obligation_emit;
pub mod policy_obligation;
pub mod policy_theory_goal;
pub mod safety;
pub mod type_encode;
pub mod vc;
pub mod wp;
pub mod wp_branch;

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
pub use safety::{generate_safety_vcs, SafetyVcGenerator};
pub use type_encode::{
    encode_gir_type, MpkTypeTerm, TypeEncodeError, TypeEncoder, STD_GO_BASE_ARRAY,
    STD_GO_BASE_ARRAY_LENGTH, STD_GO_BASE_BOOL, STD_GO_BASE_INT16, STD_GO_BASE_INT32,
    STD_GO_BASE_INT64, STD_GO_BASE_INT8, STD_GO_BASE_STRUCT_FIELD, STD_GO_BASE_STRUCT_FIELD_TYPE,
    STD_GO_BASE_STRUCT_SHAPE, STD_GO_BASE_STRUCT_VALUE, STD_GO_BASE_UINT16, STD_GO_BASE_UINT32,
    STD_GO_BASE_UINT64, STD_GO_BASE_UINT8,
};
pub use vc::{VcModule, VcObligation, VcObligationKind};
pub use wp::{generate_straight_line_vcs, WpError, WpGenerator};
pub use wp_branch::{generate_branch_vcs, BranchWpGenerator};
