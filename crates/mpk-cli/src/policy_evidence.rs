use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

pub const POLICY_EVIDENCE_SCHEMA: &str = "mpk.policy.evidence.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyEvidenceReport {
    pub schema: String,
    pub target: PolicyEvidenceTarget,
    pub strategy_profile: String,
    pub checker_profile: String,
    pub allowed_axiom_profiles: Vec<String>,
    pub trusted_evidence: PolicyTrustedEvidence,
    pub helper_artifacts: PolicyHelperArtifacts,
    pub properties: Vec<PolicyPropertyEvidence>,
    pub reproduction_commands: Vec<PolicyEvidenceReproductionCommand>,
}

impl PolicyEvidenceReport {
    pub fn new(
        target: PolicyEvidenceTarget,
        strategy_profile: impl Into<String>,
        checker_profile: impl Into<String>,
        allowed_axiom_profiles: Vec<String>,
        trusted_evidence: PolicyTrustedEvidence,
        helper_artifacts: PolicyHelperArtifacts,
    ) -> Self {
        Self {
            schema: POLICY_EVIDENCE_SCHEMA.to_owned(),
            target,
            strategy_profile: strategy_profile.into(),
            checker_profile: checker_profile.into(),
            allowed_axiom_profiles,
            trusted_evidence,
            helper_artifacts,
            properties: Vec::new(),
            reproduction_commands: Vec::new(),
        }
    }

    pub fn from_json(text: &str) -> Result<Self, PolicyEvidenceParseError> {
        let report = serde_json::from_str::<Self>(text).map_err(PolicyEvidenceParseError::Json)?;
        if report.schema != POLICY_EVIDENCE_SCHEMA {
            return Err(PolicyEvidenceParseError::SchemaMismatch {
                expected: POLICY_EVIDENCE_SCHEMA,
                actual: report.schema,
            });
        }
        report.validate()?;
        Ok(report)
    }

    pub fn to_deterministic_json(&self) -> Result<String, PolicyEvidenceParseError> {
        self.validate()?;
        let mut json =
            serde_json::to_string_pretty(self).map_err(PolicyEvidenceParseError::Json)?;
        json.push('\n');
        Ok(json)
    }

