//! Closed diagnostic classification for the private Rust driver boundary.
//!
//! This allowlist is intentionally narrower than the public Rust diagnostic
//! registry. The subordinate driver receives an already captured and
//! preflighted crate, so capture, metadata, and parent-owned emission
//! diagnostics are not valid private output even when their codes are
//! registered elsewhere. Source-map invariants discovered by the child retain
//! their normative `emission` phase.

use crate::driver_protocol::{DriverProtocolCode, DriverProtocolError};
use crate::limits::RustLimitId;

pub const DIAGNOSTIC_TRUNCATION_SUFFIX: &str = " [truncated]";
pub const DIAGNOSTIC_TRUNCATION_CODE: &str = "RUST_LIMIT_DIAGNOSTICS_TRUNCATED";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateDiagnostic {
    pub code: String,
    pub message: String,
    pub function_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateDiagnosticStatus {
    Rejected,
    SourceError,
    FrontendError,
}

pub(crate) fn normalize_private_diagnostics(
    diagnostics: &[PrivateDiagnostic],
    status: PrivateDiagnosticStatus,
    phase: &str,
) -> Result<Vec<PrivateDiagnostic>, DriverProtocolError> {
    let message_max = RustLimitId::NormalizedIssueMessage.maximum() as usize;
    let entry_max = RustLimitId::NormalizedIssueEntries.maximum() as usize;
    let message_total_max = RustLimitId::NormalizedIssueMessageTotal.maximum() as usize;
    let mut normalized = diagnostics
        .iter()
        .map(|diagnostic| {
            if diagnostic.message.is_empty()
                || diagnostic.message.chars().any(char::is_control)
                || diagnostic.code == DIAGNOSTIC_TRUNCATION_CODE
                || !valid_private_diagnostic(&diagnostic.code, status, phase)
            {
                return Err(DriverProtocolCode::Shape.into());
            }
            let mut diagnostic = diagnostic.clone();
            truncate_message(&mut diagnostic.message, message_max);
            Ok(diagnostic)
        })
        .collect::<Result<Vec<_>, DriverProtocolError>>()?;
    normalized.sort_by(|left, right| {
        (
            left.code.as_bytes(),
            left.message.as_bytes(),
            left.function_id.as_deref().unwrap_or("").as_bytes(),
        )
            .cmp(&(
                right.code.as_bytes(),
                right.message.as_bytes(),
                right.function_id.as_deref().unwrap_or("").as_bytes(),
            ))
    });

    let message_bytes = normalized.iter().try_fold(0_usize, |total, diagnostic| {
        total.checked_add(diagnostic.message.len())
    });
    if normalized.len() <= entry_max
        && message_bytes.is_some_and(|bytes| bytes <= message_total_max)
    {
        return Ok(normalized);
    }

    // The public ordering is rejected features followed by diagnostics.  A
    // private result has exactly one non-marker channel, so its stable order
    // is also the public combined order.  Reserve the final diagnostics slot
    // and retain the greatest prefix fitting both budgets.
    let mut retained = normalized.len().min(entry_max - 1);
    let mut retained_message_bytes = normalized[..retained]
        .iter()
        .map(|diagnostic| diagnostic.message.len())
        .sum::<usize>();
    loop {
        let omitted = normalized.len() - retained;
        let marker_message = format!("{omitted} normalized issues omitted");
        if retained_message_bytes
            .checked_add(marker_message.len())
            .is_some_and(|bytes| bytes <= message_total_max)
        {
            normalized.truncate(retained);
            normalized.push(PrivateDiagnostic {
                code: DIAGNOSTIC_TRUNCATION_CODE.to_owned(),
                message: marker_message,
                function_id: None,
            });
            return Ok(normalized);
        }
        if retained == 0 {
            return Err(DriverProtocolCode::Shape.into());
        }
        retained -= 1;
        retained_message_bytes -= normalized[retained].message.len();
    }
}

fn truncate_message(message: &mut String, maximum: usize) {
    if message.len() <= maximum {
        return;
    }
    let mut end = maximum - DIAGNOSTIC_TRUNCATION_SUFFIX.len();
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    message.truncate(end);
    message.push_str(DIAGNOSTIC_TRUNCATION_SUFFIX);
}

pub(crate) fn valid_private_diagnostic(
    code: &str,
    status: PrivateDiagnosticStatus,
    phase: &str,
) -> bool {
    use PrivateDiagnosticStatus::{FrontendError, Rejected, SourceError};

    if code == DIAGNOSTIC_TRUNCATION_CODE {
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
        | "RUST_LIMIT_CONTRACT"
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
        "RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT" => {
            status == FrontendError && phase == "lowering"
        }
        "RUST_FRONTEND_SOURCE_MAP_EXTERNAL" | "RUST_FRONTEND_SOURCE_MAP_RANGE" => {
            status == FrontendError && phase == "emission"
        }

        _ => false,
    }
}
