use crate::policy_evidence::{
    PolicyCheckerVerdictEvidence, PolicyCheckerVerdictStatus, PolicyEvidenceReport,
    PolicyHelperArtifactKind, PolicyPropertyEvidence, PolicyPropertyEvidenceRef,
    PolicyPropertyEvidenceStatus,
};
use crate::policy_schema::{
    render_posix_argument, validate_policy_limit, PolicyAxiomReportV1, PolicyEvidenceReferenceV1,
    PolicyEvidenceV1, PolicyHelperArtifact, PolicyValidationError, ValidatedPolicyEvidenceV1,
};

pub fn render_policy_evidence_v1_markdown(
    evidence: &ValidatedPolicyEvidenceV1,
) -> Result<String, PolicyValidationError> {
    let report = evidence.document();
    let mut output = BoundedMarkdown::default();
    output.push("# MPK Policy Evidence Report\n\n")?;
    render_v1_target_profiles(report, &mut output)?;
    render_v1_source_release(report, &mut output)?;
    render_v1_summary(report, &mut output)?;
    render_v1_properties(report, &mut output)?;
    render_v1_trusted(report, &mut output)?;
    render_v1_helpers(report, &mut output)?;
    render_v1_recipes(report, &mut output)?;
    render_v1_trust_boundary(&mut output)?;
    Ok(output.finish())
}

fn render_v1_target_profiles(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Target and Profiles\n\n")?;
    v1_code_bullet(output, "Source language", &report.source_language)?;
    v1_code_bullet(output, "Semantic profile", &report.semantic_profile)?;
    v1_code_bullet(output, "Target", report.semantic_parameters.target_id())?;
    match &report.semantic_parameters {
        crate::policy_schema::PolicySemanticParameters::Go(parameters) => {
            v1_code_bullet(
                output,
                "Pointer width",
                &parameters.pointer_width.to_string(),
            )?;
        }
        crate::policy_schema::PolicySemanticParameters::Rust(parameters) => {
            v1_code_bullet(
                output,
                "Pointer width",
                &parameters.pointer_width.to_string(),
            )?;
            v1_code_bullet(output, "Overflow mode", &parameters.overflow_mode)?;
            v1_code_bullet(output, "Panic mode", &parameters.panic_mode)?;
        }
    }
    v1_code_bullet(output, "Package", report.selection.package())?;
    if let crate::policy_schema::PolicySelection::Rust(selection) = &report.selection {
        v1_code_bullet(output, "Crate", &selection.crate_name)?;
        v1_code_bullet(output, "Unit kind", &selection.kind)?;
    }
    v1_code_bullet(output, "Function", report.selection.function())?;
    v1_code_bullet(output, "VIR limit profile", &report.limit_profile)?;
    v1_code_bullet(
        output,
        "Verification limit profile",
        &report.verification_limit_profile,
    )?;
    v1_code_bullet(output, "Strategy profile", &report.strategy_profile)?;
    v1_code_bullet(output, "Checker profile", &report.checker_profile)?;
    v1_code_bullet(output, "Axiom profile", &report.axiom_profile)?;
    v1_code_bullet(
        output,
        "Strict",
        if report.verification_options.strict {
            "true"
        } else {
            "false"
        },
    )?;
    v1_code_bullet(
        output,
        "Update fixtures",
        if report.verification_options.update_fixtures {
            "true"
        } else {
            "false"
        },
    )?;
    output.push("\n")
}

