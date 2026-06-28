//! Untrusted GIR importer and verification-condition data model.
//!
//! `mpk-vc` does not accept proofs. It imports untrusted GIR and prepares
//! theorem-obligation data for later certificate emission.

#![forbid(unsafe_code)]

pub mod gir;
pub mod vc;

pub use gir::{
    import_gir_json, GirBinding, GirBlock, GirContractExpr, GirContracts, GirField, GirFieldType,
    GirFunction, GirImportError, GirInstruction, GirInstructionKind, GirIntLiteral,
    GirLoopContract, GirModule, GirPackage, GirRejectedFeature, GirTerminator, GirTerminatorKind,
    GirType, GirTypeKind, GirValue, GIR_SCHEMA_VERSION,
};
pub use vc::{VcExpr, VcModule, VcObligation, VcObligationKind};
