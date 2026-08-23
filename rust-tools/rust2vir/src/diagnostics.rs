//! Closed diagnostic classification for the private Rust driver boundary.
//!
//! This allowlist is intentionally narrower than the public Rust diagnostic
//! registry. The subordinate driver receives an already captured and
//! preflighted crate, so capture, metadata, and parent-owned emission
//! diagnostics are not valid private output even when their codes are
//! registered elsewhere. Source-map invariants discovered by the child retain
//! their normative `emission` phase.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateDiagnosticStatus {
    Rejected,
    SourceError,
    FrontendError,
}

pub(crate) fn valid_private_diagnostic(
    code: &str,
    status: PrivateDiagnosticStatus,
    phase: &str,
) -> bool {
    use PrivateDiagnosticStatus::{FrontendError, Rejected, SourceError};

    if code == "RUST_LIMIT_DIAGNOSTICS_TRUNCATED" {
        return true;
    }

    match code {
        "RUST_SOURCE_PARSE" => status == SourceError && phase == "source",
        "RUST_SOURCE_NAME"
        | "RUST_SOURCE_TYPE"
        | "RUST_SOURCE_BORROW"
        | "RUST_SOURCE_LITERAL_RANGE" => status == SourceError && phase == "typecheck",

        "RUST_SUBSET_CFG"
        | "RUST_SUBSET_MACRO"
        | "RUST_SUBSET_ATTRIBUTE"
        | "RUST_SUBSET_IMPORT"
        | "RUST_SUBSET_VISIBILITY"
        | "RUST_SUBSET_PATH"
        | "RUST_SUBSET_EXPANSION" => status == Rejected && phase == "source",

        "RUST_LIMIT_CALL_CLOSURE"
        | "RUST_LIMIT_AGGREGATE"
        | "RUST_SUBSET_IDENTIFIER"
        | "RUST_SUBSET_ITEM"
        | "RUST_SUBSET_FUNCTION_KIND"
        | "RUST_SUBSET_GENERIC"
        | "RUST_SUBSET_TRAIT"
        | "RUST_SUBSET_IMPL"
        | "RUST_SUBSET_STATIC"
        | "RUST_SUBSET_TYPE"
        | "RUST_SUBSET_DROP"
        | "RUST_SUBSET_PATTERN"
        | "RUST_SUBSET_BINDING"
        | "RUST_SUBSET_CONTROL_FLOW"
        | "RUST_SUBSET_MUTATION"
        | "RUST_SUBSET_OPERATION"
        | "RUST_SUBSET_CALL"
        | "RUST_SUBSET_PURITY"
        | "RUST_CONTRACT_JSON"
        | "RUST_CONTRACT_SCHEMA"
        | "RUST_CONTRACT_SHAPE"
        | "RUST_CONTRACT_IDENTITY"
        | "RUST_CONTRACT_DUPLICATE"
        | "RUST_CONTRACT_UNUSED"
        | "RUST_CONTRACT_MISSING"
        | "RUST_CONTRACT_RESOLUTION"
        | "RUST_CONTRACT_PROFILE"
        | "RUST_CONTRACT_TYPE"
        | "RUST_CONTRACT_OPERATOR"
        | "RUST_CONTRACT_LIMIT"
        | "RUST_CONTRACT_HASH" => status == Rejected && phase == "subset",

        "RUST_LIMIT_MIR_BLOCKS"
        | "RUST_LIMIT_MIR_STATEMENTS"
        | "RUST_LIMIT_IR"
        | "RUST_MIR_STATEMENT"
        | "RUST_MIR_RVALUE"
        | "RUST_MIR_OPERAND"
        | "RUST_MIR_PLACE"
        | "RUST_MIR_PROJECTION"
        | "RUST_MIR_TERMINATOR"
        | "RUST_MIR_ASSERTION"
        | "RUST_MIR_CHECKED_PATTERN"
        | "RUST_MIR_CALL"
        | "RUST_MIR_MOVE"
        | "RUST_MIR_CLEANUP"
        | "RUST_SEMANTICS_TYPE"
        | "RUST_SEMANTICS_TARGET"
        | "RUST_SEMANTICS_CHECK_MISSING"
        | "RUST_SEMANTICS_CHECK_EXTRA"
        | "RUST_SEMANTICS_PANIC" => status == Rejected && phase == "lowering",

        "RUST_FRONTEND_SOURCE_INVENTORY" => status == FrontendError && phase == "source",
        "RUST_TOOLCHAIN_OPTIONS" => status == FrontendError && phase == "typecheck",
        "RUST_TOOLCHAIN_ARGUMENT" => status == FrontendError && phase == "typecheck",
        "RUST_TOOLCHAIN_COMMIT" | "RUST_TOOLCHAIN_MIR_ADAPTER" => {
            status == FrontendError && phase == "lowering"
        }
        "RUST_FRONTEND_SOURCE_MAP_EXTERNAL" | "RUST_FRONTEND_SOURCE_MAP_RANGE" => {
            status == FrontendError && phase == "emission"
        }

        _ => false,
    }
}
