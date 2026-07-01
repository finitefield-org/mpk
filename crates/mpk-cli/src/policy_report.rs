use crate::policy_evidence::{
    PolicyCheckerVerdictEvidence, PolicyCheckerVerdictStatus, PolicyEvidenceReport,
    PolicyHelperArtifactKind, PolicyPropertyEvidence, PolicyPropertyEvidenceRef,
    PolicyPropertyEvidenceStatus,
};

pub fn render_policy_evidence_markdown(report: &PolicyEvidenceReport) -> String {
    let mut output = String::new();
    output.push_str("# MPK Policy Evidence Report\n\n");
    render_target(report, &mut output);
    render_status_summary(report, &mut output);
    render_properties(
        "Verified Properties",
        report,
        PolicyPropertyEvidenceStatus::MpkVerified,
        &mut output,
    );
    render_properties(
        "Proof-Pending Properties",
        report,
        PolicyPropertyEvidenceStatus::ProofPending,
        &mut output,
    );
    render_properties(
        "Helper-Only Properties",
        report,
        PolicyPropertyEvidenceStatus::HelperOnly,
        &mut output,
    );
    render_properties(
        "Unsupported Properties",
        report,
        PolicyPropertyEvidenceStatus::Unsupported,
        &mut output,
    );
    render_required_preconditions(&mut output);
    render_hashes(report, &mut output);
    render_checker_verdicts(report, &mut output);
    render_helper_artifacts(report, &mut output);
    render_reproduction_commands(report, &mut output);
    render_trust_boundary_notes(&mut output);
    output
}

fn render_target(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Target Function\n\n");
    bullet(output, "Package", &report.target.package_path);
    bullet(output, "Function", &report.target.function_id);
    bullet(output, "Strategy profile", &report.strategy_profile);
    bullet(output, "Checker profile", &report.checker_profile);
    bullet(
        output,
        "Allowed axiom profiles",
        &join_or_none(&report.allowed_axiom_profiles),
    );
    output.push('\n');
}

fn render_status_summary(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Verification Summary\n\n");
    bullet(
        output,
        "Verified",
        &property_count(report, PolicyPropertyEvidenceStatus::MpkVerified).to_string(),
    );
    bullet(
        output,
        "Proof pending",
        &property_count(report, PolicyPropertyEvidenceStatus::ProofPending).to_string(),
    );
    bullet(
        output,
        "Helper only",
        &property_count(report, PolicyPropertyEvidenceStatus::HelperOnly).to_string(),
    );
    bullet(
        output,
        "Unsupported",
        &property_count(report, PolicyPropertyEvidenceStatus::Unsupported).to_string(),
    );
    output.push('\n');
}

fn render_properties(
    title: &str,
    report: &PolicyEvidenceReport,
    status: PolicyPropertyEvidenceStatus,
    output: &mut String,
) {
    output.push_str("## ");
    output.push_str(title);
    output.push_str("\n\n");

    let mut matched = false;
    for property in report
        .properties
        .iter()
        .filter(|property| property.status == status)
    {
        matched = true;
        render_property(property, output);
    }

    if !matched {
        output.push_str("- None recorded.\n");
    }
    output.push('\n');
}

fn render_property(property: &PolicyPropertyEvidence, output: &mut String) {
    output.push_str("- `");
    output.push_str(&property.id);
    output.push_str("`: ");
    output.push_str(&property.description);
    output.push_str(" (status: `");
    output.push_str(property_status_label(property.status));
    output.push_str("`)\n");

    if property.evidence.is_empty() {
        output.push_str("  - Evidence: none recorded.\n");
    } else {
        output.push_str("  - Evidence:\n");
        for evidence in &property.evidence {
            output.push_str("    - ");
            output.push_str(&property_evidence_label(evidence));
            output.push('\n');
        }
    }

    if !property.notes.is_empty() {
        output.push_str("  - Notes:\n");
        for note in &property.notes {
            output.push_str("    - ");
            output.push_str(note);
            output.push('\n');
        }
    }
}

