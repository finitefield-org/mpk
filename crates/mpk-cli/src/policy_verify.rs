use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mpk_api::{
    theory_strategy_certificate_evidence, ApiService, ConstTermRequest, PolicyObligationDescriptor,
    PolicyObligationPattern, PolicyStrategyMetadata, ProofProfile, SortTermRequest,
    StartSessionRequest, StrategyKind, StrategyProveRequest, TheoryStrategyKind,
};
use mpk_vc::{
    classify_payment_policy_obligations, generate_branch_vcs, import_gir_json,
    PaymentPolicyClassificationOutcome, PaymentPolicyClassifierPropertyStatus,
    PaymentPolicyObligationClassification,
    PaymentPolicyObligationPattern as VcPolicyObligationPattern, UnsupportedPropertyCode,
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
        let has_unsupported_property = classifications.iter().any(|classification| {
            classification.outcome == PaymentPolicyClassificationOutcome::UnsupportedProperty
        });
        let verified_obligation = if has_unsupported_property {
            None
        } else {
            try_close_first_linarith_obligation(request, &classifications, &mut trusted)?
        };

        properties = classifications
            .iter()
            .map(|classification| {
                property_from_classification(classification, verified_obligation.as_deref())
            })
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

fn try_close_first_linarith_obligation(
    request: &PolicyVerifyRequest,
    classifications: &[PaymentPolicyObligationClassification],
    trusted: &mut PolicyTrustedEvidence,
) -> Result<Option<String>, PolicyVerifyRunError> {
    let metadata =
        PolicyStrategyMetadata::parse_profile(&request.strategy_profile).map_err(|error| {
            PolicyVerifyRunError::with_source("policy strategy profile failed", error)
        })?;
    let Some(classification) = classifications.iter().find(|classification| {
        classification.pattern == Some(VcPolicyObligationPattern::NonNegativeResult)
    }) else {
        return Ok(None);
    };
    let pattern = map_vc_pattern(
        classification
            .pattern
            .expect("classification was filtered to a supported pattern"),
    );
    if metadata
        .validate_obligation(&PolicyObligationDescriptor::new(
            classification.obligation_id.clone(),
            pattern,
        ))
        .is_err()
    {
        return Ok(None);
    }

    let Some(theory_candidate) = metadata
        .theory_candidates()
        .into_iter()
        .find(|candidate| candidate.theory == TheoryStrategyKind::Linarith)
    else {
        return Ok(None);
    };

    let proof_profile = parse_proof_profile(&request.checker_profile)?;
    let mut api = ApiService::new();
    let session_id = api
        .start_session(
            StartSessionRequest::new("ProofOps.PolicyVerify").with_proof_profile(proof_profile),
        )
        .map_err(|error| {
            PolicyVerifyRunError::with_source("policy strategy session failed", error)
        })?
        .session_id;
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .map_err(|error| PolicyVerifyRunError::with_source("policy strategy goal failed", error))?
        .term_id;
    register_strategy_witness(&mut api, &session_id, sort)?;
    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: false,
            apply: Vec::new(),
            theory: vec![theory_candidate],
        })
        .map_err(|error| {
            PolicyVerifyRunError::with_source("policy strategy attempt failed", error)
        })?;
    if !response.ok
        || response.attempts.len() != 1
        || response.attempts[0].strategy != StrategyKind::Theory
        || !response.attempts[0].ok
    {
        return Ok(None);
    }
    let session = api
        .session(&session_id)
        .ok_or_else(|| PolicyVerifyRunError::new("policy strategy session disappeared"))?;
    if session.theory_certificate_count() != 1 {
        return Ok(None);
    }

    let evidence = theory_strategy_certificate_evidence(TheoryStrategyKind::Linarith);
    trusted
        .theory_certificates
        .push(PolicyTheoryCertificateEvidence::new(
            "theory:policy-linarith-001",
            "linarith",
            evidence.format,
            evidence.theory_certificate_hash,
            request.checker_profile.clone(),
            vec![classification.obligation_id.clone()],
        ));

    Ok(Some(classification.obligation_id.clone()))
}

fn register_strategy_witness(
    api: &mut ApiService,
    session_id: &mpk_api::SessionId,
    sort: mpk_api::ApiTermId,
) -> Result<(), PolicyVerifyRunError> {
    let sort_core = api
        .session(session_id)
        .and_then(|session| session.core_term_id(sort))
        .ok_or_else(|| PolicyVerifyRunError::new("policy strategy sort term is missing"))?;
    api.session_mut(session_id)
        .ok_or_else(|| PolicyVerifyRunError::new("policy strategy session disappeared"))?
        .environment_mut()
        .register_axiom("ProofOps.PolicyVerify.theoryWitness", sort_core)
        .map_err(|error| {
            PolicyVerifyRunError::new(format!(
                "policy strategy witness failed: {:?}",
                error.code()
            ))
        })?;
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: "ProofOps.PolicyVerify.theoryWitness".to_owned(),
        levels: Vec::new(),
    })
    .map_err(|error| {
        PolicyVerifyRunError::with_source("policy strategy witness term failed", error)
    })?;
    Ok(())
}

fn property_from_classification(
    classification: &PaymentPolicyObligationClassification,
    verified_obligation: Option<&str>,
) -> PolicyPropertyEvidence {
    if verified_obligation == Some(classification.obligation_id.as_str()) {
        let mut property = PolicyPropertyEvidence::new(
            classification.obligation_id.clone(),
            property_description(classification),
            PolicyPropertyEvidenceStatus::MpkVerified,
        );
        property
            .evidence
            .push(PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
                theory_certificate_id: "theory:policy-linarith-001".to_owned(),
                obligation_id: classification.obligation_id.clone(),
            });
        property.notes.push(
            "Closed by a checked linarith theory certificate under the configured checker profile."
                .to_owned(),
        );
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