fn render_v1_source_release(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Source and Release Identities\n\n")?;
    v1_code_bullet(output, "Registry schema", &report.release_registry.schema)?;
    v1_code_bullet(output, "Registry ID", &report.release_registry.id)?;
    v1_code_bullet(
        output,
        "Registry SHA-256",
        &report.release_registry.registry_sha256,
    )?;
    v1_code_bullet(output, "Frontend bundle", &report.frontend.bundle_id)?;
    v1_code_bullet(output, "Frontend name", &report.frontend.name)?;
    v1_code_bullet(output, "Frontend version", &report.frontend.version)?;
    v1_code_bullet(
        output,
        "Frontend binary SHA-256",
        &report.frontend.binary_sha256,
    )?;
    for subordinate in &report.frontend.subordinate_binaries {
        output.push("- Subordinate frontend `")?;
        output.push(&subordinate.name)?;
        output.push("`\n")?;
        v1_nested_code_bullet(output, "Version", &subordinate.version)?;
        v1_nested_code_bullet(output, "Binary SHA-256", &subordinate.binary_sha256)?;
    }
    v1_code_bullet(output, "Toolchain bundle", &report.toolchain.bundle_id)?;
    v1_code_bullet(
        output,
        "Toolchain distribution SHA-256",
        &report.toolchain.distribution_sha256,
    )?;
    for component in &report.toolchain.components {
        match component {
            mpk_vc::ComponentIdentity::Executable {
                name,
                release,
                commit_hash,
                binary_sha256,
            } => {
                output.push("- Toolchain executable `")?;
                output.push(name)?;
                output.push("`\n")?;
                v1_nested_code_bullet(output, "Release", release)?;
                if let Some(commit_hash) = commit_hash {
                    v1_nested_code_bullet(output, "Commit hash", commit_hash)?;
                }
                v1_nested_code_bullet(output, "Binary SHA-256", binary_sha256)?;
            }
            mpk_vc::ComponentIdentity::Content {
                name,
                release,
                content_sha256,
            } => {
                output.push("- Toolchain content `")?;
                output.push(name)?;
                output.push("`\n")?;
                v1_nested_code_bullet(output, "Release", release)?;
                v1_nested_code_bullet(output, "Content SHA-256", content_sha256)?;
            }
        }
    }
    for (label, value) in [
        (
            "Frontend source manifest SHA-256",
            report.frontend_source_manifest_hash.as_str(),
        ),
        (
            "Certificate source manifest SHA-256",
            report.certificate_source_manifest_hash.as_str(),
        ),
        ("Input set SHA-256", report.input_set_hash.as_str()),
        ("Source map SHA-256", report.source_map_hash.as_str()),
        ("Source IR schema", report.source_ir_schema.as_str()),
        ("Source IR SHA-256", report.source_ir_hash.as_str()),
        ("Source VC schema", report.source_vc_schema.as_str()),
        ("VC SHA-256", report.vc_hash.as_str()),
    ] {
        v1_code_bullet(output, label, value)?;
    }
    output.push("\n")
}

fn render_v1_summary(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Verification Summary\n\n")?;
    for status in [
        "mpk_verified",
        "proof_pending",
        "helper_only",
        "unsupported",
    ] {
        let count = report
            .properties
            .iter()
            .filter(|property| property.status == status)
            .count();
        v1_code_bullet(output, status, &count.to_string())?;
    }
    output.push("\n")
}

fn render_v1_properties(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Properties\n\n")?;
    for property in &report.properties {
        output.push("- Property `")?;
        output.push(&property.id)?;
        output.push("`: ")?;
        output.push(&escape_markdown_prose(&property.description))?;
        output.push("\n")?;
        v1_nested_code_bullet(output, "Status", &property.status)?;
        for member in &property.members {
            output.push("  - Member `")?;
            output.push(&member.member_id)?;
            output.push("`\n")?;
            v1_double_nested_code_bullet(output, "Function", &member.function_id)?;
            v1_double_nested_code_bullet(output, "Kind", &member.kind)?;
            v1_double_nested_code_bullet(output, "Group", &member.group_id)?;
            v1_double_nested_code_bullet(output, "Declaration", &member.declaration_name)?;
            v1_double_nested_code_bullet(output, "Declaration SHA-256", &member.declaration_hash)?;
            v1_double_nested_code_bullet(output, "Status", &member.status)?;
            let kinds = member
                .evidence
                .iter()
                .map(PolicyEvidenceReferenceV1::kind)
                .collect::<Vec<_>>()
                .join(",");
            v1_double_nested_code_bullet(output, "Evidence kinds", &kinds)?;
        }
        for note in &property.notes {
            output.push("  - Note: ")?;
            output.push(&escape_markdown_prose(note))?;
            output.push("\n")?;
        }
    }
    output.push("\n")
}

