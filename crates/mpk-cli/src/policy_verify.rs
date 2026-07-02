use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mpk_api::{
    PolicyObligationDescriptor, PolicyObligationPattern, PolicyStrategyMetadata, ProofProfile,
};
use mpk_cert::encode::TheoryCertificate;
use mpk_cert::{encode_theory_certificate, hash_hex, hash_with_domain, HashDomain};
use mpk_kernel::proof_theory::check_theory_certificate;
use mpk_theory::{
    check_linarith_certificate, encode_linarith_certificate, FarkasMultiplier, LinarithCertificate,
    LinearInequality, LinearTerm, LINARITH_CERT_FORMAT,
};
use mpk_vc::{
    classify_payment_policy_obligations, generate_branch_vcs, import_gir_json,
    policy_theory_goal_from_obligation, PaymentPolicyClassificationOutcome,
    PaymentPolicyClassifierPropertyStatus, PaymentPolicyObligationClassification,
    PaymentPolicyObligationPattern as VcPolicyObligationPattern, PolicyLinearGoal,
    PolicyTheoryGoalKind, UnsupportedPropertyCode, VcObligation,
};
use sha2::{Digest, Sha256};

use crate::policy_evidence::{
    PolicyContractArtifact, PolicyEvidenceReport, PolicyEvidenceReproductionCommand,
    PolicyEvidenceTarget, PolicyHelperArtifactKind, PolicyHelperArtifacts, PolicyHelperWarning,
    PolicyPropertyEvidence, PolicyPropertyEvidenceRef, PolicyPropertyEvidenceStatus,
    PolicySourceArtifact, PolicySourceFileHash, PolicyTheoryCertificateEvidence,
    PolicyTrustedEvidence,
};
use crate::policy_report::render_policy_evidence_markdown;
use crate::policy_scan::{
    run_policy_scan_with_artifacts, PolicyScanReadinessStatus, PolicyScanReport, PolicyScanRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyVerifyRequest {
    pub target: String,
    pub function_id: String,
    pub contract_path: String,
    pub strategy_profile: String,
    pub checker_profile: String,
    pub evidence_json_path: PathBuf,
    pub evidence_md_path: PathBuf,
    pub go2gir_path: PathBuf,
    pub strict: bool,
    pub update_fixtures: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyVerifyOutput {
    pub evidence_json_path: PathBuf,
    pub evidence_md_path: PathBuf,
    pub verified_count: usize,
    pub proof_pending_count: usize,
    pub unsupported_count: usize,
}

impl PolicyVerifyOutput {
    pub fn status_label(&self) -> &'static str {
        if self.unsupported_count > 0 {
            "unsupported"
        } else if self.proof_pending_count > 0 {
            "proof_pending"
        } else {
            "verified"
        }
    }
}

#[derive(Debug)]
pub struct PolicyVerifyRunError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl PolicyVerifyRunError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    fn with_source(message: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for PolicyVerifyRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PolicyVerifyRunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyClosedObligation {
    obligation_id: String,
    certificate_id: String,
    theory: String,
    format: String,
    theory_certificate_hash: String,
    evidence_note: String,
}

impl PolicyClosedObligation {
    fn new(
        obligation_id: impl Into<String>,
        certificate_id: impl Into<String>,
        theory: impl Into<String>,
        format: impl Into<String>,
        theory_certificate_hash: impl Into<String>,
        evidence_note: impl Into<String>,
    ) -> Self {
        Self {
            obligation_id: obligation_id.into(),
            certificate_id: certificate_id.into(),
            theory: theory.into(),
            format: format.into(),
            theory_certificate_hash: theory_certificate_hash.into(),
            evidence_note: evidence_note.into(),
        }
    }
}

pub fn run_policy_verify(
    request: &PolicyVerifyRequest,
) -> Result<PolicyVerifyOutput, PolicyVerifyRunError> {
    reject_tracked_output_without_update(&request.evidence_json_path, request.update_fixtures)?;
    reject_tracked_output_without_update(&request.evidence_md_path, request.update_fixtures)?;

    let scan_output = run_policy_scan_with_artifacts(&PolicyScanRequest {
        target: request.target.clone(),
        function_id: request.function_id.clone(),
        contract_path: request.contract_path.clone(),
        go2gir_path: request.go2gir_path.clone(),
    })
    .map_err(|error| {
        PolicyVerifyRunError::with_source(format!("policy verify scan failed: {error}"), error)
    })?;

    let mut trusted = PolicyTrustedEvidence::empty();
    let mut helper_artifacts = helper_artifacts_from_scan(&scan_output.report, None);
    let mut properties = Vec::new();

    if scan_output.report.readiness.status == PolicyScanReadinessStatus::Ready {
        let gir_json = scan_output
            .gir_json
            .as_deref()
            .ok_or_else(|| PolicyVerifyRunError::new("policy verify scan did not produce GIR"))?;
        let gir = import_gir_json(gir_json).map_err(|error| {
            PolicyVerifyRunError::with_source("policy verify GIR import failed", error)
        })?;
        let vc_module = generate_branch_vcs(&gir).map_err(|error| {
            PolicyVerifyRunError::with_source("policy verify VC generation failed", error)
        })?;
        let vc_hash = vc_module_hash(&vc_module)?;
        helper_artifacts = helper_artifacts_from_scan(&scan_output.report, Some(vc_hash));

        let classifications = classify_payment_policy_obligations(&vc_module)
            .obligations
            .into_iter()
            .filter(|classification| classification.function_id == request.function_id)
            .collect::<Vec<_>>();
        let closed_obligations =
            try_close_policy_obligations(request, &vc_module.obligations, &classifications)?;
        add_closed_obligations_to_trusted(
            &mut trusted,
            &closed_obligations,
            &request.checker_profile,
        );

        properties = classifications
            .iter()
            .map(|classification| property_from_classification(classification, &closed_obligations))
            .collect();
    } else {
        helper_artifacts.warnings.push(PolicyHelperWarning::new(
            "POLICY_SCAN_NOT_READY",
            scan_output.report.readiness.summary.clone(),
            scan_warning_artifact(&scan_output.report),
        ));
    }

    let mut evidence = PolicyEvidenceReport::new(
        PolicyEvidenceTarget::new(
            scan_output.report.target.package_path.clone(),
            request.function_id.clone(),
        ),
        request.strategy_profile.clone(),
        request.checker_profile.clone(),
        vec!["zero-axiom".to_owned()],
        trusted,
        helper_artifacts,
    );
    evidence.properties = properties;
    evidence.reproduction_commands = reproduction_commands(request);

    let evidence_json = evidence
        .to_deterministic_json()
        .map_err(|error| PolicyVerifyRunError::with_source("policy evidence JSON failed", error))?;
    let evidence_md = render_policy_evidence_markdown(&evidence);
    write_text(&request.evidence_json_path, &evidence_json)?;
    write_text(&request.evidence_md_path, &evidence_md)?;

    let output = PolicyVerifyOutput {
        evidence_json_path: request.evidence_json_path.clone(),
        evidence_md_path: request.evidence_md_path.clone(),
        verified_count: property_count(&evidence, PolicyPropertyEvidenceStatus::MpkVerified),
        proof_pending_count: property_count(&evidence, PolicyPropertyEvidenceStatus::ProofPending),
        unsupported_count: property_count(&evidence, PolicyPropertyEvidenceStatus::Unsupported),
    };

    if scan_output.report.readiness.status != PolicyScanReadinessStatus::Ready {
        return Err(PolicyVerifyRunError::new(format!(
            "policy verify failed: scan status={}",
            scan_output.report.readiness.status.as_str()
        )));
    }
    if output.unsupported_count > 0 {
        return Err(PolicyVerifyRunError::new(format!(
            "policy verify failed: unsupported properties={}",
            output.unsupported_count
        )));
    }
    if request.strict && output.proof_pending_count > 0 {
        return Err(PolicyVerifyRunError::new(format!(
            "policy verify failed: proof-pending properties={}",
            output.proof_pending_count
        )));
    }

    Ok(output)
}

fn try_close_policy_obligations(
    request: &PolicyVerifyRequest,
    vc_obligations: &[VcObligation],
    classifications: &[PaymentPolicyObligationClassification],
) -> Result<BTreeMap<String, PolicyClosedObligation>, PolicyVerifyRunError> {
    let proof_profile = parse_proof_profile(&request.checker_profile)?;
    if proof_profile != ProofProfile::MvpStrict {
        return Ok(BTreeMap::new());
    }

    let obligations_by_id = validate_policy_closure_inputs(vc_obligations, classifications)?;
    let metadata =
        PolicyStrategyMetadata::parse_profile(&request.strategy_profile).map_err(|error| {
            PolicyVerifyRunError::with_source("policy strategy profile failed", error)
        })?;
    let supported_classifications = supported_classifications_by_id(classifications, &metadata)?;
    let mut closed_obligations = BTreeMap::new();
    let mut linarith_certificate_index = 1usize;
    for (obligation_id, classification) in supported_classifications {
        let obligation = obligations_by_id.get(obligation_id).ok_or_else(|| {
            policy_closure_failed(format!(
                "classification references missing VC obligation {obligation_id:?}"
            ))
        })?;
        let Some(theory_goal) = policy_theory_goal_from_obligation(obligation, classification)
            .map_err(|error| {
                policy_closure_failed(format!(
                    "extract theory goal for obligation {obligation_id:?}: {error}"
                ))
            })?
        else {
            continue;
        };
        let PolicyTheoryGoalKind::Linear(linear_goal) = theory_goal.kind else {
            continue;
        };
        let Some((certificate, reason)) = linarith_certificate_from_goal(
            &linear_goal,
            classification
                .pattern
                .expect("supported classification has a validated pattern"),
        ) else {
            continue;
        };
        let closed = checked_linarith_closure(
            obligation_id,
            certificate,
            reason,
            linarith_certificate_index,
        )?;
        linarith_certificate_index += 1;
        closed_obligations.insert(closed.obligation_id.clone(), closed);
    }
    Ok(closed_obligations)
}

fn validate_policy_closure_inputs<'a>(
    vc_obligations: &'a [VcObligation],
    classifications: &[PaymentPolicyObligationClassification],
) -> Result<BTreeMap<&'a str, &'a VcObligation>, PolicyVerifyRunError> {
    let mut obligations_by_id = BTreeMap::new();
    for obligation in vc_obligations {
        if obligations_by_id
            .insert(obligation.id.as_str(), obligation)
            .is_some()
        {
            return Err(policy_closure_failed(format!(
                "duplicate VC obligation id {:?}",
                obligation.id
            )));
        }
    }

    let mut classifications_by_id = BTreeMap::new();
    for classification in classifications {
        if !obligations_by_id.contains_key(classification.obligation_id.as_str()) {
            return Err(policy_closure_failed(format!(
                "classification references missing VC obligation {:?}",
                classification.obligation_id
            )));
        }
        if classifications_by_id
            .insert(classification.obligation_id.as_str(), ())
            .is_some()
        {
            return Err(policy_closure_failed(format!(
                "duplicate classification for VC obligation {:?}",
                classification.obligation_id
            )));
        }
    }

    Ok(obligations_by_id)
}

fn supported_classifications_by_id<'a>(
    classifications: &'a [PaymentPolicyObligationClassification],
    metadata: &PolicyStrategyMetadata,
) -> Result<BTreeMap<&'a str, &'a PaymentPolicyObligationClassification>, PolicyVerifyRunError> {
    let mut supported = BTreeMap::new();
    for classification in classifications {
        if classification.outcome == PaymentPolicyClassificationOutcome::UnsupportedProperty {
            continue;
        }
        let pattern = classification.pattern.ok_or_else(|| {
            policy_closure_failed(format!(
                "classification for obligation {:?} is supported but has no pattern",
                classification.obligation_id
            ))
        })?;
        metadata
            .validate_obligation(&PolicyObligationDescriptor::new(
                classification.obligation_id.clone(),
                map_vc_pattern(pattern),
            ))
            .map_err(|error| policy_closure_failed(error.to_string()))?;
        supported.insert(classification.obligation_id.as_str(), classification);
    }
    Ok(supported)
}