fn render_required_preconditions(output: &mut String) {
    output.push_str("## Required Preconditions\n\n");
    output.push_str(
        "- Not recorded in `mpk.policy.evidence.v0`; consult the source evidence JSON and scan JSON for contract preconditions.\n\n",
    );
}

fn render_hashes(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Hashes\n\n");
    output.push_str("### Trusted Evidence Hashes\n\n");

    if report.trusted_evidence.certificates.is_empty()
        && report.trusted_evidence.theory_certificates.is_empty()
        && report.trusted_evidence.axiom_report.is_none()
    {
        output.push_str("- None recorded.\n");
    } else {
        for certificate in &report.trusted_evidence.certificates {
            output.push_str("- Certificate `");
            output.push_str(&certificate.id);
            output.push_str("`\n");
            nested_bullet(output, "Module", &certificate.module);
            nested_bullet(output, "Path", &certificate.path);
            nested_bullet(output, "Certificate hash", &certificate.certificate_hash);
            nested_bullet(output, "Export hash", &certificate.export_hash);
            nested_bullet(output, "Axiom report hash", &certificate.axiom_report_hash);
        }
        for theory_certificate in &report.trusted_evidence.theory_certificates {
            output.push_str("- Theory certificate `");
            output.push_str(&theory_certificate.id);
            output.push_str("`\n");
            nested_bullet(output, "Theory", &theory_certificate.theory);
            nested_bullet(output, "Format", &theory_certificate.format);
            nested_bullet(
                output,
                "Theory certificate hash",
                &theory_certificate.theory_certificate_hash,
            );
            nested_bullet(
                output,
                "Checker profile",
                &theory_certificate.checker_profile,
            );
        }
        if let Some(axiom_report) = &report.trusted_evidence.axiom_report {
            output.push_str("- Axiom report\n");
            nested_bullet(output, "Axiom report hash", &axiom_report.axiom_report_hash);
            nested_bullet(
                output,
                "Total axiom count",
                &axiom_report.category_counts.total_axiom_count.to_string(),
            );
            nested_bullet(
                output,
                "Core axiom count",
                &axiom_report.category_counts.core_axiom_count.to_string(),
            );
            nested_bullet(
                output,
                "Builtin theory axiom count",
                &axiom_report
                    .category_counts
                    .builtin_theory_axiom_count
                    .to_string(),
            );
            nested_bullet(
                output,
                "Go semantics axiom count",
                &axiom_report
                    .category_counts
                    .go_semantics_axiom_count
                    .to_string(),
            );
            nested_bullet(
                output,
                "External axiom count",
                &axiom_report
                    .category_counts
                    .external_axiom_count
                    .to_string(),
            );
        }
    }

    output.push_str("\n### Helper Artifact Hashes\n\n");
    bullet(output, "Source root", &report.helper_artifacts.source.root);
    bullet(
        output,
        "Source hash",
        &report.helper_artifacts.source.source_hash,
    );
    for file in &report.helper_artifacts.source.files {
        output.push_str("- Source file `");
        output.push_str(&file.path);
        output.push_str("`: `");
        output.push_str(&file.sha256);
        output.push_str("`\n");
    }
    bullet(
        output,
        "Contract path",
        &report.helper_artifacts.contract.path,
    );
    bullet(
        output,
        "Contract schema",
        &report.helper_artifacts.contract.schema,
    );
    bullet(
        output,
        "Contract hash",
        &report.helper_artifacts.contract.contract_hash,
    );
    bullet(
        output,
        "GIR hash",
        report
            .helper_artifacts
            .gir_hash
            .as_deref()
            .unwrap_or("none"),
    );
    bullet(
        output,
        "VC hash",
        report.helper_artifacts.vc_hash.as_deref().unwrap_or("none"),
    );
    output.push('\n');
}

fn render_checker_verdicts(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Checker Verdicts\n\n");
    render_checker_verdict(
        "Rust fast-kernel",
        &report.trusted_evidence.rust_checker,
        output,
    );
    render_checker_verdict(
        "Independent reference checker",
        &report.trusted_evidence.reference_checker,
        output,
    );
    output.push('\n');
}

