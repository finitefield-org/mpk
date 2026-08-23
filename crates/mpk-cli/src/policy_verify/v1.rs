//! Policy verification orchestration over the VIR/VC/evidence v1 boundary.

use crate::frontend_runner::{
    prepare_installed_frontend, run_prepared_frontend, AcceptedFrontendRun, FrontendRunRequest,
};
use crate::policy_profile::{
    summary_only_axiom_report_is_permitted, validate_package_release_profiles,
    validate_policy_profile_selection, PolicyProfileErrorKind, PolicyProfileSelection,
};
use crate::policy_report::render_policy_evidence_v1_markdown;
use crate::policy_scan::v1::{
    build_policy_scan_v1_output, capture_invocation_staging,
    parse_policy_scan_v1_argv_through_scalars, validate_owned_captured_inputs,
    validate_policy_scan_v1_profile, OwnedCapturedInput, OwnedFrontendStaging, PolicyScanV1Error,
    PolicyScanV1Invocation, PolicyScanV1RunOutput,
};
use crate::policy_schema::{
    canonical_policy_evidence_v1_json, expected_reproduction_recipes,
    import_policy_evidence_v1_json, PolicyAxiomCategoryCountsV1, PolicyAxiomReportV1,
    PolicyCertificateEvidenceV1, PolicyCheckedDeclaration, PolicyCheckerVerdictV1,
    PolicyDeclarationDependency, PolicyEvidenceLinkageContext, PolicyEvidenceReferenceV1,
    PolicyEvidenceV1, PolicyExpectedCertificateV1, PolicyExpectedMemberV1,
    PolicyExpectedPropertyV1, PolicyHelperArtifact, PolicyMemberRowV1, PolicyPropertyV1,
    PolicyTrustedEvidenceV1, PolicyValidationError, PolicyVerificationOptions,
    ValidatedPolicyEvidenceV1, POLICY_EVIDENCE_V1_SCHEMA,
};
use crate::program_certificate::{
    assemble_program_certificate_alpha, PlannedProgramDeclaration, ProgramCertificateError,
    ProgramCertificateErrorKind, ProgramCertificateOutcome, ProgramCheckerVerdict,
    ReferenceCheckerReport, UnacceptedProgramCertificate, PROGRAM_CERTIFICATE_MODULE,
};
use mpk_cert::encode::{Certificate, SourceManifest as CertificateSourceManifest, ZERO_HASH};
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate, export_block_hash, hash_hex, hash_with_domain, HashDomain,
};
use mpk_kernel::VerificationReport;
#[cfg(test)]
use mpk_vc::ReleaseSelectionRequest;
use mpk_vc::{
    attach_vc_hash, canonical_json_bytes, emit_validated_vc_skeleton_v1, generate_program_vcs,
    generate_vc_v1, parse_strict_json, CapturedInput, GroupedTheoremDeclaration,
    SourceManifestValidationContext, StrictJsonLimits, ValidatedSourceManifest,
    ValidatedVcCertificateSkeleton, ValidatedVcDocument, VcDocument, VC_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub const USAGE: &str = "mpk policy verify <source-root> --language <go|rust> --semantic-profile <profile> --require-release-registry-id <id> --require-release-registry-sha256 <sha256> --frontend-bundle <id> --toolchain-bundle <id> --target <target> --package <package> --function <function-id> --contract <contract.json> [--contract <contract.json> ...] --strategy-profile <profile> --checker-profile <profile> --axiom-profile <profile> --evidence-json <evidence.json> --evidence-md <evidence.md> [--strict] [--update-fixtures]";

const INTERNAL_SCAN_OUTPUT: &str = "mpk-internal-policy-scan.json";
const VALUE_OPTIONS: [&str; 15] = [
    "--language",
    "--semantic-profile",
    "--require-release-registry-id",
    "--require-release-registry-sha256",
    "--frontend-bundle",
    "--toolchain-bundle",
    "--target",
    "--package",
    "--function",
    "--contract",
    "--strategy-profile",
    "--checker-profile",
    "--axiom-profile",
    "--evidence-json",
    "--evidence-md",
];
const VERIFY_ONLY_OPTIONS: [&str; 5] = [
    "--strategy-profile",
    "--checker-profile",
    "--axiom-profile",
    "--evidence-json",
    "--evidence-md",
];
const FLAGS: [&str; 2] = ["--strict", "--update-fixtures"];
const FORBIDDEN_LOCATORS: [&str; 9] = [
    "--frontend",
    "--frontend-helper",
    "--driver",
    "--removed-frontend",
    "--toolchain-root",
    "--toolchain-path",
    "--registry",
    "--registry-path",
    "--release-registry-path",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PolicyVerifyV1Invocation {
    pub(crate) scan: PolicyScanV1Invocation,
    pub(crate) strategy_profile: String,
    pub(crate) checker_profile: String,
    pub(crate) axiom_profile: String,
    pub(crate) evidence_json: String,
    pub(crate) evidence_md: String,
    pub(crate) strict: bool,
    pub(crate) update_fixtures: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct PolicyVerifyV1RunOutput {
    pub(crate) invocation: PolicyVerifyV1Invocation,
    pub(crate) scan: PolicyScanV1RunOutput,
    pub(crate) vc: ValidatedVcDocument,
    pub(crate) skeleton: ValidatedVcCertificateSkeleton,
    pub(crate) certificate_manifest: ValidatedSourceManifest,
    pub(crate) certificate_source_manifest: CertificateSourceManifest,
    pub(crate) program_certificate: Option<ProgramCertificateOutcome>,
    pub(crate) evidence: ValidatedPolicyEvidenceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyVerifyV1Error {
    code: &'static str,
    detail: String,
}

impl PolicyVerifyV1Error {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for PolicyVerifyV1Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl Error for PolicyVerifyV1Error {}

pub(crate) fn parse_policy_verify_v1_argv(
    argv: &[String],
) -> Result<Option<PolicyVerifyV1Invocation>, PolicyVerifyV1Error> {
    if argv.first().map(String::as_str) != Some("mpk")
        || argv.get(1).map(String::as_str) != Some("policy")
        || argv.get(2).map(String::as_str) != Some("verify")
    {
        return Err(cli_error("expected the exact mpk policy verify route"));
    }
    let source_root = argv
        .get(3)
        .ok_or_else(|| cli_error("source-root positional is missing"))?;
    if argv.len() == 4 && matches!(source_root.as_str(), "help" | "-h" | "--help") {
        return Ok(None);
    }
    if source_root.starts_with("--") {
        recognize_tokens(&argv[3..])?;
    } else {
        recognize_tokens(&argv[4..])?;
    }
    if source_root.is_empty()
        || source_root.starts_with("--")
        || matches!(source_root.as_str(), "help" | "-h" | "--help")
    {
        return Err(cli_error("source-root must be one non-option positional"));
    }

    let mut scan_argv = vec![
        "mpk".to_owned(),
        "policy".to_owned(),
        "scan".to_owned(),
        source_root.clone(),
    ];
    let mut verify_values = BTreeMap::<&str, String>::new();
    let mut flags = BTreeSet::<&str>::new();
    let mut position = 4;
    while position < argv.len() {
        let option = argv[position].as_str();
        if FLAGS.contains(&option) {
            if !flags.insert(option) {
                return Err(cli_error("duplicate verify flag"));
            }
            position += 1;
            continue;
        }
        if !VALUE_OPTIONS.contains(&option) {
            return Err(cli_error("unexpected extra positional argument"));
        }
        let value = argv.get(position + 1).ok_or_else(|| {
            PolicyVerifyV1Error::new("POLICY_CLI_ARGUMENT", "option requires a separate value")
        })?;
        if value.is_empty() || value.starts_with('-') {
            return Err(cli_error("option requires a nonempty separate value"));
        }
        if VERIFY_ONLY_OPTIONS.contains(&option) {
            if verify_values.insert(option, value.clone()).is_some() {
                return Err(cli_error("duplicate singleton verify option"));
            }
        } else {
            scan_argv.extend([option.to_owned(), value.clone()]);
        }
        position += 2;
    }
    if VERIFY_ONLY_OPTIONS
        .iter()
        .any(|option| !verify_values.contains_key(option))
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CLI_REQUIRED",
            "a mandatory verify option is missing",
        ));
    }
    scan_argv.extend(["--json-out".to_owned(), INTERNAL_SCAN_OUTPUT.to_owned()]);
    let scan = parse_policy_scan_v1_argv_through_scalars(&scan_argv)
        .map_err(scan_error)?
        .ok_or_else(|| cli_error("verify arguments unexpectedly selected help"))?;
    let strategy_profile = take(&mut verify_values, "--strategy-profile");
    let checker_profile = take(&mut verify_values, "--checker-profile");
    let axiom_profile = take(&mut verify_values, "--axiom-profile");
    let evidence_json = take(&mut verify_values, "--evidence-json");
    let evidence_md = take(&mut verify_values, "--evidence-md");
    for path in [&evidence_json, &evidence_md] {
        mpk_vc::validate_manifest_normalized_path(path).map_err(|_| {
            PolicyVerifyV1Error::new(
                "POLICY_CLI_SCALAR",
                "evidence output must be a normalized relative path",
            )
        })?;
    }
    validate_policy_scan_v1_profile(&scan).map_err(scan_error)?;
    validate_verify_profiles(&scan, &strategy_profile, &checker_profile, &axiom_profile)?;
    Ok(Some(PolicyVerifyV1Invocation {
        scan,
        strategy_profile,
        checker_profile,
        axiom_profile,
        evidence_json,
        evidence_md,
        strict: flags.contains("--strict"),
        update_fixtures: flags.contains("--update-fixtures"),
    }))
}

fn recognize_tokens(arguments: &[String]) -> Result<(), PolicyVerifyV1Error> {
    for token in arguments.iter().filter(|token| token.starts_with('-')) {
        if FORBIDDEN_LOCATORS.contains(&token.as_str()) {
            return Err(PolicyVerifyV1Error::new(
                "POLICY_CLI_FORBIDDEN_LOCATOR",
                "raw frontend, helper, toolchain, or registry locators are forbidden",
            ));
        }
        if !VALUE_OPTIONS.contains(&token.as_str()) && !FLAGS.contains(&token.as_str()) {
            return Err(PolicyVerifyV1Error::new(
                "POLICY_CLI_UNKNOWN_OPTION",
                "option is not accepted by policy verify v1",
            ));
        }
    }
    Ok(())
}

fn validate_verify_profiles(
    scan: &PolicyScanV1Invocation,
    strategy: &str,
    checker: &str,
    axiom: &str,
) -> Result<(), PolicyVerifyV1Error> {
    validate_policy_profile_selection(PolicyProfileSelection {
        strategy_profile: strategy,
        checker_profile: checker,
        source_language: &scan.source_language,
        semantic_profile: &scan.semantic_profile,
        axiom_profile: axiom,
    })
    .map(|_| ())
    .map_err(|error| {
        let code = match error.kind() {
            PolicyProfileErrorKind::CrossedTuple => "POLICY_PROFILE_TUPLE",
            PolicyProfileErrorKind::Unknown | PolicyProfileErrorKind::PackageMismatch => {
                "POLICY_PROFILE_UNKNOWN"
            }
        };
        PolicyVerifyV1Error::new(code, error.to_string())
    })
}

fn take(values: &mut BTreeMap<&str, String>, name: &str) -> String {
    values
        .remove(name)
        .expect("mandatory verify option presence was checked")
}

fn cli_error(detail: impl Into<String>) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new("POLICY_CLI_ARGUMENT", detail)
}

fn scan_error(error: PolicyScanV1Error) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new(error.code(), error.to_string())
}