fn add_closed_obligations_to_trusted(
    trusted: &mut PolicyTrustedEvidence,
    closed_obligations: &BTreeMap<String, PolicyClosedObligation>,
    checker_profile: &str,
) {
    for closed in closed_obligations.values() {
        trusted
            .theory_certificates
            .push(PolicyTheoryCertificateEvidence::new(
                closed.certificate_id.clone(),
                closed.theory.clone(),
                closed.format.clone(),
                closed.theory_certificate_hash.clone(),
                checker_profile.to_owned(),
                vec![closed.obligation_id.clone()],
            ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinarithClosureReason {
    ExactPremise,
    IdentityGoal,
    BranchPremise,
}

impl LinarithClosureReason {
    fn evidence_note(self) -> &'static str {
        match self {
            Self::ExactPremise => {
                "Closed by checked linarith evidence for an exact linear premise."
            }
            Self::IdentityGoal => {
                "Closed by checked linarith evidence for an identity linear goal."
            }
            Self::BranchPremise => {
                "Closed by checked linarith evidence for a branch-premise linear bound."
            }
        }
    }
}

fn linarith_certificate_from_goal(
    goal: &PolicyLinearGoal,
    pattern: VcPolicyObligationPattern,
) -> Option<(LinarithCertificate, LinarithClosureReason)> {
    let premises = goal
        .premises
        .iter()
        .map(policy_linear_inequality_to_linarith)
        .collect::<Vec<_>>();
    let linarith_goal = policy_linear_inequality_to_linarith(&goal.goal);

    if goal.goal.terms.is_empty() && goal.goal.constant <= 0 {
        return Some((
            LinarithCertificate {
                premises,
                goal: linarith_goal,
                combination: Vec::new(),
            },
            LinarithClosureReason::IdentityGoal,
        ));
    }

    let (premise_index, premise) = goal.premises.iter().enumerate().find(|(_, premise)| {
        premise.terms == goal.goal.terms && premise.constant >= goal.goal.constant
    })?;
    let reason = if pattern == VcPolicyObligationPattern::NonNegativeResult
        && premise.constant == goal.goal.constant
    {
        LinarithClosureReason::ExactPremise
    } else {
        LinarithClosureReason::BranchPremise
    };
    Some((
        LinarithCertificate {
            premises,
            goal: linarith_goal,
            combination: vec![FarkasMultiplier::new(premise_index, 1)],
        },
        reason,
    ))
}

fn policy_linear_inequality_to_linarith(
    inequality: &mpk_vc::PolicyLinearInequality,
) -> LinearInequality {
    LinearInequality::new(
        inequality
            .terms
            .iter()
            .map(|term| LinearTerm::new(term.variable, term.coefficient))
            .collect(),
        inequality.constant,
    )
}

fn checked_linarith_closure(
    obligation_id: &str,
    certificate: LinarithCertificate,
    reason: LinarithClosureReason,
    certificate_index: usize,
) -> Result<PolicyClosedObligation, PolicyVerifyRunError> {
    check_linarith_certificate(&certificate).map_err(|error| {
        policy_closure_failed(format!(
            "linarith certificate rejected for obligation {obligation_id:?}: {error}"
        ))
    })?;
    let theory_certificate = TheoryCertificate {
        format: LINARITH_CERT_FORMAT.to_owned(),
        payload: encode_linarith_certificate(&certificate),
    };
    check_theory_certificate(&theory_certificate).map_err(|error| {
        policy_closure_failed(format!(
            "encoded linarith theory certificate rejected for obligation {obligation_id:?}: {error}"
        ))
    })?;
    let canonical = encode_theory_certificate(&theory_certificate);
    let theory_certificate_hash =
        hash_hex(&hash_with_domain(HashDomain::TheoryCertificate, &canonical));

    Ok(PolicyClosedObligation::new(
        obligation_id.to_owned(),
        format!("theory:policy-linarith-{certificate_index:04}"),
        "linarith",
        LINARITH_CERT_FORMAT,
        theory_certificate_hash,
        reason.evidence_note(),
    ))
}

fn property_from_classification(
    classification: &PaymentPolicyObligationClassification,
    closed_obligations: &BTreeMap<String, PolicyClosedObligation>,
) -> PolicyPropertyEvidence {
    if let Some(closed) = closed_obligations.get(&classification.obligation_id) {
        let mut property = PolicyPropertyEvidence::new(
            classification.obligation_id.clone(),
            property_description(classification),
            PolicyPropertyEvidenceStatus::MpkVerified,
        );
        property
            .evidence
            .push(PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
                theory_certificate_id: closed.certificate_id.clone(),
                obligation_id: classification.obligation_id.clone(),
            });
        property.notes.push(closed.evidence_note.clone());
        return property;
    }

    match classification.property_status {
        PaymentPolicyClassifierPropertyStatus::ProofPending => {
            let mut property = PolicyPropertyEvidence::new(
                classification.obligation_id.clone(),
                property_description(classification),
                PolicyPropertyEvidenceStatus::ProofPending,
            );
            property
                .evidence
                .push(PolicyPropertyEvidenceRef::HelperArtifact {
                    artifact: PolicyHelperArtifactKind::Vc,
                    summary: "VC was generated and classified, but no checked proof certificate was accepted for this obligation.".to_owned(),
                });
            if let Some(pattern) = classification.pattern {
                property.notes.push(format!(
                    "Classified helper pattern: {}.",
                    vc_pattern_name(pattern)
                ));
            }
            property
        }
        PaymentPolicyClassifierPropertyStatus::Unsupported => {
            let diagnostic = classification
                .diagnostic
                .as_ref()
                .expect("unsupported classification carries diagnostic");
            let mut property = PolicyPropertyEvidence::new(
                classification.obligation_id.clone(),
                property_description(classification),
                PolicyPropertyEvidenceStatus::Unsupported,
            );
            property
                .evidence
                .push(PolicyPropertyEvidenceRef::UnsupportedFeature {
                    code: unsupported_code(diagnostic.code).to_owned(),
                    message: diagnostic.message.clone(),
                });
            property
        }
    }
}

fn policy_closure_failed(message: impl Into<String>) -> PolicyVerifyRunError {
    PolicyVerifyRunError::new(format!(
        "policy verify proof closure failed: {}",
        message.into()
    ))
}

fn helper_artifacts_from_scan(
    scan: &PolicyScanReport,
    vc_hash: Option<String>,
) -> PolicyHelperArtifacts {
    let mut artifacts = PolicyHelperArtifacts::new(
        PolicySourceArtifact::new(
            scan.source.root.clone(),
            scan.source.source_sha256.clone(),
            scan.source
                .files
                .iter()
                .map(|file| PolicySourceFileHash::new(file.path.clone(), file.sha256.clone()))
                .collect(),
        ),
        PolicyContractArtifact::new(
            scan.contract.path.clone().unwrap_or_default(),
            scan.contract
                .schema
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            scan.contract
                .sha256
                .clone()
                .unwrap_or_else(|| "unavailable".to_owned()),
        ),
    );
    artifacts.gir_hash = scan.source.gir_sha256.clone();
    artifacts.vc_hash = vc_hash;
    for feature in &scan.rejected_features {
        artifacts.warnings.push(PolicyHelperWarning::new(
            feature.code.clone(),
            feature.message.clone(),
            PolicyHelperArtifactKind::GoSource,
        ));
    }
    artifacts
}

fn scan_warning_artifact(scan: &PolicyScanReport) -> PolicyHelperArtifactKind {
    if scan.contract.status != crate::policy_scan::PolicyScanContractStatus::FunctionResolved {
        PolicyHelperArtifactKind::Contract
    } else {
        PolicyHelperArtifactKind::Gir
    }
}

fn reproduction_commands(request: &PolicyVerifyRequest) -> Vec<PolicyEvidenceReproductionCommand> {
    let scan = format!(
        "mpk policy scan {} --function {} --contract {} --json-out <scan.json> --go2gir <go2gir>",
        request.target, request.function_id, request.contract_path,
    );
    let mut verify = format!(
        "mpk policy verify {} --function {} --contract {} --strategy-profile {} --checker-profile {} --evidence-json <evidence.json> --evidence-md <evidence.md> --go2gir <go2gir>",
        request.target,
        request.function_id,
        request.contract_path,
        request.strategy_profile,
        request.checker_profile,
    );
    if request.strict {
        verify.push_str(" --strict");
    }
    if request.update_fixtures {
        verify.push_str(" --update-fixtures");
    }

    vec![
        PolicyEvidenceReproductionCommand::new("scan", scan),
        PolicyEvidenceReproductionCommand::new("verify", verify),
    ]
}

fn map_vc_pattern(pattern: VcPolicyObligationPattern) -> PolicyObligationPattern {
    match pattern {
        VcPolicyObligationPattern::NonNegativeResult => PolicyObligationPattern::NonNegativeResult,
        VcPolicyObligationPattern::ResultBoundedByInput => {
            PolicyObligationPattern::ResultBoundedByInputAmount
        }
        VcPolicyObligationPattern::RefundBoundedByAvailablePaidAmount => {
            PolicyObligationPattern::RefundBoundedByPaidMinusAlreadyRefunded
        }
        VcPolicyObligationPattern::FeeOrDiscountBoundedByCap => {
            PolicyObligationPattern::DiscountOrFeeBoundedByConfiguredCaps
        }
        VcPolicyObligationPattern::SelectedBranchResultEqualsInput => {
            PolicyObligationPattern::BranchResultEqualsSelectedInput
        }
        VcPolicyObligationPattern::IntegerRuntimeSafety => {
            PolicyObligationPattern::IntegerRuntimeSafety
        }
    }
}

fn parse_proof_profile(profile: &str) -> Result<ProofProfile, PolicyVerifyRunError> {
    match profile {
        "core-bootstrap" => Ok(ProofProfile::CoreBootstrap),
        "mvp-structural" => Ok(ProofProfile::MvpStructural),
        "mvp-strict" => Ok(ProofProfile::MvpStrict),
        _ => Err(PolicyVerifyRunError::new(format!(
            "unknown checker profile {profile:?}"
        ))),
    }
}

fn property_description(classification: &PaymentPolicyObligationClassification) -> String {
    match classification.pattern {
        Some(pattern) => format!(
            "Payment policy obligation classified as {}.",
            vc_pattern_name(pattern)
        ),
        None => {
            "Payment policy obligation is outside the supported proof strategy subset.".to_owned()
        }
    }
}

fn vc_pattern_name(pattern: VcPolicyObligationPattern) -> &'static str {
    match pattern {
        VcPolicyObligationPattern::NonNegativeResult => "non_negative_result",
        VcPolicyObligationPattern::ResultBoundedByInput => "result_bounded_by_input",
        VcPolicyObligationPattern::RefundBoundedByAvailablePaidAmount => {
            "refund_bounded_by_available_paid_amount"
        }
        VcPolicyObligationPattern::FeeOrDiscountBoundedByCap => "fee_or_discount_bounded_by_cap",
        VcPolicyObligationPattern::SelectedBranchResultEqualsInput => {
            "selected_branch_result_equals_input"
        }
        VcPolicyObligationPattern::IntegerRuntimeSafety => "integer_runtime_safety",
    }
}

fn unsupported_code(code: UnsupportedPropertyCode) -> &'static str {
    match code {
        UnsupportedPropertyCode::UnsupportedBooleanStructure => "UNSUPPORTED_BOOLEAN_STRUCTURE",
        UnsupportedPropertyCode::UnsupportedArithmetic => "UNSUPPORTED_ARITHMETIC",
        UnsupportedPropertyCode::UnsupportedType => "UNSUPPORTED_TYPE",
        UnsupportedPropertyCode::UnsupportedObligationKind => "UNSUPPORTED_OBLIGATION_KIND",
        UnsupportedPropertyCode::UnsupportedPropertyShape => "UNSUPPORTED_PROPERTY_SHAPE",
    }
}

