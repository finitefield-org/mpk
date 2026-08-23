//! Frozen deterministic counters for `mpk.rust.limits.v0`.
//!
//! Keeping the complete profile in one closed registry prevents the concrete
//! capture, lowering, diagnostic, and transport boundaries from silently
//! drifting apart.  Large byte limits are exercised through these counters;
//! conformance tests therefore do not need to allocate multi-gigabyte inputs.

use std::fmt;

pub const RUST_LIMIT_PROFILE_ID: &str = "mpk.rust.limits.v0";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RustLimitId {
    ManifestBytes,
    LockBytes,
    ContractFiles,
    ContractFileBytes,
    ContractTotalBytes,
    ContractClausesFunction,
    ContractNodesFunction,
    ContractNodesClosure,
    ContractExpressionDepth,
    SourceFiles,
    SourceFileBytes,
    SourceTotalBytes,
    SnapshotEntries,
    SnapshotBytes,
    PathBytes,
    ClosureFunctions,
    MirBlocksFunction,
    MirBlocksClosure,
    MirStatementsFunction,
    MirStatementsClosure,
    ArrayElements,
    StructFields,
    AggregateDepth,
    NormalizedIssueEntries,
    NormalizedIssueMessage,
    NormalizedIssueMessageTotal,
    CargoRustcStdout,
    CargoRustcStderr,
    PrivateRequestTransport,
    PrivateOutputTransport,
    VirJcs,
    SourceMapJcs,
    SourceManifestJcs,
    PublicFrontendStdout,
    PublicFrontendStderr,
}