/// Runs the released policy-verification command over one immutable source
/// snapshot and the registry-selected frontend/toolchain pair.
pub fn run_cli(
    argv: &[String],
    working_directory: &Path,
) -> Result<Option<String>, PolicyVerifyV1Error> {
    let Some(invocation) = parse_policy_verify_v1_argv(argv)? else {
        return Ok(None);
    };
    let prepared =
        prepare_installed_frontend(&invocation.scan.release_request()).map_err(|error| {
            PolicyVerifyV1Error::new(
                error.code().as_str(),
                "generic frontend release preflight failed",
            )
        })?;
    let mut tracked = |relative: &Path| git_tracked(working_directory, relative);
    let outputs = preflight_outputs(
        working_directory,
        &invocation.evidence_json,
        &invocation.evidence_md,
        invocation.update_fixtures,
        &mut tracked,
    )?;
    let staging =
        capture_invocation_staging(&invocation.scan, working_directory).map_err(scan_error)?;
    let output = run_prepared_policy_verify_v1(
        invocation,
        outputs,
        staging,
        prepared,
        |prepared, request| {
            run_prepared_frontend(prepared, request).map_err(|error| {
                PolicyVerifyV1Error::new(error.code().as_str(), "generic frontend runner failed")
            })
        },
        assemble_program_certificate_alpha,
    )?;
    Ok(Some(format!(
        "ok policy verify status=complete evidence_json={} evidence_md={}",
        output.invocation.evidence_json, output.invocation.evidence_md
    )))
}

#[cfg(test)]
pub(crate) fn run_policy_verify_v1_with<P, F, R, T>(
    argv: &[String],
    working_directory: &Path,
    captured_inputs: Vec<OwnedCapturedInput>,
    prepare: F,
    runner: R,
    tracked: T,
) -> Result<Option<PolicyVerifyV1RunOutput>, PolicyVerifyV1Error>
where
    F: FnMut(&ReleaseSelectionRequest) -> Result<P, PolicyVerifyV1Error>,
    R: for<'a> FnMut(P, FrontendRunRequest<'a>) -> Result<AcceptedFrontendRun, PolicyVerifyV1Error>,
    T: FnMut(&Path) -> bool,
{
    run_policy_verify_v1_with_assembler(
        argv,
        working_directory,
        captured_inputs,
        prepare,
        runner,
        tracked,
        assemble_program_certificate_alpha,
    )
}

#[cfg(test)]
pub(crate) fn run_policy_verify_v1_with_assembler<P, F, R, T, A>(
    argv: &[String],
    working_directory: &Path,
    captured_inputs: Vec<OwnedCapturedInput>,
    mut prepare: F,
    runner: R,
    mut tracked: T,
    assembler: A,
) -> Result<Option<PolicyVerifyV1RunOutput>, PolicyVerifyV1Error>
where
    F: FnMut(&ReleaseSelectionRequest) -> Result<P, PolicyVerifyV1Error>,
    R: for<'a> FnMut(P, FrontendRunRequest<'a>) -> Result<AcceptedFrontendRun, PolicyVerifyV1Error>,
    T: FnMut(&Path) -> bool,
    A: FnMut(
        &VcDocument,
        &[GroupedTheoremDeclaration],
        CertificateSourceManifest,
    ) -> Result<ProgramCertificateOutcome, ProgramCertificateError>,
{
    let Some(invocation) = parse_policy_verify_v1_argv(argv)? else {
        return Ok(None);
    };
    let prepared = prepare(&invocation.scan.release_request())?;
    let outputs = preflight_outputs(
        working_directory,
        &invocation.evidence_json,
        &invocation.evidence_md,
        invocation.update_fixtures,
        &mut tracked,
    )?;
    let staging = OwnedFrontendStaging {
        captured_inputs,
        staged_directories: Vec::new(),
        staged_placeholders: Vec::new(),
    };
    run_prepared_policy_verify_v1(invocation, outputs, staging, prepared, runner, assembler)
        .map(Some)
}

fn run_prepared_policy_verify_v1<P, R, A>(
    invocation: PolicyVerifyV1Invocation,
    outputs: OutputPreflight,
    staging: OwnedFrontendStaging,
    prepared: P,
    mut runner: R,
    mut assembler: A,
) -> Result<PolicyVerifyV1RunOutput, PolicyVerifyV1Error>
where
    R: for<'a> FnMut(P, FrontendRunRequest<'a>) -> Result<AcceptedFrontendRun, PolicyVerifyV1Error>,
    A: FnMut(
        &VcDocument,
        &[GroupedTheoremDeclaration],
        CertificateSourceManifest,
    ) -> Result<ProgramCertificateOutcome, ProgramCertificateError>,
{
    validate_owned_captured_inputs(&staging.captured_inputs).map_err(scan_error)?;
    let OwnedFrontendStaging {
        captured_inputs,
        staged_directories,
        staged_placeholders,
    } = staging;
    let policy_parameters = invocation.scan.semantic_parameters();
    let policy_selection = invocation.scan.selection();
    let semantic_parameters = serde_json::to_value(&policy_parameters)
        .map_err(|error| linkage_error(error.to_string()))?;
    let selection = serde_json::to_value(&policy_selection)
        .map_err(|error| linkage_error(error.to_string()))?;
    let captured_refs = captured_inputs
        .iter()
        .map(OwnedCapturedInput::as_ref)
        .collect::<Vec<_>>();
    let staged_directory_refs = staged_directories
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let staged_placeholder_refs = staged_placeholders
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let frontend = runner(
        prepared,
        FrontendRunRequest {
            release: invocation.scan.release_request(),
            semantic_parameters: &semantic_parameters,
            selection: &selection,
            captured_inputs: &captured_refs,
            staged_directories: &staged_directory_refs,
            staged_placeholders: &staged_placeholder_refs,
            contracts: &invocation.scan.contracts,
        },
    )?;
    let scan = build_policy_scan_v1_output(invocation.scan.clone(), frontend, captured_inputs)
        .map_err(scan_error)?;
    let finalized = finalize_evidence(&invocation, scan, &mut assembler)?;
    let markdown =
        render_policy_evidence_v1_markdown(&finalized.evidence).map_err(policy_validation_error)?;
    commit_outputs(
        &outputs,
        finalized.evidence.canonical_bytes(),
        markdown.as_bytes(),
    )?;
    if let Some(ProgramCertificateOutcome::Unaccepted(candidate)) =
        finalized.program_certificate.as_ref()
    {
        let code = match candidate.failure_kind {
            ProgramCertificateErrorKind::CheckerRejected => "POLICY_CHECKER_REJECTED",
            ProgramCertificateErrorKind::CheckerDisagreement => "POLICY_CHECKER_DISAGREEMENT",
            ProgramCertificateErrorKind::Limit(limit) => limit.code(),
            ProgramCertificateErrorKind::Foundation
            | ProgramCertificateErrorKind::Skeleton
            | ProgramCertificateErrorKind::Interface
            | ProgramCertificateErrorKind::CheckerExecution
            | ProgramCertificateErrorKind::Internal => "POLICY_CERTIFICATE_ASSEMBLY",
        };
        return Err(PolicyVerifyV1Error::new(
            code,
            candidate.failure_detail.clone(),
        ));
    }
    if invocation.strict
        && finalized
            .evidence
            .document()
            .properties
            .iter()
            .flat_map(|property| &property.members)
            .any(|member| member.status == "proof_pending")
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_PROOF_PENDING",
            "strict verification retained one or more proof-pending members",
        ));
    }
    Ok(PolicyVerifyV1RunOutput {
        invocation,
        scan: finalized.scan,
        vc: finalized.vc,
        skeleton: finalized.skeleton,
        certificate_manifest: finalized.certificate_manifest,
        certificate_source_manifest: finalized.certificate_source_manifest,
        program_certificate: finalized.program_certificate,
        evidence: finalized.evidence,
    })
}

struct FinalizedEvidence {
    scan: PolicyScanV1RunOutput,
    vc: ValidatedVcDocument,
    skeleton: ValidatedVcCertificateSkeleton,
    certificate_manifest: ValidatedSourceManifest,
    certificate_source_manifest: CertificateSourceManifest,
    program_certificate: Option<ProgramCertificateOutcome>,
    evidence: ValidatedPolicyEvidenceV1,
}