fn property_count(evidence: &PolicyEvidenceReport, status: PolicyPropertyEvidenceStatus) -> usize {
    evidence
        .properties
        .iter()
        .filter(|property| property.status == status)
        .count()
}

fn vc_module_hash(module: &mpk_vc::VcModule) -> Result<String, PolicyVerifyRunError> {
    let mut json = serde_json::to_vec_pretty(module)
        .map_err(|error| PolicyVerifyRunError::with_source("encode VC artifact", error))?;
    json.push(b'\n');
    Ok(sha256_hex(&json))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn write_text(path: &Path, text: &str) -> Result<(), PolicyVerifyRunError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            PolicyVerifyRunError::with_source(
                format!("create output directory {}", parent.display()),
                error,
            )
        })?;
    }
    fs::write(path, text).map_err(|error| {
        PolicyVerifyRunError::with_source(format!("write {}", path.display()), error)
    })
}

fn reject_tracked_output_without_update(
    path: &Path,
    update_fixtures: bool,
) -> Result<(), PolicyVerifyRunError> {
    if update_fixtures || !is_git_tracked(path) {
        return Ok(());
    }
    Err(PolicyVerifyRunError::new(format!(
        "policy verify refuses to overwrite tracked fixture {} without --update-fixtures",
        path.display()
    )))
}

