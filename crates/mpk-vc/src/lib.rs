//! Untrusted GIR importer and verification-condition data model.
//!
//! `mpk-vc` does not accept proofs. It imports untrusted GIR and prepares
//! theorem-obligation data for later certificate emission.

#![forbid(unsafe_code)]

pub mod gir;
pub mod type_encode;
pub mod vc;

pub use gir::{
    import_gir_json, GirBinding, GirBlock, GirContractExpr, GirContracts, GirField, GirFieldType,
    GirFunction, GirImportError, GirInstruction, GirInstructionKind, GirIntLiteral,
    GirLoopContract, GirModule, GirPackage, GirRejectedFeature, GirTerminator, GirTerminatorKind,
    GirType, GirTypeKind, GirValue, GIR_SCHEMA_VERSION,
};
pub use type_encode::{
    encode_gir_type, MpkTypeTerm, TypeEncodeError, TypeEncoder, STD_GO_BASE_ARRAY,
    STD_GO_BASE_ARRAY_LENGTH, STD_GO_BASE_BOOL, STD_GO_BASE_INT16, STD_GO_BASE_INT32,
    STD_GO_BASE_INT64, STD_GO_BASE_INT8, STD_GO_BASE_STRUCT_FIELD, STD_GO_BASE_STRUCT_FIELD_TYPE,
    STD_GO_BASE_STRUCT_SHAPE, STD_GO_BASE_STRUCT_VALUE, STD_GO_BASE_UINT16, STD_GO_BASE_UINT32,
    STD_GO_BASE_UINT64, STD_GO_BASE_UINT8,
};
pub use vc::{VcExpr, VcModule, VcObligation, VcObligationKind};