fn finalize_evidence<A>(
    invocation: &PolicyVerifyV1Invocation,
    scan: PolicyScanV1RunOutput,
    assembler: &mut A,
) -> Result<FinalizedEvidence, PolicyVerifyV1Error>
where
    A: FnMut(
        &VcDocument,
        &[GroupedTheoremDeclaration],
        CertificateSourceManifest,
    ) -> Result<ProgramCertificateOutcome, ProgramCertificateError>,
{
    if scan.scan.document().readiness != "ready" {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_SCAN_NOT_READY",
            "verify requires the retained internal scan to be ready",
        ));
    }
    let artifacts = scan
        .frontend
        .envelope
        .artifacts
        .as_ref()
        .ok_or_else(|| linkage_error("ready scan omitted validated frontend artifacts"))?;

    // This explicit call is the v1 strategy classification input. Generation
    // of canonical VC below independently revalidates the same VIR/manifest.
    let program = generate_program_vcs(&artifacts.vir)
        .map_err(|error| vc_error(error.code(), error.to_string()))?;
    let vc = generate_vc_v1(&artifacts.vir, &artifacts.source_manifest)
        .map_err(|error| vc_error(error.code(), error.to_string()))?;
    validate_program_projection(&program, vc.document(), &invocation.scan.function)?;
    let skeleton = emit_validated_vc_skeleton_v1(&vc)
        .map_err(|error| vc_error(error.code(), error.to_string()))?;

    let captured = scan
        .captured_inputs
        .iter()
        .map(OwnedCapturedInput::as_ref)
        .collect::<Vec<CapturedInput<'_>>>();
    let expected_language_configuration = (invocation.scan.source_language == "rust").then_some(
        &artifacts
            .source_manifest
            .manifest()
            .target
            .language_configuration,
    );
    let source_context = SourceManifestValidationContext {
        vir: &artifacts.vir,
        source_map: &artifacts.source_map,
        captured_inputs: &captured,
        release_registry: &scan.frontend.registry,
        expected_language_configuration,
    };
    let vc_identity = vc
        .validated_identity()
        .map_err(|error| vc_error(error.code(), error.to_string()))?;
    let certificate_manifest = attach_vc_hash(
        artifacts.source_manifest.canonical_bytes(),
        source_context,
        &vc_identity,
    )
    .map_err(|error| PolicyVerifyV1Error::new("POLICY_MANIFEST_LIFECYCLE", error.to_string()))?;
    let certificate_source_manifest = CertificateSourceManifest {
        payload: certificate_manifest.canonical_bytes().to_vec(),
    };
    let program_certificate = match invocation.scan.source_language.as_str() {
        "rust" => Some(
            assembler(
                vc.document(),
                skeleton.skeleton().theorem_declarations.as_slice(),
                certificate_source_manifest.clone(),
            )
            .map_err(program_certificate_error)?,
        ),
        "go" => None,
        _ => {
            return Err(linkage_error(
                "policy verification reached an unsupported source language",
            ));
        }
    };
    validate_program_certificate_outcome(
        program_certificate.as_ref(),
        &certificate_source_manifest,
    )?;
    let program_declaration_hashes = program_certificate
        .as_ref()
        .map(program_declaration_hashes)
        .transpose()?;
    let verified = matches!(
        program_certificate.as_ref(),
        Some(ProgramCertificateOutcome::Candidate(_))
    );

    let projections = evidence_projections(
        &invocation.scan.function,
        vc.document(),
        skeleton.skeleton().theorem_declarations.as_slice(),
        program_declaration_hashes.as_ref(),
        verified,
    )?;
    let scan_document = scan.scan.document();
    let mut helper_artifacts = scan_document
        .helper_artifacts
        .clone()
        .ok_or_else(|| linkage_error("ready scan omitted helper artifacts"))?;
    helper_artifacts.push(PolicyHelperArtifact::Vc {
        id: "vc".to_owned(),
        schema: VC_SCHEMA_VERSION.to_owned(),
        sha256: vc.hash().as_str().to_owned(),
    });
    let TrustedProgramEvidence {
        certificates,
        axiom_report,
        checker_verdicts,
        expected_certificate,
    } = trusted_program_evidence(
        program_certificate.as_ref(),
        &projections.declarations,
        &invocation.checker_profile,
    )?;
    let mut document = PolicyEvidenceV1 {
        schema: POLICY_EVIDENCE_V1_SCHEMA.to_owned(),
        source_language: scan_document.source_language.clone(),
        semantic_profile: scan_document.semantic_profile.clone(),
        semantic_parameters: scan_document.semantic_parameters.clone(),
        selection: scan_document.selection.clone(),
        limit_profile: scan_document
            .limit_profile
            .clone()
            .ok_or_else(|| linkage_error("ready scan omitted VIR limit profile"))?,
        release_registry: scan_document.release_registry.clone(),
        frontend: scan_document.frontend.clone(),
        toolchain: scan_document.toolchain.clone(),
        frontend_source_manifest_hash: artifacts.source_manifest.hash().as_str().to_owned(),
        input_set_hash: vc.document().input_set_hash.clone(),
        source_map_hash: artifacts.source_map.hash().as_str().to_owned(),
        source_ir_schema: vc.document().source_ir_schema.clone(),
        source_ir_hash: vc.document().source_ir_hash.clone(),
        certificate_source_manifest_hash: certificate_manifest.hash().as_str().to_owned(),
        source_vc_schema: vc.document().schema.clone(),
        vc_hash: vc.hash().as_str().to_owned(),
        verification_limit_profile: vc.document().verification_limit_profile.clone(),
        strategy_profile: invocation.strategy_profile.clone(),
        checker_profile: invocation.checker_profile.clone(),
        axiom_profile: invocation.axiom_profile.clone(),
        verification_options: PolicyVerificationOptions {
            strict: invocation.strict,
            update_fixtures: invocation.update_fixtures,
        },
        helper_artifacts,
        trusted_evidence: PolicyTrustedEvidenceV1 {
            certificates,
            theory_certificates: Vec::new(),
            axiom_report: axiom_report.clone(),
            checker_verdicts: checker_verdicts.clone(),
        },
        properties: projections.properties,
        reproduction_recipes: Vec::new(),
    };
    document.reproduction_recipes = expected_reproduction_recipes(&document);
    let context = PolicyEvidenceLinkageContext {
        scan: &scan.scan,
        certificate_source_manifest_hash: certificate_manifest.hash().as_str().to_owned(),
        source_vc_schema: vc.document().schema.clone(),
        vc_hash: vc.hash().as_str().to_owned(),
        verification_limit_profile: vc.document().verification_limit_profile.clone(),
        expected_members: projections.members,
        expected_declarations: projections.declarations,
        expected_certificate,
        expected_theory_certificates: Vec::new(),
        expected_axiom_report: axiom_report,
        expected_checker_verdicts: checker_verdicts,
        expected_properties: projections.expected_properties,
        expected_unsupported_codes: Vec::new(),
        expected_optional_helpers: Vec::new(),
    };
    let canonical =
        canonical_policy_evidence_v1_json(&document).map_err(policy_validation_error)?;
    let evidence =
        import_policy_evidence_v1_json(&canonical, &context).map_err(policy_validation_error)?;
    Ok(FinalizedEvidence {
        scan,
        vc,
        skeleton,
        certificate_manifest,
        certificate_source_manifest,
        program_certificate,
        evidence,
    })
}