fn is_git_tracked(path: &Path) -> bool {
    let Ok(current_dir) = std::env::current_dir() else {
        return false;
    };
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let Ok(relative) = absolute.strip_prefix(&current_dir) else {
        return false;
    };
    Command::new("git")
        .args(["ls-files", "--error-unmatch"])
        .arg(relative)
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mpk_vc::{
        MpkExprTerm, PaymentPolicyEvidenceLabel, PaymentPolicyObligationPattern, VcObligationKind,
    };

    #[test]
    fn policy_closure_input_validation_rejects_duplicate_obligation_ids() {
        let obligations = vec![test_obligation("dup"), test_obligation("dup")];
        let error = validate_policy_closure_inputs(&obligations, &[])
            .expect_err("duplicate obligation ids reject");

        assert_eq!(
            error.to_string(),
            "policy verify proof closure failed: duplicate VC obligation id \"dup\""
        );
    }

    #[test]
    fn policy_closure_input_validation_rejects_missing_classification_refs() {
        let obligations = vec![test_obligation("present")];
        let classifications = vec![supported_classification("missing")];
        let error = validate_policy_closure_inputs(&obligations, &classifications)
            .expect_err("missing classification obligation rejects");

        assert_eq!(
            error.to_string(),
            "policy verify proof closure failed: classification references missing VC obligation \"missing\""
        );
    }

    #[test]
    fn policy_linarith_closure_leaves_unproved_tampered_goal_pending() {
        let obligations = vec![linear_obligation(
            "tampered",
            vec![sge(var("balance"), int64("0"))],
            sge(var("requested"), int64("0")),
        )];
        let classifications = vec![supported_classification("tampered")];

        let closed =
            try_close_policy_obligations(&strict_request(), &obligations, &classifications)
                .expect("closure planner runs");

        assert!(closed.is_empty());
    }

    #[test]
    fn policy_linarith_closure_hash_changes_with_linear_payload() {
        let first_obligations = vec![linear_obligation(
            "first",
            vec![sge(result(0), int64("0"))],
            sge(result(0), int64("0")),
        )];
        let second_obligations = vec![linear_obligation(
            "second",
            vec![sge(result(0), int64("1"))],
            sge(result(0), int64("0")),
        )];

        let first = try_close_policy_obligations(
            &strict_request(),
            &first_obligations,
            &[supported_classification("first")],
        )
        .expect("first closure runs");
        let second = try_close_policy_obligations(
            &strict_request(),
            &second_obligations,
            &[supported_classification("second")],
        )
        .expect("second closure runs");

        assert_ne!(
            first
                .get("first")
                .expect("first obligation closes")
                .theory_certificate_hash,
            second
                .get("second")
                .expect("second obligation closes")
                .theory_certificate_hash
        );
    }

    #[test]
    fn policy_linarith_closure_surfaces_internal_certificate_failures() {
        let certificate = LinarithCertificate {
            premises: vec![LinearInequality::new(vec![LinearTerm::new(0, -1)], 0)],
            goal: LinearInequality::new(vec![LinearTerm::new(0, 1)], 0),
            combination: vec![FarkasMultiplier::new(0, 1)],
        };

        let error =
            checked_linarith_closure("bad", certificate, LinarithClosureReason::ExactPremise, 1)
                .expect_err("invalid certificate rejects");

        assert!(error
            .to_string()
            .starts_with("policy verify proof closure failed: linarith certificate rejected"));
    }

    fn test_obligation(id: &str) -> VcObligation {
        VcObligation {
            id: id.to_owned(),
            function_id: "example.Policy".to_owned(),
            kind: VcObligationKind::Postcondition,
            assumptions: Vec::new(),
            conclusion: MpkExprTerm::bool_literal(true),
        }
    }

    fn linear_obligation(
        id: &str,
        assumptions: Vec<MpkExprTerm>,
        conclusion: MpkExprTerm,
    ) -> VcObligation {
        VcObligation {
            assumptions,
            conclusion,
            ..test_obligation(id)
        }
    }

    fn supported_classification(id: &str) -> PaymentPolicyObligationClassification {
        PaymentPolicyObligationClassification {
            obligation_id: id.to_owned(),
            function_id: "example.Policy".to_owned(),
            outcome: PaymentPolicyClassificationOutcome::SupportedProperty,
            pattern: Some(PaymentPolicyObligationPattern::NonNegativeResult),
            evidence_label: PaymentPolicyEvidenceLabel::HelperAnalysis,
            property_status: PaymentPolicyClassifierPropertyStatus::ProofPending,
            diagnostic: None,
        }
    }

    fn strict_request() -> PolicyVerifyRequest {
        PolicyVerifyRequest {
            target: "unused".to_owned(),
            function_id: "example.Policy".to_owned(),
            contract_path: "unused".to_owned(),
            strategy_profile: "payment-policy-alpha".to_owned(),
            checker_profile: "mvp-strict".to_owned(),
            evidence_json_path: PathBuf::from("unused.json"),
            evidence_md_path: PathBuf::from("unused.md"),
            go2gir_path: PathBuf::from("unused-go2gir"),
            strict: false,
            update_fixtures: false,
        }
    }

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn result(index: u32) -> MpkExprTerm {
        MpkExprTerm::Result { index }
    }

    fn int64(value: &str) -> MpkExprTerm {
        MpkExprTerm::BitVecLiteral {
            value: value.to_owned(),
            width: 64,
            signed: true,
        }
    }

    fn sge(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        MpkExprTerm::apply("Std.BitVec.BV64.sge", vec![lhs, rhs])
    }
}