fn render_checker_verdict(
    label: &str,
    evidence: &Option<PolicyCheckerVerdictEvidence>,
    output: &mut String,
) {
    match evidence {
        Some(evidence) => {
            output.push_str("- ");
            output.push_str(label);
            output.push_str(": `");
            output.push_str(checker_verdict_label(evidence.verdict));
            output.push_str("`\n");
            nested_bullet(output, "Command", &evidence.command);
            nested_bullet(
                output,
                "Certificate ids",
                &join_or_none(&evidence.certificate_ids),
            );
        }
        None => {
            output.push_str("- ");
            output.push_str(label);
            output.push_str(": not recorded.\n");
        }
    }
}

fn render_helper_artifacts(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Helper Artifacts\n\n");
    output.push_str("- Go source text is helper evidence only.\n");
    output.push_str("- Contract JSON is helper evidence only.\n");
    output.push_str("- GIR is helper evidence only.\n");
    output.push_str("- VC JSON is helper evidence only.\n");
    output.push_str("- CI status is helper evidence only.\n");

    if report.helper_artifacts.warnings.is_empty() {
        output.push_str("- Helper warnings: none recorded.\n");
    } else {
        output.push_str("- Helper warnings:\n");
        for warning in &report.helper_artifacts.warnings {
            output.push_str("  - `");
            output.push_str(&warning.code);
            output.push_str("` (");
            output.push_str(helper_artifact_label(warning.artifact));
            output.push_str("): ");
            output.push_str(&warning.message);
            output.push('\n');
        }
    }
    if !report.helper_artifacts.call_site_preconditions.is_empty() {
        output.push_str("- Call-site preconditions (helper analysis):\n");
        for precondition in &report.helper_artifacts.call_site_preconditions {
            output.push_str("  - `");
            output.push_str(&precondition.expression);
            output.push_str("`: `");
            output.push_str(precondition.status.as_str());
            output.push_str("` (`");
            output.push_str(call_site_evidence_label(precondition.evidence_label));
            output.push_str("`)\n");
            double_nested_bullet(output, "Id", &precondition.id);
            if let Some(source_path) = &precondition.source_path {
                double_nested_bullet(output, "Source path", source_path);
            }
            if let Some(function_id) = &precondition.function_id {
                double_nested_bullet(output, "Function", function_id);
            }
            double_nested_bullet(output, "Summary", &precondition.summary);
        }
    }
    output.push('\n');
}

fn render_reproduction_commands(report: &PolicyEvidenceReport, output: &mut String) {
    output.push_str("## Reproduction Commands\n\n");
    if report.reproduction_commands.is_empty() {
        output.push_str("- None recorded.\n\n");
        return;
    }

    for command in &report.reproduction_commands {
        output.push_str("- ");
        output.push_str(&command.label);
        output.push_str("\n\n");
        output.push_str("```sh\n");
        output.push_str(&command.command);
        output.push_str("\n```\n\n");
    }
}

fn render_trust_boundary_notes(output: &mut String) {
    output.push_str("## Trust-Boundary Notes\n\n");
    output.push_str("- The `mpk.policy.evidence.v0` JSON is the source of truth; this Markdown is a derived view.\n");
    output.push_str("- A property is `mpk_verified` only when it references checked declaration evidence or checked theory-certificate evidence.\n");
    output.push_str("- GIR, VC JSON, source text, contract JSON, and CI status are helper artifacts and are not proof evidence.\n");
}

fn property_count(report: &PolicyEvidenceReport, status: PolicyPropertyEvidenceStatus) -> usize {
    report
        .properties
        .iter()
        .filter(|property| property.status == status)
        .count()
}