fn validate_program_projection(
    program: &mpk_vc::ProgramVcModule,
    vc: &mpk_vc::VcDocument,
    selected_function: &str,
) -> Result<(), PolicyVerifyV1Error> {
    let generated = program
        .functions
        .iter()
        .find(|function| function.function_id == selected_function)
        .ok_or_else(|| linkage_error("selected function is absent from generated program VCs"))?;
    let canonical = vc
        .functions
        .iter()
        .find(|function| function.function_id == selected_function)
        .ok_or_else(|| linkage_error("selected function is absent from validated VC"))?;
    let generated_members = generated
        .members
        .iter()
        .map(|member| {
            (
                member.id.as_str(),
                member.kind.as_str(),
                member.group_id.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let canonical_members = canonical
        .members
        .iter()
        .map(|member| {
            (
                member.id.as_str(),
                member.kind.as_str(),
                member.group_id.as_str(),
            )
        })
        .collect::<Vec<_>>();
    if generated_members != canonical_members {
        return Err(linkage_error(
            "strategy classification differs from the validated VC member projection",
        ));
    }
    Ok(())
}

fn program_certificate_error(error: ProgramCertificateError) -> PolicyVerifyV1Error {
    let code = match error.kind() {
        ProgramCertificateErrorKind::CheckerExecution => "POLICY_CHECKER_EXECUTION",
        ProgramCertificateErrorKind::CheckerRejected => "POLICY_CHECKER_REJECTED",
        ProgramCertificateErrorKind::CheckerDisagreement => "POLICY_CHECKER_DISAGREEMENT",
        ProgramCertificateErrorKind::Limit(limit) => limit.code(),
        ProgramCertificateErrorKind::Foundation
        | ProgramCertificateErrorKind::Skeleton
        | ProgramCertificateErrorKind::Interface
        | ProgramCertificateErrorKind::Internal => "POLICY_CERTIFICATE_ASSEMBLY",
    };
    PolicyVerifyV1Error::new(code, error.to_string())
}

fn validate_program_certificate_outcome(
    outcome: Option<&ProgramCertificateOutcome>,
    expected_source_manifest: &CertificateSourceManifest,
) -> Result<(), PolicyVerifyV1Error> {
    match outcome {
        None | Some(ProgramCertificateOutcome::Pending { .. }) => Ok(()),
        Some(ProgramCertificateOutcome::Candidate(candidate)) => {
            let decoded = validate_retained_candidate(
                &candidate.bytes,
                &candidate.certificate,
                &candidate.generated_declarations,
                expected_source_manifest,
            )?;
            validate_dual_accepted_reports(
                &candidate.bytes,
                &decoded,
                &candidate.rust_report,
                &candidate.reference_report,
            )
        }
        Some(ProgramCertificateOutcome::Unaccepted(candidate)) => {
            match candidate.failure_kind {
                ProgramCertificateErrorKind::CheckerRejected
                | ProgramCertificateErrorKind::CheckerDisagreement => {}
                ProgramCertificateErrorKind::CheckerExecution => {
                    return Err(PolicyVerifyV1Error::new(
                        "POLICY_CHECKER_EXECUTION",
                        candidate.failure_detail.clone(),
                    ));
                }
                ProgramCertificateErrorKind::Limit(limit) => {
                    return Err(PolicyVerifyV1Error::new(
                        limit.code(),
                        candidate.failure_detail.clone(),
                    ));
                }
                ProgramCertificateErrorKind::Foundation
                | ProgramCertificateErrorKind::Skeleton
                | ProgramCertificateErrorKind::Interface
                | ProgramCertificateErrorKind::Internal => {
                    return Err(PolicyVerifyV1Error::new(
                        "POLICY_CERTIFICATE_ASSEMBLY",
                        candidate.failure_detail.clone(),
                    ));
                }
            }
            let decoded = validate_retained_candidate(
                &candidate.bytes,
                &candidate.certificate,
                &candidate.generated_declarations,
                expected_source_manifest,
            )?;
            validate_unaccepted_reports(&candidate.bytes, &decoded, candidate)
        }
    }
}

fn policy_validation_error(error: PolicyValidationError) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new(error.code(), error.to_string())
}

fn validate_retained_candidate(
    bytes: &[u8],
    retained: &Certificate,
    generated: &[PlannedProgramDeclaration],
    expected_source_manifest: &CertificateSourceManifest,
) -> Result<Certificate, PolicyVerifyV1Error> {
    let decoded = decode_canonical_certificate(bytes).map_err(|error| {
        PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            format!("retained program-certificate bytes are not canonical: {error:?}"),
        )
    })?;
    if &decoded != retained {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "retained program-certificate object differs from its canonical bytes",
        ));
    }
    if decoded.module != PROGRAM_CERTIFICATE_MODULE
        || decoded.source_manifest.as_ref() != Some(expected_source_manifest)
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "retained program certificate has the wrong module or certificate-stage manifest",
        ));
    }
    let rebuilt_export = build_export_block(&decoded).map_err(|error| {
        PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            format!("recompute retained export block: {}", error.detail()),
        )
    })?;
    let rebuilt_axiom_report = build_axiom_report(&decoded).map_err(|error| {
        PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            format!("recompute retained axiom report: {}", error.detail()),
        )
    })?;
    if decoded.export_block != rebuilt_export
        || decoded.axiom_report != rebuilt_axiom_report
        || decoded.hashes.export_hash != export_block_hash(&rebuilt_export)
        || decoded.hashes.axiom_report_hash != axiom_report_hash_for_report(&rebuilt_axiom_report)
        || decoded.hashes.certificate_hash != ZERO_HASH
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "retained candidate sections, hashes, or certificate-hash placeholder are invalid",
        ));
    }
    if rebuilt_axiom_report.summary.total_axiom_count != 0 {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_AXIOM_REPORT",
            "the alpha program certificate did not produce a zero-axiom report",
        ));
    }
    let generated_start = decoded
        .export_block
        .len()
        .checked_sub(generated.len())
        .ok_or_else(|| {
            PolicyVerifyV1Error::new(
                "POLICY_CERTIFICATE_ASSEMBLY",
                "retained certificate has fewer exports than planned generated declarations",
            )
        })?;
    let actual = decoded.export_block[generated_start..]
        .iter()
        .map(|entry| {
            let name = decoded
                .name_table
                .get(entry.name as usize)
                .ok_or_else(|| {
                    PolicyVerifyV1Error::new(
                        "POLICY_CERTIFICATE_ASSEMBLY",
                        "retained generated export has a missing name",
                    )
                })?
                .clone();
            Ok(PlannedProgramDeclaration {
                name,
                declaration_hash: hash_hex(&entry.declaration_hash),
            })
        })
        .collect::<Result<Vec<_>, PolicyVerifyV1Error>>()?;
    if actual != generated {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "planned generated declarations differ from the exact certificate export suffix",
        ));
    }
    Ok(decoded)
}

fn validate_dual_accepted_reports(
    bytes: &[u8],
    certificate: &Certificate,
    rust: &VerificationReport,
    reference: &ReferenceCheckerReport,
) -> Result<(), PolicyVerifyV1Error> {
    let rust_matches = rust_report_matches(bytes, certificate, rust);
    let reference_matches = reference_report_matches(bytes, certificate, reference);
    if rust_matches && reference_matches {
        return Ok(());
    }
    if !accepted_reports_agree(rust, reference) {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CHECKER_DISAGREEMENT",
            "retained dual-accepted checker reports disagree",
        ));
    }
    Err(PolicyVerifyV1Error::new(
        "POLICY_CHECKER_EXECUTION",
        "retained dual-accepted checker reports agree but are not bound to the candidate bytes",
    ))
}

fn accepted_reports_agree(rust: &VerificationReport, reference: &ReferenceCheckerReport) -> bool {
    rust.module == reference.module
        && rust.declaration_count == reference.declaration_count
        && rust.axiom_count == reference.axiom_count
        && hash_hex(&rust.export_hash) == reference.export_hash
        && hash_hex(&rust.axiom_report_hash) == reference.axiom_report_hash
        && hash_hex(&rust.certificate_hash) == reference.certificate_hash
}

fn validate_unaccepted_reports(
    bytes: &[u8],
    certificate: &Certificate,
    candidate: &UnacceptedProgramCertificate,
) -> Result<(), PolicyVerifyV1Error> {
    match (
        candidate.failure_kind,
        candidate.rust_verdict,
        candidate.reference_verdict,
        candidate.rust_report.as_ref(),
        candidate.reference_report.as_ref(),
    ) {
        (
            ProgramCertificateErrorKind::CheckerRejected,
            ProgramCheckerVerdict::Rejected,
            ProgramCheckerVerdict::Rejected,
            None,
            None,
        ) => Ok(()),
        (
            ProgramCertificateErrorKind::CheckerDisagreement,
            ProgramCheckerVerdict::Accepted,
            ProgramCheckerVerdict::Rejected,
            Some(rust),
            None,
        ) if rust_report_matches(bytes, certificate, rust) => Ok(()),
        (
            ProgramCertificateErrorKind::CheckerDisagreement,
            ProgramCheckerVerdict::Rejected,
            ProgramCheckerVerdict::Accepted,
            None,
            Some(reference),
        ) if reference_report_matches(bytes, certificate, reference) => Ok(()),
        _ => Err(PolicyVerifyV1Error::new(
            "POLICY_CHECKER_EXECUTION",
            "unaccepted candidate has a malformed, contradictory, or byte-unbound checker report",
        )),
    }
}

fn rust_report_matches(
    bytes: &[u8],
    certificate: &Certificate,
    report: &VerificationReport,
) -> bool {
    report.module == certificate.module
        && report.declaration_count == certificate.declarations.len()
        && report.axiom_count == certificate.axiom_report.summary.total_axiom_count
        && report.export_hash == certificate.hashes.export_hash
        && report.axiom_report_hash == certificate.hashes.axiom_report_hash
        && report.certificate_hash == certificate_hash(bytes)
        && report.axiom_report == certificate.axiom_report
}

fn reference_report_matches(
    bytes: &[u8],
    certificate: &Certificate,
    report: &ReferenceCheckerReport,
) -> bool {
    report.module == certificate.module
        && report.declaration_count == certificate.declarations.len()
        && report.axiom_count == certificate.axiom_report.summary.total_axiom_count
        && report.export_hash == hash_hex(&certificate.hashes.export_hash)
        && report.axiom_report_hash == hash_hex(&certificate.hashes.axiom_report_hash)
        && report.certificate_hash == hash_hex(&certificate_hash(bytes))
}

fn program_declaration_hashes(
    outcome: &ProgramCertificateOutcome,
) -> Result<BTreeMap<String, String>, PolicyVerifyV1Error> {
    let declarations = outcome.generated_declarations();
    let hashes = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.name.clone(),
                declaration.declaration_hash.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if hashes.len() != declarations.len() {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "program-certificate plan contains duplicate generated declaration names",
        ));
    }
    Ok(hashes)
}

struct TrustedProgramEvidence {
    certificates: Vec<PolicyCertificateEvidenceV1>,
    axiom_report: PolicyAxiomReportV1,
    checker_verdicts: Vec<PolicyCheckerVerdictV1>,
    expected_certificate: Option<PolicyExpectedCertificateV1>,
}

fn trusted_program_evidence(
    outcome: Option<&ProgramCertificateOutcome>,
    checked_declarations: &[PolicyCheckedDeclaration],
    checker_profile: &str,
) -> Result<TrustedProgramEvidence, PolicyVerifyV1Error> {
    let (certificate, bytes, rust_verdict, reference_verdict) = match outcome {
        Some(ProgramCertificateOutcome::Candidate(candidate)) => (
            &candidate.certificate,
            candidate.bytes.as_slice(),
            ProgramCheckerVerdict::Accepted,
            ProgramCheckerVerdict::Accepted,
        ),
        Some(ProgramCertificateOutcome::Unaccepted(candidate)) => (
            &candidate.certificate,
            candidate.bytes.as_slice(),
            candidate.rust_verdict,
            candidate.reference_verdict,
        ),
        None | Some(ProgramCertificateOutcome::Pending { .. }) => {
            return Ok(TrustedProgramEvidence {
                certificates: Vec::new(),
                axiom_report: PolicyAxiomReportV1::NotGenerated,
                checker_verdicts: not_run_checker_verdicts(checker_profile),
                expected_certificate: None,
            });
        }
    };
    let summary = &certificate.axiom_report.summary;
    if summary.total_axiom_count != 0 {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_AXIOM_REPORT",
            "the alpha program certificate did not produce a zero-axiom report",
        ));
    }
    let computed_export_hash = export_block_hash(&certificate.export_block);
    let computed_axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    if certificate.hashes.export_hash != computed_export_hash
        || certificate.hashes.axiom_report_hash != computed_axiom_report_hash
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_ASSEMBLY",
            "retained program-certificate hashes differ from its canonical contents",
        ));
    }
    let certificate_hash = hash_hex(&certificate_hash(bytes));
    let export_hash = hash_hex(&computed_export_hash);
    let axiom_report_hash = hash_hex(&computed_axiom_report_hash);
    let expected_certificate = PolicyExpectedCertificateV1 {
        module: certificate.module.clone(),
        certificate_hash: certificate_hash.clone(),
        export_hash: export_hash.clone(),
        axiom_report_hash: axiom_report_hash.clone(),
    };
    let certificate = PolicyCertificateEvidenceV1 {
        id: "program".to_owned(),
        module: certificate.module.clone(),
        certificate_hash,
        export_hash,
        axiom_report_hash: axiom_report_hash.clone(),
        checked_declarations: checked_declarations.to_vec(),
    };
    let axiom_report = PolicyAxiomReportV1::Checked {
        axiom_report_hash,
        category_counts: axiom_category_counts(summary)?,
    };
    let certificate_ids = vec!["program".to_owned()];
    let checker_verdicts = vec![
        PolicyCheckerVerdictV1 {
            checker: "rust_fast_kernel".to_owned(),
            checker_profile: checker_profile.to_owned(),
            verdict: rust_verdict.as_str().to_owned(),
            certificate_ids: certificate_ids.clone(),
        },
        PolicyCheckerVerdictV1 {
            checker: "reference_checker".to_owned(),
            checker_profile: checker_profile.to_owned(),
            verdict: reference_verdict.as_str().to_owned(),
            certificate_ids,
        },
    ];
    Ok(TrustedProgramEvidence {
        certificates: vec![certificate],
        axiom_report,
        checker_verdicts,
        expected_certificate: Some(expected_certificate),
    })
}

