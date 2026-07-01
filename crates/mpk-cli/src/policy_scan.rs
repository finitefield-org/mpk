use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

pub const POLICY_SCAN_SCHEMA: &str = "mpk.policy.scan.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanReport {
    pub schema: String,
    pub target: PolicyScanTarget,
    pub source: PolicyScanSource,
    pub contract: PolicyScanContract,
    pub readiness: PolicyScanReadiness,
    pub supported_features: Vec<PolicyScanFeature>,
    pub rejected_features: Vec<PolicyScanFeature>,
    pub preconditions: Vec<PolicyScanPrecondition>,
}

impl PolicyScanReport {
    pub fn new(
        target: PolicyScanTarget,
        source: PolicyScanSource,
        contract: PolicyScanContract,
        readiness: PolicyScanReadiness,
    ) -> Self {
        Self {
            schema: POLICY_SCAN_SCHEMA.to_owned(),
            target,
            source,
            contract,
            readiness,
            supported_features: Vec::new(),
            rejected_features: Vec::new(),
            preconditions: Vec::new(),
        }
    }

    pub fn from_json(text: &str) -> Result<Self, PolicyScanParseError> {
        let report = serde_json::from_str::<Self>(text).map_err(PolicyScanParseError::Json)?;
        if report.schema != POLICY_SCAN_SCHEMA {
            return Err(PolicyScanParseError::SchemaMismatch {
                expected: POLICY_SCAN_SCHEMA,
                actual: report.schema,
            });
        }
        Ok(report)
    }

    pub fn to_deterministic_json(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        Ok(json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanTarget {
    pub package_path: String,
    pub function_id: String,
}

impl PolicyScanTarget {
    pub fn new(package_path: impl Into<String>, function_id: impl Into<String>) -> Self {
        Self {
            package_path: package_path.into(),
            function_id: function_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanSource {
    pub root: String,
    pub go_toolchain: String,
    pub go2gir_sha256: String,
    pub source_sha256: String,
    pub gir_sha256: Option<String>,
    pub files: Vec<PolicyScanSourceFile>,
}

impl PolicyScanSource {
    pub fn new(
        root: impl Into<String>,
        go_toolchain: impl Into<String>,
        go2gir_sha256: impl Into<String>,
        source_sha256: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
            go_toolchain: go_toolchain.into(),
            go2gir_sha256: go2gir_sha256.into(),
            source_sha256: source_sha256.into(),
            gir_sha256: None,
            files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanSourceFile {
    pub path: String,
    pub sha256: String,
}

impl PolicyScanSourceFile {
    pub fn new(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            sha256: sha256.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanContract {
    pub path: Option<String>,
    pub schema: Option<String>,
    pub sha256: Option<String>,
    pub status: PolicyScanContractStatus,
    pub function_id: Option<String>,
}

impl PolicyScanContract {
    pub fn resolved(
        path: impl Into<String>,
        schema: impl Into<String>,
        sha256: impl Into<String>,
        function_id: impl Into<String>,
    ) -> Self {
        Self {
            path: Some(path.into()),
            schema: Some(schema.into()),
            sha256: Some(sha256.into()),
            status: PolicyScanContractStatus::FunctionResolved,
            function_id: Some(function_id.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScanContractStatus {
    NotProvided,
    Parsed,
    FunctionResolved,
    FunctionNotFound,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanReadiness {
    pub status: PolicyScanReadinessStatus,
    pub summary: String,
}

impl PolicyScanReadiness {
    pub fn new(status: PolicyScanReadinessStatus, summary: impl Into<String>) -> Self {
        Self {
            status,
            summary: summary.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScanReadinessStatus {
    Ready,
    NeedsRefactor,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanFeature {
    pub code: String,
    pub message: String,
    pub source_path: Option<String>,
    pub function_id: Option<String>,
    pub evidence_label: PolicyScanEvidenceLabel,
    pub location: Option<PolicyScanLocation>,
}

impl PolicyScanFeature {
    pub fn helper(
        code: impl Into<String>,
        message: impl Into<String>,
        source_path: Option<String>,
        function_id: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source_path,
            function_id,
            evidence_label: PolicyScanEvidenceLabel::HelperEvidence,
            location: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScanEvidenceLabel {
    HelperEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanLocation {
    pub line: u32,
    pub column: u32,
}

impl PolicyScanLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyScanPrecondition {
    pub id: String,
    pub expression: String,
    pub source: PolicyScanPreconditionSource,
    pub source_path: Option<String>,
    pub function_id: Option<String>,
    pub evidence_label: PolicyScanEvidenceLabel,
}

impl PolicyScanPrecondition {
    pub fn helper(
        id: impl Into<String>,
        expression: impl Into<String>,
        source: PolicyScanPreconditionSource,
        source_path: Option<String>,
        function_id: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            expression: expression.into(),
            source,
            source_path,
            function_id,
            evidence_label: PolicyScanEvidenceLabel::HelperEvidence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScanPreconditionSource {
    ContractRequires,
}

#[derive(Debug)]
pub enum PolicyScanParseError {
    Json(serde_json::Error),
    SchemaMismatch {
        expected: &'static str,
        actual: String,
    },
}

impl fmt::Display for PolicyScanParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "{error}"),
            Self::SchemaMismatch { expected, actual } => write!(
                formatter,
                "policy scan schema mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl Error for PolicyScanParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::SchemaMismatch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_scan_schema_constant_is_stable() {
        assert_eq!(POLICY_SCAN_SCHEMA, "mpk.policy.scan.v0");
    }
}