fn render_v1_trusted(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Trusted Evidence\n\n")?;
    if report.trusted_evidence.certificates.is_empty() {
        output.push("- Candidate certificate: `not_generated`\n")?;
    }
    for certificate in &report.trusted_evidence.certificates {
        output.push("- Certificate `")?;
        output.push(&certificate.id)?;
        output.push("`\n")?;
        v1_nested_code_bullet(output, "Module", &certificate.module)?;
        v1_nested_code_bullet(output, "Certificate SHA-256", &certificate.certificate_hash)?;
        v1_nested_code_bullet(output, "Export SHA-256", &certificate.export_hash)?;
        v1_nested_code_bullet(
            output,
            "Axiom report SHA-256",
            &certificate.axiom_report_hash,
        )?;
        for declaration in &certificate.checked_declarations {
            output.push("  - Checked declaration `")?;
            output.push(&declaration.name)?;
            output.push("`\n")?;
            v1_double_nested_code_bullet(
                output,
                "Declaration SHA-256",
                &declaration.declaration_hash,
            )?;
            v1_double_nested_code_bullet(output, "Function", &declaration.function_id)?;
            v1_double_nested_code_bullet(output, "Group", &declaration.group_id)?;
            v1_double_nested_code_bullet(output, "Group kind", &declaration.group_kind)?;
            for dependency in &declaration.dependencies {
                output.push("    - Dependency `")?;
                output.push(&dependency.name)?;
                output.push("`: `")?;
                output.push(&dependency.declaration_hash)?;
                output.push("`\n")?;
            }
        }
    }
    for theory in &report.trusted_evidence.theory_certificates {
        output.push("- Theory certificate `")?;
        output.push(&theory.id)?;
        output.push("`\n")?;
        v1_nested_code_bullet(output, "Theory", &theory.theory)?;
        v1_nested_code_bullet(output, "Format", &theory.format)?;
        v1_nested_code_bullet(
            output,
            "Theory certificate SHA-256",
            &theory.theory_certificate_hash,
        )?;
        v1_nested_code_bullet(output, "Checker profile", &theory.checker_profile)?;
        v1_nested_code_bullet(
            output,
            "Checked members",
            &theory.checked_member_ids.join(","),
        )?;
    }
    match &report.trusted_evidence.axiom_report {
        PolicyAxiomReportV1::NotGenerated => {
            output.push("- Axiom report: `not_generated`\n")?;
        }
        PolicyAxiomReportV1::Checked {
            axiom_report_hash,
            category_counts,
        } => {
            output.push("- Axiom report: `checked`\n")?;
            v1_nested_code_bullet(output, "SHA-256", axiom_report_hash)?;
            v1_nested_code_bullet(
                output,
                "Total axiom count",
                &category_counts.total_axiom_count.to_string(),
            )?;
            v1_nested_code_bullet(
                output,
                "Core axiom count",
                &category_counts.core_axiom_count.to_string(),
            )?;
            v1_nested_code_bullet(
                output,
                "Builtin theory axiom count",
                &category_counts.builtin_theory_axiom_count.to_string(),
            )?;
            v1_nested_code_bullet(
                output,
                "Go semantics axiom count",
                &category_counts.go_semantics_axiom_count.to_string(),
            )?;
            v1_nested_code_bullet(
                output,
                "External axiom count",
                &category_counts.external_axiom_count.to_string(),
            )?;
        }
    }
    for verdict in &report.trusted_evidence.checker_verdicts {
        output.push("- Checker `")?;
        output.push(&verdict.checker)?;
        output.push("`\n")?;
        v1_nested_code_bullet(output, "Profile", &verdict.checker_profile)?;
        v1_nested_code_bullet(output, "Verdict", &verdict.verdict)?;
        let certificate_ids = if verdict.certificate_ids.is_empty() {
            "none".to_owned()
        } else {
            verdict.certificate_ids.join(",")
        };
        v1_nested_code_bullet(output, "Certificate IDs", &certificate_ids)?;
    }
    output.push("\n")
}