fn not_run_checker_verdicts(checker_profile: &str) -> Vec<PolicyCheckerVerdictV1> {
    vec![
        PolicyCheckerVerdictV1 {
            checker: "rust_fast_kernel".to_owned(),
            checker_profile: checker_profile.to_owned(),
            verdict: "not_run".to_owned(),
            certificate_ids: Vec::new(),
        },
        PolicyCheckerVerdictV1 {
            checker: "reference_checker".to_owned(),
            checker_profile: checker_profile.to_owned(),
            verdict: "not_run".to_owned(),
            certificate_ids: Vec::new(),
        },
    ]
}

fn axiom_category_counts(
    summary: &mpk_cert::encode::AxiomReportSummary,
) -> Result<PolicyAxiomCategoryCountsV1, PolicyVerifyV1Error> {
    Ok(PolicyAxiomCategoryCountsV1 {
        total_axiom_count: checked_axiom_count(summary.total_axiom_count)?,
        core_axiom_count: checked_axiom_count(summary.core_axiom_count)?,
        builtin_theory_axiom_count: checked_axiom_count(summary.builtin_theory_axiom_count)?,
        go_semantics_axiom_count: checked_axiom_count(summary.go_semantics_axiom_count)?,
        external_axiom_count: checked_axiom_count(summary.external_axiom_count)?,
    })
}

fn checked_axiom_count(value: u64) -> Result<i64, PolicyVerifyV1Error> {
    i64::try_from(value).map_err(|_| {
        PolicyVerifyV1Error::new(
            "POLICY_CERTIFICATE_AXIOM_REPORT",
            "checked axiom count exceeds the policy evidence integer range",
        )
    })
}

struct EvidenceProjections {
    properties: Vec<PolicyPropertyV1>,
    members: Vec<PolicyExpectedMemberV1>,
    declarations: Vec<PolicyCheckedDeclaration>,
    expected_properties: Vec<PolicyExpectedPropertyV1>,
}

fn evidence_projections(
    selected_function: &str,
    vc: &mpk_vc::VcDocument,
    skeleton: &[GroupedTheoremDeclaration],
    planned_declaration_hashes: Option<&BTreeMap<String, String>>,
    verified: bool,
) -> Result<EvidenceProjections, PolicyVerifyV1Error> {
    let declaration_hashes = match planned_declaration_hashes {
        Some(hashes) => {
            if hashes.len() != skeleton.len()
                || skeleton
                    .iter()
                    .any(|declaration| !hashes.contains_key(&declaration.name))
            {
                return Err(PolicyVerifyV1Error::new(
                    "POLICY_CERTIFICATE_ASSEMBLY",
                    "program-certificate hashes differ from the complete grouped skeleton",
                ));
            }
            hashes.clone()
        }
        None => skeleton
            .iter()
            .map(|declaration| Ok((declaration.name.clone(), declaration_hash(declaration)?)))
            .collect::<Result<BTreeMap<_, _>, PolicyVerifyV1Error>>()?,
    };
    let declarations = skeleton
        .iter()
        .map(|declaration| {
            let dependencies = declaration
                .dependencies
                .iter()
                .map(|name| {
                    declaration_hashes
                        .get(name)
                        .map(|hash| PolicyDeclarationDependency {
                            name: name.clone(),
                            declaration_hash: hash.clone(),
                        })
                        .ok_or_else(|| linkage_error("skeleton dependency declaration is absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(PolicyCheckedDeclaration {
                name: declaration.name.clone(),
                declaration_hash: declaration_hashes[&declaration.name].clone(),
                function_id: declaration.function_id.clone(),
                group_id: declaration.group_id.clone(),
                group_kind: declaration.group_kind.as_str().to_owned(),
                member_ids: declaration.member_ids.clone(),
                dependencies,
            })
        })
        .collect::<Result<Vec<_>, PolicyVerifyV1Error>>()?;
    let vc_function = vc
        .functions
        .iter()
        .find(|function| function.function_id == selected_function)
        .ok_or_else(|| linkage_error("selected function is absent from VC evidence projection"))?;
    if vc_function.members.is_empty() {
        return Err(linkage_error(
            "selected strategy produced no classifiable VC members",
        ));
    }
    let groups = vc_function
        .groups
        .iter()
        .map(|group| (group.id.as_str(), group))
        .collect::<BTreeMap<_, _>>();
    let mut by_property = BTreeMap::<String, Vec<PolicyMemberRowV1>>::new();
    let mut members = Vec::new();
    for member in &vc_function.members {
        let group = groups
            .get(member.group_id.as_str())
            .ok_or_else(|| linkage_error("VC member group is absent"))?;
        let hash = declaration_hashes
            .get(&group.declaration_name)
            .ok_or_else(|| linkage_error("VC group declaration is absent from skeleton"))?
            .clone();
        let expected = PolicyExpectedMemberV1 {
            member_id: member.id.clone(),
            function_id: member.function_id.clone(),
            kind: member.kind.as_str().to_owned(),
            group_id: member.group_id.clone(),
            declaration_name: group.declaration_name.clone(),
            declaration_hash: hash.clone(),
        };
        members.push(expected);
        by_property
            .entry(property_id(selected_function, member.kind.as_str()))
            .or_default()
            .push(PolicyMemberRowV1 {
                member_id: member.id.clone(),
                function_id: member.function_id.clone(),
                kind: member.kind.as_str().to_owned(),
                group_id: member.group_id.clone(),
                declaration_name: group.declaration_name.clone(),
                declaration_hash: hash,
                status: if verified {
                    "mpk_verified".to_owned()
                } else {
                    "proof_pending".to_owned()
                },
                evidence: if verified {
                    vec![PolicyEvidenceReferenceV1::CheckedDeclaration {
                        certificate_id: "program".to_owned(),
                    }]
                } else {
                    vec![PolicyEvidenceReferenceV1::HelperArtifact {
                        artifact_id: "vc".to_owned(),
                    }]
                },
            });
    }
    members.sort_by(|left, right| left.member_id.as_bytes().cmp(right.member_id.as_bytes()));
    let mut properties = Vec::new();
    let mut expected_properties = Vec::new();
    for (id, mut rows) in by_property {
        rows.sort_by(|left, right| left.member_id.as_bytes().cmp(right.member_id.as_bytes()));
        let description = property_description(selected_function, &rows[0].kind);
        expected_properties.push(PolicyExpectedPropertyV1 {
            id: id.clone(),
            description: description.clone(),
            member_ids: rows.iter().map(|row| row.member_id.clone()).collect(),
            notes: Vec::new(),
        });
        properties.push(PolicyPropertyV1 {
            id,
            description,
            status: if verified {
                "mpk_verified".to_owned()
            } else {
                "proof_pending".to_owned()
            },
            members: rows,
            notes: Vec::new(),
        });
    }
    Ok(EvidenceProjections {
        properties,
        members,
        declarations,
        expected_properties,
    })
}

fn declaration_hash(
    declaration: &GroupedTheoremDeclaration,
) -> Result<String, PolicyVerifyV1Error> {
    let serialized = serde_json::to_vec(declaration)
        .map_err(|error| linkage_error(format!("serialize declaration: {error}")))?;
    let strict = parse_strict_json(
        &serialized,
        StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576),
    )
    .map_err(|error| linkage_error(format!("parse declaration: {error}")))?;
    let canonical = canonical_json_bytes(&strict)
        .map_err(|error| linkage_error(format!("canonicalize declaration: {error}")))?;
    Ok(hash_hex(&hash_with_domain(
        HashDomain::Declaration,
        &canonical,
    )))
}

fn property_id(function: &str, kind: &str) -> String {
    let symbol = function
        .rsplit(['.', ':', '/'])
        .find(|segment| !segment.is_empty())
        .unwrap_or("selected_function");
    format!("{}_{}", ascii_snake(symbol), kind)
}

fn ascii_snake(value: &str) -> String {
    let mut result = String::new();
    for (index, byte) in value.bytes().enumerate() {
        if byte.is_ascii_uppercase() {
            if index > 0 {
                result.push('_');
            }
            result.push(char::from(byte.to_ascii_lowercase()));
        } else if byte.is_ascii_alphanumeric() || byte == b'_' {
            result.push(char::from(byte.to_ascii_lowercase()));
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    result.trim_matches('_').to_owned()
}

fn property_description(function: &str, kind: &str) -> String {
    if function.ends_with(".Identity") && kind == "postcondition" {
        "The selected identity result equals its input.".to_owned()
    } else {
        format!("The selected function satisfies its {kind} verification condition.")
    }
}

fn vc_error(code: &'static str, detail: impl Into<String>) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new(code, detail)
}

fn linkage_error(detail: impl Into<String>) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new("POLICY_SOURCE_LINKAGE", detail)
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) struct PackagePolicyV1 {
    pub(crate) checker_profile: String,
    pub(crate) allowed_axiom_profiles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ActiveReleasePolicyV1 {
    pub(crate) source_language: String,
    pub(crate) semantic_profile: String,
    pub(crate) strategy_profile: String,
    pub(crate) checker_profile: String,
    pub(crate) axiom_profile: String,
}

#[allow(dead_code)]
pub(crate) fn validate_package_release_policy_v1(
    evidence: &ValidatedPolicyEvidenceV1,
    package: &PackagePolicyV1,
    active: &ActiveReleasePolicyV1,
    vc: &ValidatedVcDocument,
    certificate_manifest: &ValidatedSourceManifest,
    recomputed_axiom_report: &PolicyAxiomReportV1,
) -> Result<(), PolicyVerifyV1Error> {
    let document = evidence.document();
    let validated_profiles = validate_package_release_profiles(
        PolicyProfileSelection {
            strategy_profile: &document.strategy_profile,
            checker_profile: &document.checker_profile,
            source_language: &document.source_language,
            semantic_profile: &document.semantic_profile,
            axiom_profile: &document.axiom_profile,
        },
        &package.checker_profile,
        &package.allowed_axiom_profiles,
        PolicyProfileSelection {
            strategy_profile: &active.strategy_profile,
            checker_profile: &active.checker_profile,
            source_language: &active.source_language,
            semantic_profile: &active.semantic_profile,
            axiom_profile: &active.axiom_profile,
        },
    )
    .map_err(|error| PolicyVerifyV1Error::new("POLICY_PACKAGE_PROFILE", error.to_string()))?;
    if vc.hash().as_str() != document.vc_hash
        || certificate_manifest.hash().as_str() != document.certificate_source_manifest_hash
        || certificate_manifest.manifest().vc_hash.as_deref() != Some(document.vc_hash.as_str())
    {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_PACKAGE_LINKAGE",
            "package inputs do not match the retained VC and certificate manifest",
        ));
    }
    if recomputed_axiom_report != &document.trusted_evidence.axiom_report {
        return Err(PolicyVerifyV1Error::new(
            "POLICY_PACKAGE_AXIOM_REPORT",
            "evidence axiom report differs from the recomputed package input",
        ));
    }
    match (validated_profiles.axiom_profile, recomputed_axiom_report) {
        (mpk_api::PolicyAxiomProfile::ZeroAxiom, PolicyAxiomReportV1::NotGenerated)
            if document.trusted_evidence.certificates.is_empty() => {}
        (
            mpk_api::PolicyAxiomProfile::ZeroAxiom,
            PolicyAxiomReportV1::Checked {
                category_counts, ..
            },
        ) if category_counts.total_axiom_count == 0 => {}
        (mpk_api::PolicyAxiomProfile::MvpTheory, PolicyAxiomReportV1::NotGenerated)
            if document.trusted_evidence.certificates.is_empty() => {}
        (
            mpk_api::PolicyAxiomProfile::MvpTheory,
            PolicyAxiomReportV1::Checked {
                category_counts, ..
            },
        ) if summary_only_axiom_report_is_permitted(
            validated_profiles.axiom_profile,
            category_counts.total_axiom_count,
        ) => {}
        _ => {
            return Err(PolicyVerifyV1Error::new(
                "POLICY_PACKAGE_AXIOM_REPORT",
                "recomputed axiom report is incompatible with the active axiom profile",
            ));
        }
    }
    Ok(())
}

#[derive(Debug)]
struct OutputTarget {
    path: PathBuf,
    parent: PathBuf,
    parent_identity: DirectoryIdentity,
    previous_identity: Option<FileIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    canonical_path: PathBuf,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(not(unix))]
    len: u64,
}