impl RustLimitId {
    pub const ALL: [Self; 35] = [
        Self::ManifestBytes,
        Self::LockBytes,
        Self::ContractFiles,
        Self::ContractFileBytes,
        Self::ContractTotalBytes,
        Self::ContractClausesFunction,
        Self::ContractNodesFunction,
        Self::ContractNodesClosure,
        Self::ContractExpressionDepth,
        Self::SourceFiles,
        Self::SourceFileBytes,
        Self::SourceTotalBytes,
        Self::SnapshotEntries,
        Self::SnapshotBytes,
        Self::PathBytes,
        Self::ClosureFunctions,
        Self::MirBlocksFunction,
        Self::MirBlocksClosure,
        Self::MirStatementsFunction,
        Self::MirStatementsClosure,
        Self::ArrayElements,
        Self::StructFields,
        Self::AggregateDepth,
        Self::NormalizedIssueEntries,
        Self::NormalizedIssueMessage,
        Self::NormalizedIssueMessageTotal,
        Self::CargoRustcStdout,
        Self::CargoRustcStderr,
        Self::PrivateRequestTransport,
        Self::PrivateOutputTransport,
        Self::VirJcs,
        Self::SourceMapJcs,
        Self::SourceManifestJcs,
        Self::PublicFrontendStdout,
        Self::PublicFrontendStderr,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManifestBytes => "manifest_bytes",
            Self::LockBytes => "lock_bytes",
            Self::ContractFiles => "contract_files",
            Self::ContractFileBytes => "contract_file_bytes",
            Self::ContractTotalBytes => "contract_total_bytes",
            Self::ContractClausesFunction => "contract_clauses_function",
            Self::ContractNodesFunction => "contract_nodes_function",
            Self::ContractNodesClosure => "contract_nodes_closure",
            Self::ContractExpressionDepth => "contract_expression_depth",
            Self::SourceFiles => "source_files",
            Self::SourceFileBytes => "source_file_bytes",
            Self::SourceTotalBytes => "source_total_bytes",
            Self::SnapshotEntries => "snapshot_entries",
            Self::SnapshotBytes => "snapshot_bytes",
            Self::PathBytes => "path_bytes",
            Self::ClosureFunctions => "closure_functions",
            Self::MirBlocksFunction => "mir_blocks_function",
            Self::MirBlocksClosure => "mir_blocks_closure",
            Self::MirStatementsFunction => "mir_statements_function",
            Self::MirStatementsClosure => "mir_statements_closure",
            Self::ArrayElements => "array_elements",
            Self::StructFields => "struct_fields",
            Self::AggregateDepth => "aggregate_depth",
            Self::NormalizedIssueEntries => "normalized_issue_entries",
            Self::NormalizedIssueMessage => "normalized_issue_message",
            Self::NormalizedIssueMessageTotal => "normalized_issue_message_total",
            Self::CargoRustcStdout => "cargo_rustc_stdout",
            Self::CargoRustcStderr => "cargo_rustc_stderr",
            Self::PrivateRequestTransport => "private_request_transport",
            Self::PrivateOutputTransport => "private_output_transport",
            Self::VirJcs => "vir_jcs",
            Self::SourceMapJcs => "source_map_jcs",
            Self::SourceManifestJcs => "source_manifest_jcs",
            Self::PublicFrontendStdout => "public_frontend_stdout",
            Self::PublicFrontendStderr => "public_frontend_stderr",
        }
    }

    pub const fn unit(self) -> &'static str {
        match self {
            Self::ManifestBytes
            | Self::LockBytes
            | Self::ContractFileBytes
            | Self::ContractTotalBytes
            | Self::SourceFileBytes
            | Self::SourceTotalBytes
            | Self::SnapshotBytes
            | Self::PathBytes
            | Self::NormalizedIssueMessage
            | Self::NormalizedIssueMessageTotal
            | Self::VirJcs
            | Self::SourceMapJcs
            | Self::SourceManifestJcs => "bytes",
            Self::PrivateRequestTransport
            | Self::PrivateOutputTransport
            | Self::PublicFrontendStdout => "bytes_including_lf",
            Self::PublicFrontendStderr => "stream_bytes",
            Self::CargoRustcStdout | Self::CargoRustcStderr => "aggregate_stream_bytes",
            Self::ContractFiles | Self::SourceFiles => "files",
            Self::ContractClausesFunction => "requires_plus_ensures",
            Self::ContractNodesFunction | Self::ContractNodesClosure => "expression_nodes",
            Self::ContractExpressionDepth | Self::AggregateDepth => "levels",
            Self::SnapshotEntries | Self::NormalizedIssueEntries => "entries",
            Self::ClosureFunctions => "functions",
            Self::MirBlocksFunction | Self::MirBlocksClosure => "blocks",
            Self::MirStatementsFunction | Self::MirStatementsClosure => "statements",
            Self::ArrayElements => "elements",
            Self::StructFields => "fields",
        }
    }

    pub const fn maximum(self) -> u64 {
        match self {
            Self::ManifestBytes => 1_048_576,
            Self::LockBytes => 4_194_304,
            Self::ContractFiles => 128,
            Self::ContractFileBytes => 1_048_576,
            Self::ContractTotalBytes => 8_388_608,
            Self::ContractClausesFunction => 64,
            Self::ContractNodesFunction => 1_024,
            Self::ContractNodesClosure => 8_192,
            Self::ContractExpressionDepth => 32,
            Self::SourceFiles => 256,
            Self::SourceFileBytes => 1_048_576,
            Self::SourceTotalBytes => 16_777_216,
            Self::SnapshotEntries => 512,
            Self::SnapshotBytes => 33_554_432,
            Self::PathBytes => 1_024,
            Self::ClosureFunctions => 128,
            Self::MirBlocksFunction => 1_024,
            Self::MirBlocksClosure => 8_192,
            Self::MirStatementsFunction => 100_000,
            Self::MirStatementsClosure => 250_000,
            Self::ArrayElements => 256,
            Self::StructFields => 64,
            Self::AggregateDepth => 16,
            Self::NormalizedIssueEntries => 1_024,
            Self::NormalizedIssueMessage => 4_096,
            Self::NormalizedIssueMessageTotal => 2_097_152,
            Self::CargoRustcStdout => 67_108_864,
            Self::CargoRustcStderr => 2_097_152,
            Self::PrivateRequestTransport => 4_194_304,
            Self::PrivateOutputTransport => 268_435_456,
            Self::VirJcs => 201_326_592,
            Self::SourceMapJcs => 33_554_432,
            Self::SourceManifestJcs => 4_194_304,
            Self::PublicFrontendStdout => 268_435_456,
            Self::PublicFrontendStderr => 2_097_152,
        }
    }

    pub const fn above_action(self) -> &'static str {
        match self {
            Self::ManifestBytes
            | Self::LockBytes
            | Self::SourceFileBytes
            | Self::SourceTotalBytes
            | Self::SnapshotBytes => "RUST_LIMIT_INPUT_BYTES",
            Self::ContractFiles
            | Self::ContractFileBytes
            | Self::ContractTotalBytes
            | Self::ContractClausesFunction
            | Self::ContractNodesFunction
            | Self::ContractNodesClosure
            | Self::ContractExpressionDepth => "RUST_LIMIT_CONTRACT",
            Self::SourceFiles | Self::SnapshotEntries => "RUST_LIMIT_INPUT_COUNT",
            Self::PathBytes => "RUST_LIMIT_PATH",
            Self::ClosureFunctions => "RUST_LIMIT_CALL_CLOSURE",
            Self::MirBlocksFunction | Self::MirBlocksClosure => "RUST_LIMIT_MIR_BLOCKS",
            Self::MirStatementsFunction | Self::MirStatementsClosure => "RUST_LIMIT_MIR_STATEMENTS",
            Self::ArrayElements | Self::StructFields | Self::AggregateDepth => {
                "RUST_LIMIT_AGGREGATE"
            }
            Self::NormalizedIssueEntries | Self::NormalizedIssueMessageTotal => {
                "append_RUST_LIMIT_DIAGNOSTICS_TRUNCATED_and_preserve_status"
            }
            Self::NormalizedIssueMessage => "truncate_message_and_preserve_status",
            Self::CargoRustcStdout | Self::CargoRustcStderr => "RUST_FRONTEND_CHILD_OUTPUT_LIMIT",
            Self::PrivateRequestTransport => "RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT",
            Self::PrivateOutputTransport => "RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT",
            Self::VirJcs | Self::SourceMapJcs | Self::SourceManifestJcs => "RUST_LIMIT_IR",
            Self::PublicFrontendStdout | Self::PublicFrontendStderr => "FRONTEND_PROTOCOL_LIMIT",
        }
    }
}