fn property_evidence_label(evidence: &PolicyPropertyEvidenceRef) -> String {
    match evidence {
        PolicyPropertyEvidenceRef::CheckedDeclaration {
            certificate_id,
            declaration_id,
        } => format!("checked declaration `{declaration_id}` in certificate `{certificate_id}`"),
        PolicyPropertyEvidenceRef::CheckedTheoryCertificate {
            theory_certificate_id,
            obligation_id,
        } => format!(
            "checked theory certificate `{theory_certificate_id}` for obligation `{obligation_id}`"
        ),
        PolicyPropertyEvidenceRef::HelperArtifact { artifact, summary } => {
            format!(
                "helper artifact `{}`: {summary}",
                helper_artifact_label(*artifact)
            )
        }
        PolicyPropertyEvidenceRef::UnsupportedFeature { code, message } => {
            format!("unsupported feature `{code}`: {message}")
        }
    }
}

fn property_status_label(status: PolicyPropertyEvidenceStatus) -> &'static str {
    match status {
        PolicyPropertyEvidenceStatus::MpkVerified => "mpk_verified",
        PolicyPropertyEvidenceStatus::ProofPending => "proof_pending",
        PolicyPropertyEvidenceStatus::HelperOnly => "helper_only",
        PolicyPropertyEvidenceStatus::Unsupported => "unsupported",
    }
}

fn checker_verdict_label(status: PolicyCheckerVerdictStatus) -> &'static str {
    match status {
        PolicyCheckerVerdictStatus::Accepted => "accepted",
        PolicyCheckerVerdictStatus::Rejected => "rejected",
    }
}

fn helper_artifact_label(artifact: PolicyHelperArtifactKind) -> &'static str {
    match artifact {
        PolicyHelperArtifactKind::GoSource => "go_source",
        PolicyHelperArtifactKind::Contract => "contract",
        PolicyHelperArtifactKind::Gir => "gir",
        PolicyHelperArtifactKind::Vc => "vc",
        PolicyHelperArtifactKind::AiAnalysis => "ai_analysis",
        PolicyHelperArtifactKind::CiStatus => "ci_status",
    }
}

fn call_site_evidence_label(
    label: crate::policy_evidence::PolicyCallSiteEvidenceLabel,
) -> &'static str {
    match label {
        crate::policy_evidence::PolicyCallSiteEvidenceLabel::HelperAnalysis => "helper_analysis",
    }
}

fn bullet(output: &mut String, label: &str, value: &str) {
    output.push_str("- ");
    output.push_str(label);
    output.push_str(": `");
    output.push_str(value);
    output.push_str("`\n");
}

fn nested_bullet(output: &mut String, label: &str, value: &str) {
    output.push_str("  - ");
    output.push_str(label);
    output.push_str(": `");
    output.push_str(value);
    output.push_str("`\n");
}

fn double_nested_bullet(output: &mut String, label: &str, value: &str) {
    output.push_str("    - ");
    output.push_str(label);
    output.push_str(": `");
    output.push_str(value);
    output.push_str("`\n");
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use crate::policy_evidence::{PolicyEvidenceReport, PolicyPropertyEvidenceStatus};

    use super::render_policy_evidence_markdown;

    #[test]
    fn policy_report_summary_counts_are_deterministic() {
        let report = PolicyEvidenceReport::from_json(
            r#"{
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
    "gir_hash": null,
    "vc_hash": null,
    "warnings": []
  },
  "properties": [
    {
      "id": "pending",
      "description": "Pending property.",
      "status": "proof_pending",
      "evidence": [],
      "notes": []
    },
    {
      "id": "helper",
      "description": "Helper-only property.",
      "status": "helper_only",
      "evidence": [],
      "notes": []
    }
  ],
  "reproduction_commands": []
}
"#,
        )
        .expect("valid report");

        assert_eq!(
            report.properties[0].status,
            PolicyPropertyEvidenceStatus::ProofPending
        );
        let markdown = render_policy_evidence_markdown(&report);
        assert!(markdown.contains("- Proof pending: `1`\n"));
        assert!(markdown.contains("- Helper only: `1`\n"));
        assert!(markdown.contains("- `pending`: Pending property. (status: `proof_pending`)"));
        assert!(markdown.contains("- `helper`: Helper-only property. (status: `helper_only`)"));
    }
}