struct OutputPreflight {
    json: OutputTarget,
    markdown: OutputTarget,
}

fn preflight_outputs<T>(
    root: &Path,
    json: &str,
    markdown: &str,
    update: bool,
    tracked: &mut T,
) -> Result<OutputPreflight, PolicyVerifyV1Error>
where
    T: FnMut(&Path) -> bool,
{
    if json == markdown || json.eq_ignore_ascii_case(markdown) {
        return Err(output_error(
            "evidence JSON and Markdown outputs alias under portable path rules",
        ));
    }
    let root_identity = directory_identity(root)?;
    Ok(OutputPreflight {
        json: preflight_output(root, &root_identity, json, update, tracked)?,
        markdown: preflight_output(root, &root_identity, markdown, update, tracked)?,
    })
}

fn preflight_output<T>(
    root: &Path,
    root_identity: &DirectoryIdentity,
    relative: &str,
    update: bool,
    tracked: &mut T,
) -> Result<OutputTarget, PolicyVerifyV1Error>
where
    T: FnMut(&Path) -> bool,
{
    let relative = PathBuf::from(relative);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(output_error("evidence output is not normalized relative"));
    }
    let path = root.join(&relative);
    let parent = path
        .parent()
        .ok_or_else(|| output_error("evidence output parent is absent"))?
        .to_path_buf();
    let mut current = root.to_path_buf();
    if let Some(relative_parent) = relative.parent() {
        for component in relative_parent.components() {
            let Component::Normal(component) = component else {
                return Err(output_error("evidence output parent is not normalized"));
            };
            current.push(component);
            let metadata = fs::symlink_metadata(&current)
                .map_err(|error| output_error(format!("evidence output parent: {error}")))?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(output_error(
                    "evidence output parent is not a retained directory",
                ));
            }
        }
    }
    if current != parent {
        return Err(output_error("evidence output escaped the retained root"));
    }
    let parent_identity = directory_identity(&parent)?;
    if !parent_identity
        .canonical_path
        .starts_with(&root_identity.canonical_path)
    {
        return Err(output_error("evidence output escaped the retained root"));
    }
    let previous_identity = match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(output_error(
                    "existing evidence output is not a regular file",
                ));
            }
            #[cfg(unix)]
            if metadata.nlink() != 1 {
                return Err(output_error(
                    "existing evidence output has a hard-link alias",
                ));
            }
            if !update {
                return Err(output_error(
                    "evidence output already exists; use --update-fixtures for tracked fixtures",
                ));
            }
            if !tracked(&relative) {
                return Err(output_error(
                    "--update-fixtures may replace only a tracked evidence fixture",
                ));
            }
            Some(file_identity(&metadata))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(output_error(format!("inspect evidence output: {error}"))),
    };
    Ok(OutputTarget {
        path,
        parent,
        parent_identity,
        previous_identity,
    })
}

fn directory_identity(path: &Path) -> Result<DirectoryIdentity, PolicyVerifyV1Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| output_error(format!("inspect evidence directory: {error}")))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(output_error("evidence directory is not retained"));
    }
    Ok(DirectoryIdentity {
        canonical_path: fs::canonicalize(path)
            .map_err(|error| output_error(format!("resolve evidence directory: {error}")))?,
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
    })
}

fn file_identity(metadata: &fs::Metadata) -> FileIdentity {
    FileIdentity {
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
        #[cfg(not(unix))]
        len: metadata.len(),
    }
}

fn revalidate_target(
    target: &OutputTarget,
    backup_retained: bool,
) -> Result<(), PolicyVerifyV1Error> {
    if directory_identity(&target.parent)? != target.parent_identity {
        return Err(output_error(
            "evidence output parent changed after preflight",
        ));
    }
    match (target.previous_identity, fs::symlink_metadata(&target.path)) {
        (None, Err(error)) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        (Some(expected), Ok(metadata))
            if metadata.is_file()
                && !metadata.file_type().is_symlink()
                && file_identity(&metadata) == expected
                && retained_link_count(&metadata, backup_retained) =>
        {
            Ok(())
        }
        _ => Err(output_error("evidence output changed after preflight")),
    }
}

#[cfg(unix)]
fn retained_link_count(metadata: &fs::Metadata, backup_retained: bool) -> bool {
    metadata.nlink() == if backup_retained { 2 } else { 1 }
}

#[cfg(not(unix))]
fn retained_link_count(_metadata: &fs::Metadata, _backup_retained: bool) -> bool {
    true
}

fn stage_output(
    target: &OutputTarget,
    bytes: &[u8],
) -> Result<(tempfile::NamedTempFile, FileIdentity), PolicyVerifyV1Error> {
    let mut temporary = tempfile::NamedTempFile::new_in(&target.parent)
        .map_err(|error| output_error(format!("create evidence temporary: {error}")))?;
    temporary
        .write_all(bytes)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| output_error(format!("write evidence temporary: {error}")))?;
    let identity = file_identity(
        &temporary
            .as_file()
            .metadata()
            .map_err(|error| output_error(format!("inspect evidence temporary: {error}")))?,
    );
    Ok((temporary, identity))
}

fn reserve_backup(target: &OutputTarget) -> Result<Option<PathBuf>, PolicyVerifyV1Error> {
    if target.previous_identity.is_none() {
        return Ok(None);
    }
    let reservation = tempfile::NamedTempFile::new_in(&target.parent)
        .map_err(|error| output_error(format!("reserve evidence backup: {error}")))?;
    let path = reservation.path().to_path_buf();
    reservation
        .close()
        .map_err(|error| output_error(format!("release evidence backup reservation: {error}")))?;
    fs::hard_link(&target.path, &path)
        .map_err(|error| output_error(format!("retain evidence backup: {error}")))?;
    Ok(Some(path))
}

fn commit_outputs(
    outputs: &OutputPreflight,
    json: &[u8],
    markdown: &[u8],
) -> Result<(), PolicyVerifyV1Error> {
    let (json_temp, json_identity) = stage_output(&outputs.json, json)?;
    let (markdown_temp, markdown_identity) = stage_output(&outputs.markdown, markdown)?;
    revalidate_target(&outputs.json, false)?;
    revalidate_target(&outputs.markdown, false)?;
    let json_backup = reserve_backup(&outputs.json)?;
    let markdown_backup = match reserve_backup(&outputs.markdown) {
        Ok(backup) => backup,
        Err(error) => {
            remove_backup(json_backup.as_deref())?;
            return Err(error);
        }
    };
    let result = publish_pair(
        outputs,
        json_temp,
        markdown_temp,
        json_identity,
        markdown_identity,
        json_backup.as_deref(),
        markdown_backup.as_deref(),
    );
    if result.is_ok() {
        remove_backup(json_backup.as_deref())?;
        remove_backup(markdown_backup.as_deref())?;
        sync_output_parents(outputs)?;
    }
    result
}

