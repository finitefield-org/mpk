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