    fn validate(&self) -> Result<(), PolicyEvidenceParseError> {
        if self.allowed_axiom_profiles.is_empty() {
            return Err(PolicyEvidenceParseError::InvalidReport(
                "allowed_axiom_profiles must not be empty".to_owned(),
            ));
        }

        for property in &self.properties {
            let mut has_trusted_reference = false;
            for evidence in &property.evidence {
                match evidence {
                    PolicyPropertyEvidenceRef::CheckedDeclaration {
                        certificate_id,
                        declaration_id,
                    } => {
                        let Some(certificate) = self
                            .trusted_evidence
                            .certificates
                            .iter()
                            .find(|certificate| certificate.id == *certificate_id)
                        else {
                            return Err(PolicyEvidenceParseError::InvalidReport(format!(
                                "property {} references missing certificate {}",
                                property.id, certificate_id
                            )));
                        };
                        if !certificate
                            .checked_declarations
                            .iter()
                            .any(|checked| checked == declaration_id)
                        {
                            return Err(PolicyEvidenceParseError::InvalidReport(format!(
                                "property {} references unchecked declaration {} in certificate {}",
                                property.id, declaration_id, certificate_id
                            )));
                        }
                        has_trusted_reference = true;
                    }
                    PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
                        theory_certificate_id,
                        obligation_id,
                    } => {
                        let Some(theory_certificate) = self
                            .trusted_evidence
                            .theory_certificates
                            .iter()
                            .find(|certificate| certificate.id == *theory_certificate_id)
                        else {
                            return Err(PolicyEvidenceParseError::InvalidReport(format!(
                                "property {} references missing theory certificate {}",
                                property.id, theory_certificate_id
                            )));
                        };
                        if !theory_certificate
                            .checked_obligations
                            .iter()
                            .any(|checked| checked == obligation_id)
                        {
                            return Err(PolicyEvidenceParseError::InvalidReport(format!(
                                "property {} references unchecked obligation {} in theory certificate {}",
                                property.id, obligation_id, theory_certificate_id
                            )));
                        }
                        has_trusted_reference = true;
                    }
                    PolicyPropertyEvidenceRef::HelperArtifact { .. }
                    | PolicyPropertyEvidenceRef::UnsupportedFeature { .. } => {}
                }
            }

            if property.status == PolicyPropertyEvidenceStatus::MpkVerified
                && !has_trusted_reference
            {
                return Err(PolicyEvidenceParseError::InvalidReport(format!(
                    "property {} is mpk_verified without checked declaration or checked theory-certificate evidence",
                    property.id
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyEvidenceTarget {
    pub package_path: String,
    pub function_id: String,
}

impl PolicyEvidenceTarget {
    pub fn new(package_path: impl Into<String>, function_id: impl Into<String>) -> Self {
        Self {
            package_path: package_path.into(),
            function_id: function_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTrustedEvidence {
    pub certificates: Vec<PolicyCertificateEvidence>,
    pub theory_certificates: Vec<PolicyTheoryCertificateEvidence>,
    pub axiom_report: Option<PolicyAxiomReportEvidence>,
    pub rust_checker: Option<PolicyCheckerVerdictEvidence>,
    pub reference_checker: Option<PolicyCheckerVerdictEvidence>,
}

impl PolicyTrustedEvidence {
    pub fn empty() -> Self {
        Self {
            certificates: Vec::new(),
            theory_certificates: Vec::new(),
            axiom_report: None,
            rust_checker: None,
            reference_checker: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCertificateEvidence {
    pub id: String,
    pub module: String,
    pub path: String,
    pub certificate_hash: String,
    pub export_hash: String,
    pub axiom_report_hash: String,
    pub checked_declarations: Vec<String>,
}

impl PolicyCertificateEvidence {
    pub fn new(
        id: impl Into<String>,
        module: impl Into<String>,
        path: impl Into<String>,
        certificate_hash: impl Into<String>,
        export_hash: impl Into<String>,
        axiom_report_hash: impl Into<String>,
        checked_declarations: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            module: module.into(),
            path: path.into(),
            certificate_hash: certificate_hash.into(),
            export_hash: export_hash.into(),
            axiom_report_hash: axiom_report_hash.into(),
            checked_declarations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTheoryCertificateEvidence {
    pub id: String,
    pub theory: String,
    pub theory_certificate_hash: String,
    pub checker_profile: String,
    pub checked_obligations: Vec<String>,
}

impl PolicyTheoryCertificateEvidence {
    pub fn new(
        id: impl Into<String>,
        theory: impl Into<String>,
        theory_certificate_hash: impl Into<String>,
        checker_profile: impl Into<String>,
        checked_obligations: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            theory: theory.into(),
            theory_certificate_hash: theory_certificate_hash.into(),
            checker_profile: checker_profile.into(),
            checked_obligations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAxiomReportEvidence {
    pub axiom_report_hash: String,
    pub category_counts: PolicyAxiomCategoryCounts,
}

impl PolicyAxiomReportEvidence {
    pub fn new(
        axiom_report_hash: impl Into<String>,
        category_counts: PolicyAxiomCategoryCounts,
    ) -> Self {
        Self {
            axiom_report_hash: axiom_report_hash.into(),
            category_counts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAxiomCategoryCounts {
    pub total_axiom_count: u32,
    pub core_axiom_count: u32,
    pub builtin_theory_axiom_count: u32,
    pub go_semantics_axiom_count: u32,
    pub external_axiom_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCheckerVerdictEvidence {
    pub verdict: PolicyCheckerVerdictStatus,
    pub command: String,
    pub certificate_ids: Vec<String>,
}

impl PolicyCheckerVerdictEvidence {
    pub fn accepted(command: impl Into<String>, certificate_ids: Vec<String>) -> Self {
        Self {
            verdict: PolicyCheckerVerdictStatus::Accepted,
            command: command.into(),
            certificate_ids,
        }
    }

    pub fn rejected(command: impl Into<String>, certificate_ids: Vec<String>) -> Self {
        Self {
            verdict: PolicyCheckerVerdictStatus::Rejected,
            command: command.into(),
            certificate_ids,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheckerVerdictStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyHelperArtifacts {
    pub source: PolicySourceArtifact,
    pub contract: PolicyContractArtifact,
    pub gir_hash: Option<String>,
    pub vc_hash: Option<String>,
    pub warnings: Vec<PolicyHelperWarning>,
}

impl PolicyHelperArtifacts {
    pub fn new(source: PolicySourceArtifact, contract: PolicyContractArtifact) -> Self {
        Self {
            source,
            contract,
            gir_hash: None,
            vc_hash: None,
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySourceArtifact {
    pub root: String,
    pub source_hash: String,
    pub files: Vec<PolicySourceFileHash>,
}

impl PolicySourceArtifact {
    pub fn new(
        root: impl Into<String>,
        source_hash: impl Into<String>,
        files: Vec<PolicySourceFileHash>,
    ) -> Self {
        Self {
            root: root.into(),
            source_hash: source_hash.into(),
            files,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySourceFileHash {
    pub path: String,
    pub sha256: String,
}

impl PolicySourceFileHash {
    pub fn new(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            sha256: sha256.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyContractArtifact {
    pub path: String,
    pub schema: String,
    pub contract_hash: String,
}

impl PolicyContractArtifact {
    pub fn new(
        path: impl Into<String>,
        schema: impl Into<String>,
        contract_hash: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            schema: schema.into(),
            contract_hash: contract_hash.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyHelperWarning {
    pub code: String,
    pub message: String,
    pub artifact: PolicyHelperArtifactKind,
}

impl PolicyHelperWarning {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        artifact: PolicyHelperArtifactKind,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            artifact,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyHelperArtifactKind {
    GoSource,
    Contract,
    Gir,
    Vc,
    AiAnalysis,
    CiStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyPropertyEvidence {
    pub id: String,
    pub description: String,
    pub status: PolicyPropertyEvidenceStatus,
    pub evidence: Vec<PolicyPropertyEvidenceRef>,
    pub notes: Vec<String>,
}

impl PolicyPropertyEvidence {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        status: PolicyPropertyEvidenceStatus,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            status,
            evidence: Vec::new(),
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyPropertyEvidenceStatus {
    MpkVerified,
    ProofPending,
    HelperOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolicyPropertyEvidenceRef {
    CheckedDeclaration {
        certificate_id: String,
        declaration_id: String,
    },
    CheckedTheoryCertificate {
        theory_certificate_id: String,
        obligation_id: String,
    },
    HelperArtifact {
        artifact: PolicyHelperArtifactKind,
        summary: String,
    },
    UnsupportedFeature {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyEvidenceReproductionCommand {
    pub label: String,
    pub command: String,
}

impl PolicyEvidenceReproductionCommand {
    pub fn new(label: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            command: command.into(),
        }
    }
}

#[derive(Debug)]
pub enum PolicyEvidenceParseError {
    Json(serde_json::Error),
    SchemaMismatch {
        expected: &'static str,
        actual: String,
    },
    InvalidReport(String),
}

impl fmt::Display for PolicyEvidenceParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid policy evidence JSON: {error}"),
            Self::SchemaMismatch { expected, actual } => {
                write!(
                    formatter,
                    "policy evidence schema = {actual:?}, want {expected:?}"
                )
            }
            Self::InvalidReport(message) => formatter.write_str(message),
        }
    }
}

impl Error for PolicyEvidenceParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::SchemaMismatch { .. } | Self::InvalidReport(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PolicyEvidenceReport, POLICY_EVIDENCE_SCHEMA};

    #[test]
    fn policy_evidence_schema_constant_is_stable() {
        assert_eq!(POLICY_EVIDENCE_SCHEMA, "mpk.policy.evidence.v0");
    }

    #[test]
    fn mpk_verified_requires_trusted_evidence() {
        let json = r#"{
  "schema": "mpk.policy.evidence.v0",
  "target": {
    "package_path": "example.com/orderpolicy",
    "function_id": "example.com/orderpolicy.ApprovedReserveCents"
  },
  "strategy_profile": "payment-policy-alpha",
  "checker_profile": "mvp-strict",
  "allowed_axiom_profiles": [
    "zero-axiom"
  ],
  "trusted_evidence": {
    "certificates": [],
    "theory_certificates": [],
    "axiom_report": null,
    "rust_checker": null,
    "reference_checker": null
  },
  "helper_artifacts": {
    "source": {
      "root": "examples/order_policy",
      "source_hash": "5059e9b3d3e45e2310ec2bdeefcc8fda71c0dd95a506afd10d84bb41ee5ee502",
      "files": []
    },
    "contract": {
      "path": "examples/order_policy/policy_contract.json",
      "schema": "mpk.go.contract.v0",
      "contract_hash": "fcf5c2aec662011ea5b00382710a35d051a4b93ce56b70cadbf22a9573f14a00"
    },
    "gir_hash": "83746ecfbc479a244f421a6df8ffece093c43aaef6bf9c126592ad260343b950",
    "vc_hash": null,
    "warnings": []
  },
  "properties": [
    {
      "id": "approved_reserve_nonnegative",
      "description": "Approved reserve cents never goes negative.",
      "status": "mpk_verified",
      "evidence": [
        {
          "kind": "helper_artifact",
          "artifact": "gir",
          "summary": "GIR lowering succeeded"
        }
      ],
      "notes": []
    }
  ],
  "reproduction_commands": []
}
"#;

        let error = PolicyEvidenceReport::from_json(json).expect_err("report is invalid");
        assert!(error
            .to_string()
            .contains("mpk_verified without checked declaration"));
    }
}