fn publish_pair(
    outputs: &OutputPreflight,
    json_temp: tempfile::NamedTempFile,
    markdown_temp: tempfile::NamedTempFile,
    json_identity: FileIdentity,
    markdown_identity: FileIdentity,
    json_backup: Option<&Path>,
    markdown_backup: Option<&Path>,
) -> Result<(), PolicyVerifyV1Error> {
    if let Err(error) = revalidate_target(&outputs.json, json_backup.is_some())
        .and_then(|()| revalidate_target(&outputs.markdown, markdown_backup.is_some()))
    {
        if let Err(cleanup) = cleanup_backups(json_backup, markdown_backup) {
            return Err(output_error(format!(
                "{error}; evidence output recovery required: {cleanup}"
            )));
        }
        return Err(error);
    }
    if outputs.json.previous_identity.is_some() {
        if let Err(error) = fs::remove_file(&outputs.json.path) {
            rollback_outputs(outputs, None, None, json_backup, markdown_backup)?;
            return Err(output_error(format!("replace evidence JSON: {error}")));
        }
    }
    let json_published = match json_temp.persist_noclobber(&outputs.json.path) {
        Ok(published) => published,
        Err(error) => {
            rollback_outputs(outputs, None, None, json_backup, markdown_backup)?;
            return Err(output_error(format!(
                "publish evidence JSON: {}",
                error.error
            )));
        }
    };
    drop(json_published);
    if outputs.markdown.previous_identity.is_some() {
        if let Err(error) = fs::remove_file(&outputs.markdown.path) {
            rollback_outputs(
                outputs,
                Some(json_identity),
                None,
                json_backup,
                markdown_backup,
            )?;
            return Err(output_error(format!("replace evidence Markdown: {error}")));
        }
    }
    match markdown_temp.persist_noclobber(&outputs.markdown.path) {
        Ok(markdown_published) => {
            drop(markdown_published);
            if let Err(error) = validate_published(&outputs.json, json_identity)
                .and_then(|()| validate_published(&outputs.markdown, markdown_identity))
                .and_then(|()| sync_output_parents(outputs))
            {
                rollback_outputs(
                    outputs,
                    Some(json_identity),
                    Some(markdown_identity),
                    json_backup,
                    markdown_backup,
                )?;
                return Err(error);
            }
            Ok(())
        }
        Err(error) => {
            rollback_outputs(
                outputs,
                Some(json_identity),
                None,
                json_backup,
                markdown_backup,
            )?;
            Err(output_error(format!(
                "publish evidence Markdown: {}",
                error.error
            )))
        }
    }
}

fn validate_published(
    target: &OutputTarget,
    expected: FileIdentity,
) -> Result<(), PolicyVerifyV1Error> {
    if directory_identity(&target.parent)? != target.parent_identity {
        return Err(output_error(
            "evidence output parent changed during publish",
        ));
    }
    let metadata = fs::symlink_metadata(&target.path)
        .map_err(|error| output_error(format!("inspect published evidence: {error}")))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || file_identity(&metadata) != expected
    {
        return Err(output_error("published evidence identity changed"));
    }
    #[cfg(unix)]
    if metadata.nlink() != 1 {
        return Err(output_error("published evidence has a hard-link alias"));
    }
    Ok(())
}

fn rollback_outputs(
    outputs: &OutputPreflight,
    json_published: Option<FileIdentity>,
    markdown_published: Option<FileIdentity>,
    json_backup: Option<&Path>,
    markdown_backup: Option<&Path>,
) -> Result<(), PolicyVerifyV1Error> {
    let mut failures = Vec::new();
    if let Err(error) = rollback_target(&outputs.markdown, markdown_published, markdown_backup) {
        failures.push(error.to_string());
    }
    if let Err(error) = rollback_target(&outputs.json, json_published, json_backup) {
        failures.push(error.to_string());
    }
    if let Err(error) = sync_output_parents(outputs) {
        failures.push(error.to_string());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(output_error(format!(
            "evidence output recovery required: {}",
            failures.join("; ")
        )))
    }
}

fn rollback_target(
    target: &OutputTarget,
    published: Option<FileIdentity>,
    backup: Option<&Path>,
) -> Result<(), PolicyVerifyV1Error> {
    if let Some(expected) = published {
        let metadata = fs::symlink_metadata(&target.path)
            .map_err(|error| output_error(format!("inspect evidence rollback: {error}")))?;
        if file_identity(&metadata) != expected || metadata.file_type().is_symlink() {
            return Err(output_error("evidence identity changed before rollback"));
        }
        fs::remove_file(&target.path)
            .map_err(|error| output_error(format!("remove evidence during rollback: {error}")))?;
    }
    if let Some(backup) = backup {
        match fs::symlink_metadata(&target.path) {
            Ok(metadata)
                if target.previous_identity == Some(file_identity(&metadata))
                    && metadata.is_file()
                    && !metadata.file_type().is_symlink() => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::hard_link(backup, &target.path).map_err(|error| {
                    output_error(format!("restore evidence during rollback: {error}"))
                })?;
            }
            _ => return Err(output_error("evidence destination changed during rollback")),
        }
        fs::remove_file(backup)
            .map_err(|error| output_error(format!("remove evidence backup: {error}")))?;
    }
    Ok(())
}

fn remove_backup(path: Option<&Path>) -> Result<(), PolicyVerifyV1Error> {
    if let Some(path) = path {
        fs::remove_file(path)
            .map_err(|error| output_error(format!("remove evidence backup: {error}")))?;
    }
    Ok(())
}

fn cleanup_backups(first: Option<&Path>, second: Option<&Path>) -> Result<(), PolicyVerifyV1Error> {
    let mut failures = Vec::new();
    for path in [first, second].into_iter().flatten() {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => failures.push(error.to_string()),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(output_error(format!(
            "remove evidence backups: {}",
            failures.join("; ")
        )))
    }
}

fn sync_output_parents(outputs: &OutputPreflight) -> Result<(), PolicyVerifyV1Error> {
    let mut parents = BTreeSet::new();
    parents.insert(outputs.json.parent.as_path());
    parents.insert(outputs.markdown.parent.as_path());
    for parent in parents {
        OpenOptions::new()
            .read(true)
            .open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| output_error(format!("synchronize evidence directory: {error}")))?;
    }
    Ok(())
}

fn output_error(detail: impl Into<String>) -> PolicyVerifyV1Error {
    PolicyVerifyV1Error::new("POLICY_CLI_OUTPUT", detail)
}