impl TryFrom<&str> for RustLimitId {
    type Error = RustLimitError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .into_iter()
            .find(|limit| limit.as_str() == value)
            .ok_or_else(|| RustLimitError::Unknown(value.to_owned()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustLimitError {
    Unknown(String),
    Exceeded { limit: RustLimitId, observed: u64 },
    CounterOverflow { limit: RustLimitId },
}

impl RustLimitError {
    pub fn action(&self) -> Option<&'static str> {
        match self {
            Self::Unknown(_) => None,
            Self::Exceeded { limit, .. } | Self::CounterOverflow { limit } => {
                Some(limit.above_action())
            }
        }
    }
}

impl fmt::Display for RustLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(id) => write!(formatter, "unknown Rust limit {id:?}"),
            Self::Exceeded { limit, observed } => write!(
                formatter,
                "{} count {observed} exceeds inclusive maximum {}",
                limit.as_str(),
                limit.maximum()
            ),
            Self::CounterOverflow { limit } => {
                write!(formatter, "{} counter overflow", limit.as_str())
            }
        }
    }
}

impl std::error::Error for RustLimitError {}

pub fn validate_rust_limit(id: &str, observed: u64) -> Result<(), RustLimitError> {
    validate_limit(RustLimitId::try_from(id)?, observed)
}

pub fn validate_limit(limit: RustLimitId, observed: u64) -> Result<(), RustLimitError> {
    if observed > limit.maximum() {
        Err(RustLimitError::Exceeded { limit, observed })
    } else {
        Ok(())
    }
}

pub fn checked_add(
    limit: RustLimitId,
    current: u64,
    additional: u64,
) -> Result<u64, RustLimitError> {
    let observed = current
        .checked_add(additional)
        .ok_or(RustLimitError::CounterOverflow { limit })?;
    validate_limit(limit, observed)?;
    Ok(observed)
}