fn render_v1_helpers(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Helper Artifacts\n\n")?;
    for helper in &report.helper_artifacts {
        output.push("- Untrusted helper `")?;
        output.push(helper.id())?;
        output.push("`\n")?;
        match helper {
            PolicyHelperArtifact::Source {
                normalized_path,
                sha256,
                ..
            } => {
                v1_nested_code_bullet(output, "Kind", "source")?;
                v1_nested_code_bullet(output, "Normalized path", normalized_path)?;
                v1_nested_code_bullet(output, "SHA-256", sha256)?;
            }
            PolicyHelperArtifact::Contract {
                normalized_path,
                schema,
                raw_input_sha256,
                function_id,
                contract_hash,
                ..
            } => {
                v1_nested_code_bullet(output, "Kind", "contract")?;
                v1_nested_code_bullet(output, "Normalized path", normalized_path)?;
                v1_nested_code_bullet(output, "Schema", schema)?;
                v1_nested_code_bullet(output, "Raw input SHA-256", raw_input_sha256)?;
                v1_nested_code_bullet(output, "Function", function_id)?;
                v1_nested_code_bullet(output, "Normalized contract SHA-256", contract_hash)?;
            }
            PolicyHelperArtifact::VerificationIr { schema, sha256, .. } => {
                v1_nested_code_bullet(output, "Kind", "verification_ir")?;
                v1_nested_code_bullet(output, "Schema", schema)?;
                v1_nested_code_bullet(output, "SHA-256", sha256)?;
            }
            PolicyHelperArtifact::Vc { schema, sha256, .. } => {
                v1_nested_code_bullet(output, "Kind", "vc")?;
                v1_nested_code_bullet(output, "Schema", schema)?;
                v1_nested_code_bullet(output, "SHA-256", sha256)?;
            }
            PolicyHelperArtifact::AiAnalysis { schema, sha256, .. } => {
                v1_nested_code_bullet(output, "Kind", "ai_analysis")?;
                v1_nested_code_bullet(output, "Schema", schema)?;
                v1_nested_code_bullet(output, "SHA-256", sha256)?;
            }
            PolicyHelperArtifact::CiStatus {
                system,
                check,
                status,
                subject_sha256,
                ..
            } => {
                v1_nested_code_bullet(output, "Kind", "ci_status")?;
                v1_nested_code_bullet(output, "System", system)?;
                v1_nested_code_bullet(output, "Check", check)?;
                v1_nested_code_bullet(output, "Status", status)?;
                v1_nested_code_bullet(output, "Subject SHA-256", subject_sha256)?;
            }
        }
    }
    output.push("\n")
}

fn render_v1_recipes(
    report: &PolicyEvidenceV1,
    output: &mut BoundedMarkdown,
) -> Result<(), PolicyValidationError> {
    output.push("## Reproduction Recipes\n\n")?;
    for recipe in &report.reproduction_recipes {
        output.push("- Recipe `")?;
        output.push(&recipe.label)?;
        output.push("`; working directory role: `")?;
        output.push(&recipe.working_directory_role)?;
        output.push("` (the source root)\n\n```sh\n")?;
        for (index, argument) in recipe.argv.iter().enumerate() {
            if index > 0 {
                output.push(" ")?;
            }
            output.push(&render_posix_argument(argument))?;
        }
        output.push("\n```\n\n")?;
    }
    Ok(())
}

fn render_v1_trust_boundary(output: &mut BoundedMarkdown) -> Result<(), PolicyValidationError> {
    output.push("## Trust-Boundary Notes\n\n")?;
    output.push("- Only checker-accepted canonical certificate and theory-certificate bytes are trusted evidence.\n")?;
    output.push("- Policy JSON, source text, contracts, VIR, VC, AI analysis, CI status, and this Markdown report are not proof evidence.\n")
}

fn escape_markdown_prose(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '!'
                | '<'
                | '>'
                | '&'
                | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn v1_code_bullet(
    output: &mut BoundedMarkdown,
    label: &str,
    value: &str,
) -> Result<(), PolicyValidationError> {
    output.push("- ")?;
    output.push(label)?;
    output.push(": ")?;
    push_code_span(output, value)?;
    output.push("\n")
}

fn v1_nested_code_bullet(
    output: &mut BoundedMarkdown,
    label: &str,
    value: &str,
) -> Result<(), PolicyValidationError> {
    output.push("  - ")?;
    output.push(label)?;
    output.push(": ")?;
    push_code_span(output, value)?;
    output.push("\n")
}

fn v1_double_nested_code_bullet(
    output: &mut BoundedMarkdown,
    label: &str,
    value: &str,
) -> Result<(), PolicyValidationError> {
    output.push("    - ")?;
    output.push(label)?;
    output.push(": ")?;
    push_code_span(output, value)?;
    output.push("\n")
}

fn push_code_span(output: &mut BoundedMarkdown, value: &str) -> Result<(), PolicyValidationError> {
    let longest_run = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let delimiter = "`".repeat(longest_run.saturating_add(1));
    output.push(&delimiter)?;
    if value.starts_with('`') || value.ends_with('`') {
        output.push(" ")?;
    }
    output.push(value)?;
    if value.starts_with('`') || value.ends_with('`') {
        output.push(" ")?;
    }
    output.push(&delimiter)
}

#[derive(Default)]
struct BoundedMarkdown {
    value: String,
}

impl BoundedMarkdown {
    fn push(&mut self, text: &str) -> Result<(), PolicyValidationError> {
        let next = self.value.len().saturating_add(text.len());
        validate_policy_limit("markdown_bytes", u64::try_from(next).unwrap_or(u64::MAX))?;
        self.value.push_str(text);
        Ok(())
    }

    fn finish(self) -> String {
        self.value
    }
}

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