fn git_tracked(root: &Path, relative: &Path) -> bool {
    Command::new("git")
        .current_dir(root)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(relative)
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy_scan::v1::tests::{
        go_identity_inputs, go_scan_argv, non_success_frontend_run, successful_frontend_run,
    };
    use mpk_vc::validate_release_registry;
    use std::cell::Cell;

    fn verify_argv(json: &str, markdown: &str) -> Vec<String> {
        let mut argv = go_scan_argv();
        argv[2] = "verify".to_owned();
        let json_position = argv
            .iter()
            .position(|argument| argument == "--json-out")
            .unwrap();
        argv.drain(json_position..=json_position + 1);
        argv.extend([
            "--strategy-profile".to_owned(),
            "payment-policy-alpha".to_owned(),
            "--checker-profile".to_owned(),
            "mvp-strict".to_owned(),
            "--axiom-profile".to_owned(),
            "zero-axiom".to_owned(),
            "--evidence-json".to_owned(),
            json.to_owned(),
            "--evidence-md".to_owned(),
            markdown.to_owned(),
        ]);
        argv
    }

    fn run_fixture(
        root: &Path,
        argv: &[String],
    ) -> Result<Option<PolicyVerifyV1RunOutput>, PolicyVerifyV1Error> {
        let inputs = go_identity_inputs();
        run_policy_verify_v1_with(
            argv,
            root,
            inputs.clone(),
            |_| Ok(()),
            |(), _| Ok(successful_frontend_run(&inputs)),
            |_| false,
        )
    }

    #[test]
    fn policy_verify_v1_parser_records_independent_profiles_and_rejects_crossed_tuple() {
        let argv = verify_argv("evidence.json", "evidence.md");
        let parsed = parse_policy_verify_v1_argv(&argv).unwrap().unwrap();
        assert_eq!(parsed.strategy_profile, "payment-policy-alpha");
        assert_eq!(parsed.checker_profile, "mvp-strict");
        assert_eq!(parsed.axiom_profile, "zero-axiom");
        assert_eq!(parsed.scan.contracts, ["contracts/identity.json"]);

        let mut crossed = argv;
        let position = crossed
            .iter()
            .position(|argument| argument == "--axiom-profile")
            .unwrap();
        crossed[position + 1] = "mvp-theory".to_owned();
        assert_eq!(
            parse_policy_verify_v1_argv(&crossed).unwrap_err().code(),
            "POLICY_PROFILE_TUPLE"
        );
    }

    #[test]
    fn policy_verify_v1_parser_enforces_required_options_forbidden_locators_and_output_scalars() {
        let argv = verify_argv("evidence.json", "evidence.md");

        let mut missing = argv.clone();
        let position = missing
            .iter()
            .position(|argument| argument == "--checker-profile")
            .unwrap();
        missing.drain(position..=position + 1);
        assert_eq!(
            parse_policy_verify_v1_argv(&missing).unwrap_err().code(),
            "POLICY_CLI_REQUIRED"
        );

        for option in FORBIDDEN_LOCATORS {
            let mut forbidden = argv.clone();
            forbidden.extend([option.to_owned(), "/tmp/unregistered-frontend".to_owned()]);
            assert_eq!(
                parse_policy_verify_v1_argv(&forbidden).unwrap_err().code(),
                "POLICY_CLI_FORBIDDEN_LOCATOR",
                "{option}"
            );
        }

        let mut malformed_output = argv;
        let position = malformed_output
            .iter()
            .position(|argument| argument == "--evidence-json")
            .unwrap();
        malformed_output[position + 1] = "../evidence.json".to_owned();
        let profile_position = malformed_output
            .iter()
            .position(|argument| argument == "--semantic-profile")
            .unwrap();
        malformed_output[profile_position + 1] = "mpk.rust.checked.v0".to_owned();
        assert_eq!(
            parse_policy_verify_v1_argv(&malformed_output)
                .unwrap_err()
                .code(),
            "POLICY_CLI_SCALAR"
        );
    }

    #[test]
    fn released_verify_release_preflight_precedes_source_capture() {
        let directory = tempfile::tempdir().unwrap();
        let mut argv = verify_argv("evidence.json", "evidence.md");
        argv[3] = "missing-source".to_owned();
        let registry = validate_release_registry(include_bytes!(
            "../../../../release/bundles/bundle-registry.json"
        ))
        .unwrap();
        let hash_position = argv
            .iter()
            .position(|argument| argument == "--require-release-registry-sha256")
            .unwrap();
        argv[hash_position + 1] = registry.registry_digest().to_hex();

        let error = run_cli(&argv, directory.path()).unwrap_err();
        assert_eq!(error.code(), "FRONTEND_SANDBOX_UNAVAILABLE");
        assert!(!directory.path().join("evidence.json").exists());
        assert!(!directory.path().join("evidence.md").exists());
    }

    #[test]
    fn policy_verify_v1_output_aliases_and_symlinks_reject_before_launch() {
        let directory = tempfile::tempdir().unwrap();
        let inputs = go_identity_inputs();
        let launches = Cell::new(0);
        let alias = verify_argv("Evidence.json", "evidence.JSON");
        let error = run_policy_verify_v1_with(
            &alias,
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| {
                launches.set(launches.get() + 1);
                Ok(successful_frontend_run(&inputs))
            },
            |_| false,
        )
        .unwrap_err();
        assert_eq!(error.code(), "POLICY_CLI_OUTPUT");
        assert_eq!(launches.get(), 0);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            fs::write(directory.path().join("real.json"), b"old").unwrap();
            symlink("real.json", directory.path().join("evidence.json")).unwrap();
            let mut symlink_argv = verify_argv("evidence.json", "evidence.md");
            symlink_argv.push("--update-fixtures".to_owned());
            let error = run_policy_verify_v1_with(
                &symlink_argv,
                directory.path(),
                go_identity_inputs(),
                |_| Ok(()),
                |(), _| unreachable!("symlink rejection precedes launch"),
                |_| true,
            )
            .unwrap_err();
            assert_eq!(error.code(), "POLICY_CLI_OUTPUT");
            assert_eq!(
                fs::read(directory.path().join("real.json")).unwrap(),
                b"old"
            );
        }
    }

    #[test]
    fn policy_verify_v1_uses_one_frontend_run_and_emits_valid_pending_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let inputs = go_identity_inputs();
        let prepares = Cell::new(0);
        let launches = Cell::new(0);
        let argv = verify_argv("evidence.json", "evidence.md");
        let output = run_policy_verify_v1_with(
            &argv,
            directory.path(),
            inputs.clone(),
            |_| {
                prepares.set(prepares.get() + 1);
                Ok(())
            },
            |(), _| {
                launches.set(launches.get() + 1);
                Ok(successful_frontend_run(&inputs))
            },
            |_| false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(prepares.get(), 1);
        assert_eq!(launches.get(), 1);
        assert!(!directory.path().join(INTERNAL_SCAN_OUTPUT).exists());
        assert_eq!(
            fs::read(directory.path().join("evidence.json")).unwrap(),
            output.evidence.canonical_bytes()
        );
        assert!(fs::read_to_string(directory.path().join("evidence.md"))
            .unwrap()
            .contains("proof_pending"));
        assert!(output
            .evidence
            .document()
            .properties
            .iter()
            .flat_map(|property| &property.members)
            .all(|member| member.status == "proof_pending"
                && member.evidence
                    == [PolicyEvidenceReferenceV1::HelperArtifact {
                        artifact_id: "vc".to_owned()
                    }]));
        assert_eq!(
            output.certificate_source_manifest.payload,
            output.certificate_manifest.canonical_bytes()
        );
    }

    #[test]
    fn policy_verify_v1_manifest_finalization_mutates_only_vc_and_self_hash() {
        let directory = tempfile::tempdir().unwrap();
        let output = run_fixture(
            directory.path(),
            &verify_argv("evidence.json", "evidence.md"),
        )
        .unwrap()
        .unwrap();
        let frontend: serde_json::Value = serde_json::from_slice(
            output
                .scan
                .frontend
                .envelope
                .artifacts
                .as_ref()
                .unwrap()
                .source_manifest
                .canonical_bytes(),
        )
        .unwrap();
        let certificate: serde_json::Value =
            serde_json::from_slice(output.certificate_manifest.canonical_bytes()).unwrap();
        let mut normalized_certificate = certificate.clone();
        normalized_certificate
            .as_object_mut()
            .unwrap()
            .remove("vc_hash");
        normalized_certificate["source_manifest_hash"] = frontend["source_manifest_hash"].clone();
        assert_eq!(normalized_certificate, frontend);
        assert_ne!(
            output.evidence.document().frontend_source_manifest_hash,
            output.evidence.document().certificate_source_manifest_hash
        );
    }

    #[test]
    fn policy_verify_v1_strict_writes_valid_untrusted_evidence_then_fails() {
        let directory = tempfile::tempdir().unwrap();
        let mut argv = verify_argv("strict.json", "strict.md");
        argv.push("--strict".to_owned());
        let error = run_fixture(directory.path(), &argv).unwrap_err();
        assert_eq!(error.code(), "POLICY_PROOF_PENDING");
        let bytes = fs::read(directory.path().join("strict.json")).unwrap();
        let document: PolicyEvidenceV1 = serde_json::from_slice(&bytes).unwrap();
        assert!(document.verification_options.strict);
        assert!(document.trusted_evidence.certificates.is_empty());
        assert!(matches!(
            document.trusted_evidence.axiom_report,
            PolicyAxiomReportV1::NotGenerated
        ));
        assert!(directory.path().join("strict.md").exists());
    }

    #[test]
    fn policy_verify_v1_nonready_internal_scan_commits_no_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let argv = verify_argv("evidence.json", "evidence.md");
        let error = run_policy_verify_v1_with(
            &argv,
            directory.path(),
            Vec::new(),
            |_| Ok(()),
            |(), _| Ok(non_success_frontend_run("rejected", "subset", 3)),
            |_| false,
        )
        .unwrap_err();
        assert_eq!(error.code(), "POLICY_SCAN_NOT_READY");
        assert!(!directory.path().join("evidence.json").exists());
        assert!(!directory.path().join("evidence.md").exists());
        assert!(!directory.path().join(INTERNAL_SCAN_OUTPUT).exists());
    }

    #[test]
    fn policy_verify_v1_output_preflight_precedes_frontend_launch_and_requires_explicit_tracked_update(
    ) {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("evidence.json"), b"old-json").unwrap();
        fs::write(directory.path().join("evidence.md"), b"old-md").unwrap();
        let argv = verify_argv("evidence.json", "evidence.md");
        let launches = Cell::new(0);
        let inputs = go_identity_inputs();
        let error = run_policy_verify_v1_with(
            &argv,
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| {
                launches.set(launches.get() + 1);
                Ok(successful_frontend_run(&inputs))
            },
            |_| false,
        )
        .unwrap_err();
        assert_eq!(error.code(), "POLICY_CLI_OUTPUT");
        assert_eq!(launches.get(), 0);

        let mut update = argv;
        update.push("--update-fixtures".to_owned());
        let error = run_policy_verify_v1_with(
            &update,
            directory.path(),
            go_identity_inputs(),
            |_| Ok(()),
            |(), _| unreachable!("untracked fixture rejection precedes launch"),
            |_| false,
        )
        .unwrap_err();
        assert_eq!(error.code(), "POLICY_CLI_OUTPUT");
        assert_eq!(
            fs::read(directory.path().join("evidence.json")).unwrap(),
            b"old-json"
        );

        let inputs = go_identity_inputs();
        let output = run_policy_verify_v1_with(
            &update,
            directory.path(),
            inputs.clone(),
            |_| Ok(()),
            |(), _| Ok(successful_frontend_run(&inputs)),
            |_| true,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            fs::read(directory.path().join("evidence.json")).unwrap(),
            output.evidence.canonical_bytes()
        );
    }

    #[test]
    fn policy_verify_v1_package_gate_binds_active_profiles_and_manifest() {
        let directory = tempfile::tempdir().unwrap();
        let output = run_fixture(
            directory.path(),
            &verify_argv("evidence.json", "evidence.md"),
        )
        .unwrap()
        .unwrap();
        let package = PackagePolicyV1 {
            checker_profile: "mvp-strict".to_owned(),
            allowed_axiom_profiles: vec!["zero-axiom".to_owned()],
        };
        let active = ActiveReleasePolicyV1 {
            source_language: "go".to_owned(),
            semantic_profile: "mpk.go.fixed.v0".to_owned(),
            strategy_profile: "payment-policy-alpha".to_owned(),
            checker_profile: "mvp-strict".to_owned(),
            axiom_profile: "zero-axiom".to_owned(),
        };
        validate_package_release_policy_v1(
            &output.evidence,
            &package,
            &active,
            &output.vc,
            &output.certificate_manifest,
            &PolicyAxiomReportV1::NotGenerated,
        )
        .unwrap();
        let mut crossed = active;
        crossed.checker_profile = "mvp-structural".to_owned();
        assert_eq!(
            validate_package_release_policy_v1(
                &output.evidence,
                &package,
                &crossed,
                &output.vc,
                &output.certificate_manifest,
                &PolicyAxiomReportV1::NotGenerated,
            )
            .unwrap_err()
            .code(),
            "POLICY_PACKAGE_PROFILE"
        );
        let checked = PolicyAxiomReportV1::Checked {
            axiom_report_hash: "11".repeat(32),
            category_counts: crate::policy_schema::PolicyAxiomCategoryCountsV1 {
                total_axiom_count: 0,
                core_axiom_count: 0,
                builtin_theory_axiom_count: 0,
                go_semantics_axiom_count: 0,
                external_axiom_count: 0,
            },
        };
        assert_eq!(
            validate_package_release_policy_v1(
                &output.evidence,
                &package,
                &ActiveReleasePolicyV1 {
                    checker_profile: "mvp-strict".to_owned(),
                    ..crossed
                },
                &output.vc,
                &output.certificate_manifest,
                &checked,
            )
            .unwrap_err()
            .code(),
            "POLICY_PACKAGE_AXIOM_REPORT"
        );
    }
}
