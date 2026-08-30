//! Active successor policy and explanation command routes.
//!
//! These routes accept only revision-2 semantic-context and selection
//! envelopes. Bundle identities and profile contracts are compiled into the
//! release; callers cannot select paths, predecessor registries, or alternate
//! contract payloads.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_semantic_profile_registry,
    validate_semantic_request, CompiledSemanticProfile, ProfileContractField, RegistryRevision,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_vc::{emit_successor_vc_skeleton, generate_successor_vc, SuccessorVcSource};
use mpk_vc::{parse_strict_json, CapturedInput, StrictJsonLimits, StrictJsonValue};
use serde_json::{json, Map, Value};

use crate::policy_scan::v1::{capture_successor_staging, OwnedFrontendStaging, PolicyScanV1Error};
use crate::program_certificate::ProgramCertificateOutcome;
use crate::successor_ai_explain::{
    prepare_successor_ai_explanation, ExplainLanguageV1, SuccessorAiSource,
};
use crate::successor_frontend_runner::{
    run_installed_frontend, AcceptedInstalledFrontendRun, InstalledFrontendRunRequest,
};
use crate::successor_policy::{
    generate_successor_policy_scan, run_successor_policy, PolicyVerificationOptions,
    SuccessorPolicyScanSource, SuccessorPolicySource,
};
use crate::successor_release_bundle::{
    ACTIVE_RELEASE_REGISTRY_SHA256, CSHARP_FRONTEND_BUNDLE_ID, CSHARP_TOOLCHAIN_BUNDLE_ID,
    GO_FRONTEND_BUNDLE_ID, GO_TOOLCHAIN_BUNDLE_ID, RUST_FRONTEND_BUNDLE_ID,
    RUST_TOOLCHAIN_BUNDLE_ID, SUCCESSOR_RELEASE_REGISTRY_ID,
};

pub const POLICY_SCAN_USAGE: &str = "mpk policy scan <source-root> --semantic-context <context.json> --selection <selection.json> [--contract <normalized-path> ...] --json-out <scan.json>";
pub const POLICY_VERIFY_USAGE: &str = "mpk policy verify <source-root> --semantic-context <context.json> --selection <selection.json> [--contract <normalized-path> ...] --evidence-json <evidence.json>";
pub const EXPLAIN_USAGE: &str = "mpk explain <source-root> --semantic-context <context.json> --selection <selection.json> [--contract <normalized-path> ...] [--language <en|ja>] --request-json-out <sanitized-request.json>";

const ACTIVE_SEMANTIC_REGISTRY: &[u8] =
    include_bytes!("../../../release/bundles/semantic-profile-registry.json");
const COMMAND_JSON_LIMITS: StrictJsonLimits = StrictJsonLimits::new(65_536, 65_536, 32, 65_536);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorCliErrorKind {
    Usage,
    Input,
}

#[derive(Debug)]
pub struct SuccessorCliError {
    kind: SuccessorCliErrorKind,
    detail: String,
}

impl SuccessorCliError {
    pub const fn kind(&self) -> SuccessorCliErrorKind {
        self.kind
    }
}

impl std::fmt::Display for SuccessorCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for SuccessorCliError {}

#[derive(Clone, Copy)]
enum CommandMode {
    Scan,
    Verify,
    Explain,
}

impl CommandMode {
    const fn usage(self) -> &'static str {
        match self {
            Self::Scan => POLICY_SCAN_USAGE,
            Self::Verify => POLICY_VERIFY_USAGE,
            Self::Explain => EXPLAIN_USAGE,
        }
    }

    const fn output_flag(self) -> &'static str {
        match self {
            Self::Scan => "--json-out",
            Self::Verify => "--evidence-json",
            Self::Explain => "--request-json-out",
        }
    }
}

struct ParsedInvocation {
    source_root: PathBuf,
    semantic_context: PathBuf,
    selection: PathBuf,
    contracts: Vec<String>,
    output: PathBuf,
    language: ExplainLanguageV1,
}

struct PreparedInvocation {
    registry: ValidatedSemanticProfileRegistry,
    semantic_context: Value,
    selection: Value,
    staging: OwnedFrontendStaging,
    frontend_bundle_id: &'static str,
    toolchain_bundle_id: &'static str,
    launcher_contracts: Vec<String>,
}

pub fn run_policy_scan(
    arguments: &[String],
    working_directory: &Path,
) -> Result<String, SuccessorCliError> {
    let parsed = parse_invocation(CommandMode::Scan, arguments, working_directory)?;
    let prepared = prepare_invocation(&parsed)?;
    let mut output = OutputReservation::create(parsed.output)?;
    let accepted = run_frontend(&prepared)?;
    let artifacts = accepted_artifacts(&accepted)?;
    let captured = captured_refs(&prepared.staging);
    let policy_contract = active_contract(
        &prepared.registry,
        artifacts
            .vir()
            .module()
            .semantic_context()
            .source_language(),
        artifacts
            .vir()
            .module()
            .semantic_context()
            .semantic_profile(),
        ProfileContractField::Policy,
    )?;
    let scan = generate_successor_policy_scan(SuccessorPolicyScanSource {
        registry: &prepared.registry,
        vir: artifacts.vir(),
        source_map: artifacts.source_map(),
        frontend_manifest: artifacts.source_manifest(),
        policy_contract: &policy_contract,
        captured_inputs: &captured,
    })
    .map_err(|error| input(format!("policy scan failed: {error}")))?;
    output.commit(scan.canonical_bytes())?;
    Ok(format!(
        "ok policy scan schema={} json={}",
        scan.document().schema(),
        output.path().display()
    ))
}

pub fn run_policy_verify(
    arguments: &[String],
    working_directory: &Path,
) -> Result<String, SuccessorCliError> {
    let parsed = parse_invocation(CommandMode::Verify, arguments, working_directory)?;
    let prepared = prepare_invocation(&parsed)?;
    let mut output = OutputReservation::create(parsed.output)?;
    let accepted = run_frontend(&prepared)?;
    let artifacts = accepted_artifacts(&accepted)?;
    let captured = captured_refs(&prepared.staging);
    let profile = artifacts.vir().module().semantic_context();
    let vc_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Vc,
    )?;
    let vc_source = SuccessorVcSource {
        registry: &prepared.registry,
        vir: artifacts.vir(),
        manifest: artifacts.source_manifest(),
        profile_contract: &vc_contract,
    };
    let vc = generate_successor_vc(vc_source)
        .map_err(|error| input(format!("policy verify VC generation failed: {error}")))?;
    let skeleton = emit_successor_vc_skeleton(&vc, vc_source)
        .map_err(|error| input(format!("policy verify skeleton generation failed: {error}")))?;
    let policy_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Policy,
    )?;
    let evidence_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Evidence,
    )?;
    let run = run_successor_policy(
        SuccessorPolicySource {
            registry: &prepared.registry,
            vir: artifacts.vir(),
            source_map: artifacts.source_map(),
            frontend_manifest: artifacts.source_manifest(),
            vc: &vc,
            skeleton: &skeleton,
            policy_contract: &policy_contract,
            evidence_contract: &evidence_contract,
            captured_inputs: &captured,
        },
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    )
    .map_err(|error| input(format!("policy verify failed: {error}")))?;
    require_accepted_certificate(run.program_certificate())?;
    output.commit(run.evidence().canonical_bytes())?;
    Ok(format!(
        "ok policy verify schema={} evidence={}",
        run.evidence().document().schema(),
        output.path().display()
    ))
}

pub fn run_explain(
    arguments: &[String],
    working_directory: &Path,
) -> Result<String, SuccessorCliError> {
    let parsed = parse_invocation(CommandMode::Explain, arguments, working_directory)?;
    let prepared = prepare_invocation(&parsed)?;
    let mut output = OutputReservation::create(parsed.output)?;
    let accepted = run_frontend(&prepared)?;
    let artifacts = accepted_artifacts(&accepted)?;
    let captured = captured_refs(&prepared.staging);
    let profile = artifacts.vir().module().semantic_context();
    let vc_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Vc,
    )?;
    let vc_source = SuccessorVcSource {
        registry: &prepared.registry,
        vir: artifacts.vir(),
        manifest: artifacts.source_manifest(),
        profile_contract: &vc_contract,
    };
    let vc = generate_successor_vc(vc_source)
        .map_err(|error| input(format!("explanation VC generation failed: {error}")))?;
    let skeleton = emit_successor_vc_skeleton(&vc, vc_source)
        .map_err(|error| input(format!("explanation skeleton generation failed: {error}")))?;
    let policy_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Policy,
    )?;
    let evidence_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Evidence,
    )?;
    let run = run_successor_policy(
        SuccessorPolicySource {
            registry: &prepared.registry,
            vir: artifacts.vir(),
            source_map: artifacts.source_map(),
            frontend_manifest: artifacts.source_manifest(),
            vc: &vc,
            skeleton: &skeleton,
            policy_contract: &policy_contract,
            evidence_contract: &evidence_contract,
            captured_inputs: &captured,
        },
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    )
    .map_err(|error| input(format!("explanation evidence generation failed: {error}")))?;
    require_accepted_certificate(run.program_certificate())?;
    let ai_contract = active_contract(
        &prepared.registry,
        profile.source_language(),
        profile.semantic_profile(),
        ProfileContractField::Ai,
    )?;
    let request = prepare_successor_ai_explanation(
        SuccessorAiSource {
            registry: &prepared.registry,
            evidence: run.evidence(),
            ai_contract: &ai_contract,
        },
        parsed.language,
    )
    .map_err(|error| input(format!("explanation request generation failed: {error}")))?;
    output.commit(request.canonical_request_bytes())?;
    Ok(format!(
        "ok explanation request schema={} json={}",
        request.document().schema(),
        output.path().display()
    ))
}

fn parse_invocation(
    mode: CommandMode,
    arguments: &[String],
    working_directory: &Path,
) -> Result<ParsedInvocation, SuccessorCliError> {
    let Some(source_root) = arguments.first() else {
        return Err(usage(mode, "source-root positional is missing"));
    };
    if source_root.is_empty() || source_root.starts_with("--") {
        return Err(usage(mode, "source-root positional is invalid"));
    }

    let mut semantic_context = None;
    let mut selection = None;
    let mut contracts = Vec::new();
    let mut output = None;
    let mut language = ExplainLanguageV1::En;
    let mut language_seen = false;
    let mut index = 1;
    while index < arguments.len() {
        let flag = arguments[index].as_str();
        let Some(value) = arguments.get(index + 1) else {
            return Err(usage(mode, format!("{flag} requires a value")));
        };
        if value.is_empty() || value.starts_with("--") {
            return Err(usage(mode, format!("{flag} requires a non-empty value")));
        }
        match flag {
            "--semantic-context" => set_once(&mut semantic_context, value, mode, flag)?,
            "--selection" => set_once(&mut selection, value, mode, flag)?,
            "--contract" => contracts.push(value.clone()),
            candidate if candidate == mode.output_flag() => {
                set_once(&mut output, value, mode, flag)?
            }
            "--language" if matches!(mode, CommandMode::Explain) => {
                if language_seen {
                    return Err(usage(mode, "--language was supplied more than once"));
                }
                language = match value.as_str() {
                    "en" => ExplainLanguageV1::En,
                    "ja" => ExplainLanguageV1::Ja,
                    _ => return Err(usage(mode, "--language must be en or ja")),
                };
                language_seen = true;
            }
            _ => return Err(usage(mode, format!("unknown flag: {flag}"))),
        }
        index += 2;
    }

    let semantic_context =
        semantic_context.ok_or_else(|| usage(mode, "--semantic-context is required"))?;
    let selection = selection.ok_or_else(|| usage(mode, "--selection is required"))?;
    let output =
        output.ok_or_else(|| usage(mode, format!("{} is required", mode.output_flag())))?;
    normalize_contracts(&mut contracts, mode)?;
    Ok(ParsedInvocation {
        source_root: resolve_path(working_directory, source_root),
        semantic_context: resolve_path(working_directory, &semantic_context),
        selection: resolve_path(working_directory, &selection),
        contracts,
        output: resolve_path(working_directory, &output),
        language,
    })
}

fn prepare_invocation(parsed: &ParsedInvocation) -> Result<PreparedInvocation, SuccessorCliError> {
    let registry =
        validate_semantic_profile_registry(ACTIVE_SEMANTIC_REGISTRY, RegistryRevision::Revision2)
            .map_err(|error| input(format!("compiled semantic registry is invalid: {error}")))?;
    let semantic_context = read_strict_value(&parsed.semantic_context, "semantic context")?;
    let selection = read_strict_value(&parsed.selection, "selection")?;
    let request = json!({
        "semantic_context": semantic_context,
        "selection": selection
    });
    let validated = validate_semantic_request(&registry, &request)
        .map_err(|error| input(format!("semantic request is invalid: {error}")))?;
    let semantic_context = serde_json::to_value(validated.semantic_context())
        .map_err(|error| input(format!("serialize semantic context: {error}")))?;
    let selection = serde_json::to_value(validated.selection())
        .map_err(|error| input(format!("serialize selection: {error}")))?;
    let language = validated.semantic_context().source_language();
    let (frontend_bundle_id, toolchain_bundle_id) = release_pair(language)?;
    let (capture_contracts, launcher_contracts) = if language == "csharp" {
        if !parsed.contracts.is_empty() {
            return Err(input(
                "C# contract paths are selected only by the validated selection envelope",
            ));
        }
        let contracts = validated.selection().value()["contracts"]
            .as_array()
            .ok_or_else(|| input("C# selection has no contract list"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| input("C# selection contract path is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        (contracts, Vec::new())
    } else {
        if parsed.contracts.is_empty() {
            return Err(input("Go and Rust successor routes require --contract"));
        }
        (parsed.contracts.clone(), parsed.contracts.clone())
    };
    let staging = capture_successor_staging(&parsed.source_root, language, &capture_contracts)
        .map_err(capture_error)?;
    Ok(PreparedInvocation {
        registry,
        semantic_context,
        selection,
        staging,
        frontend_bundle_id,
        toolchain_bundle_id,
        launcher_contracts,
    })
}

fn run_frontend(
    prepared: &PreparedInvocation,
) -> Result<AcceptedInstalledFrontendRun, SuccessorCliError> {
    let captured = captured_refs(&prepared.staging);
    let csharp = prepared.semantic_context["source_language"] == "csharp";
    let staged_directories = if csharp {
        Vec::new()
    } else {
        prepared
            .staging
            .staged_directories
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    let staged_placeholders = if csharp {
        Vec::new()
    } else {
        prepared
            .staging
            .staged_placeholders
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    run_installed_frontend(InstalledFrontendRunRequest {
        semantic_context: &prepared.semantic_context,
        selection: &prepared.selection,
        release_registry_id: SUCCESSOR_RELEASE_REGISTRY_ID,
        release_registry_sha256: ACTIVE_RELEASE_REGISTRY_SHA256,
        frontend_bundle_id: prepared.frontend_bundle_id,
        toolchain_bundle_id: prepared.toolchain_bundle_id,
        captured_inputs: &captured,
        synthetic_permissions: &[],
        staged_directories: &staged_directories,
        staged_placeholders: &staged_placeholders,
        contracts: &prepared.launcher_contracts,
    })
    .map_err(|error| input(format!("installed successor frontend failed: {error}")))
}

fn accepted_artifacts(
    accepted: &AcceptedInstalledFrontendRun,
) -> Result<
    &crate::successor_frontend_protocol::AcceptedSuccessorFrontendArtifacts,
    SuccessorCliError,
> {
    accepted
        .envelope()
        .artifacts()
        .ok_or_else(|| input("installed successor frontend returned no accepted artifacts"))
}

fn captured_refs(staging: &OwnedFrontendStaging) -> Vec<CapturedInput<'_>> {
    staging
        .captured_inputs
        .iter()
        .map(|input| input.as_ref())
        .collect()
}

fn release_pair(language: &str) -> Result<(&'static str, &'static str), SuccessorCliError> {
    match language {
        "go" => Ok((GO_FRONTEND_BUNDLE_ID, GO_TOOLCHAIN_BUNDLE_ID)),
        "rust" => Ok((RUST_FRONTEND_BUNDLE_ID, RUST_TOOLCHAIN_BUNDLE_ID)),
        "csharp" => Ok((CSHARP_FRONTEND_BUNDLE_ID, CSHARP_TOOLCHAIN_BUNDLE_ID)),
        _ => Err(input(
            "semantic request selected an inactive source language",
        )),
    }
}

fn active_contract(
    registry: &ValidatedSemanticProfileRegistry,
    source_language: &str,
    semantic_profile: &str,
    field: ProfileContractField,
) -> Result<Value, SuccessorCliError> {
    let entry = registry
        .lookup(source_language, semantic_profile)
        .ok_or_else(|| input("semantic profile entry is absent from the active registry"))?;
    let value = match (entry.compiled_profile(), field) {
        (CompiledSemanticProfile::GoFixedV0, ProfileContractField::Ai) => json!({
            "display_language":"Go",
            "projection_profile_id":"mpk.go.ai_projection.v0",
            "proof_authority":false,
            "redaction_profile_id":"minimal-v1",
            "source_access":false
        }),
        (CompiledSemanticProfile::RustCheckedV0, ProfileContractField::Ai) => json!({
            "display_language":"Rust",
            "projection_profile_id":"mpk.rust.ai_projection.v0",
            "proof_authority":false,
            "redaction_profile_id":"minimal-v1",
            "source_access":false
        }),
        (CompiledSemanticProfile::CSharpScalarV0, ProfileContractField::Ai) => json!({
            "display_language":"C#",
            "projection_profile_id":"mpk.csharp.ai_projection.v0",
            "proof_authority":false,
            "redaction_profile_id":"minimal-v1",
            "source_access":false
        }),
        (profile, ProfileContractField::Evidence) => json!({
            "proof_authority":"certificate_only",
            "recipe_profile_id":match profile {
                CompiledSemanticProfile::GoFixedV0 => "mpk.go.evidence_recipe.v0",
                CompiledSemanticProfile::RustCheckedV0 => "mpk.rust.evidence_recipe.v0",
                CompiledSemanticProfile::CSharpScalarV0 => "mpk.csharp.evidence_recipe.v0",
            },
            "require_reference_checker":true,
            "require_source_free_check":true
        }),
        (profile, ProfileContractField::Policy) => json!({
            "axiom_profile":match profile {
                CompiledSemanticProfile::GoFixedV0 => "zero-axiom",
                CompiledSemanticProfile::RustCheckedV0 | CompiledSemanticProfile::CSharpScalarV0 => "mvp-theory",
            },
            "checker_profile":"mvp-strict",
            "strategy_profile":match profile {
                CompiledSemanticProfile::GoFixedV0 => "payment-policy-alpha",
                CompiledSemanticProfile::RustCheckedV0 => "payment-policy-rust-alpha",
                CompiledSemanticProfile::CSharpScalarV0 => "payment-policy-csharp-alpha",
            }
        }),
        (profile, ProfileContractField::Vc) => json!({
            "contract_profile_id":match profile {
                CompiledSemanticProfile::GoFixedV0 => "mpk.go.contract.v0",
                CompiledSemanticProfile::RustCheckedV0 => "mpk.rust.contract.v0",
                CompiledSemanticProfile::CSharpScalarV0 => "mpk.csharp.contract.v0",
            },
            "required_check_profile_id":match profile {
                CompiledSemanticProfile::GoFixedV0 => "mpk.go.fixed.v0",
                CompiledSemanticProfile::RustCheckedV0 => "mpk.rust.checked.v0",
                CompiledSemanticProfile::CSharpScalarV0 => "mpk.csharp.required_checks.v0",
            },
            "verification_limit_profile_id":"mpk.verify.limits.v0"
        }),
        _ => {
            return Err(input(
                "command requested an unsupported compiled profile contract",
            ))
        }
    };
    let envelope = json!({
        "profile_entry_sha256":entry.entry_sha256(),
        "contract_id":entry.contracts().contract_id(field),
        "value":value
    });
    validate_compiled_profile_envelope(registry, &envelope, field)
        .map_err(|error| input(format!("compiled profile contract is invalid: {error}")))?;
    Ok(envelope)
}

fn require_accepted_certificate(
    outcome: &ProgramCertificateOutcome,
) -> Result<(), SuccessorCliError> {
    match outcome {
        ProgramCertificateOutcome::Candidate(_) => Ok(()),
        ProgramCertificateOutcome::Pending {
            missing_member_ids, ..
        } => Err(input(format!(
            "strict verification has {} proof-pending members",
            missing_member_ids.len()
        ))),
        ProgramCertificateOutcome::Unaccepted(candidate) => Err(input(format!(
            "source-free checkers did not accept the certificate: {}",
            candidate.failure_detail
        ))),
    }
}

fn read_strict_value(path: &Path, label: &str) -> Result<Value, SuccessorCliError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| input(format!("read {label} {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(input(format!("{label} must be a regular non-symlink file")));
    }
    let bytes = fs::read(path)
        .map_err(|error| input(format!("read {label} {}: {error}", path.display())))?;
    let strict = parse_strict_json(&bytes, COMMAND_JSON_LIMITS)
        .map_err(|error| input(format!("parse {label}: {error}")))?;
    Ok(strict_to_serde(strict))
}

fn strict_to_serde(value: StrictJsonValue) -> Value {
    match value {
        StrictJsonValue::Null => Value::Null,
        StrictJsonValue::Bool(value) => Value::Bool(value),
        StrictJsonValue::Integer(value) => Value::Number(value.into()),
        StrictJsonValue::String(value) => Value::String(value),
        StrictJsonValue::Array(values) => {
            Value::Array(values.into_iter().map(strict_to_serde).collect())
        }
        StrictJsonValue::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(name, value)| (name, strict_to_serde(value)))
                .collect::<Map<_, _>>(),
        ),
    }
}

fn normalize_contracts(
    contracts: &mut [String],
    mode: CommandMode,
) -> Result<(), SuccessorCliError> {
    for contract in contracts.iter() {
        mpk_vc::validate_manifest_normalized_path(contract)
            .map_err(|error| usage(mode, format!("invalid --contract path: {error}")))?;
    }
    contracts.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    if contracts.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(usage(mode, "duplicate --contract path"));
    }
    Ok(())
}

fn set_once(
    slot: &mut Option<String>,
    value: &str,
    mode: CommandMode,
    flag: &str,
) -> Result<(), SuccessorCliError> {
    if slot.replace(value.to_owned()).is_some() {
        Err(usage(mode, format!("{flag} was supplied more than once")))
    } else {
        Ok(())
    }
}

fn resolve_path(working_directory: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_owned()
    } else {
        working_directory.join(path)
    }
}

fn capture_error(error: PolicyScanV1Error) -> SuccessorCliError {
    input(format!("source capture failed: {error}"))
}

fn usage(mode: CommandMode, detail: impl std::fmt::Display) -> SuccessorCliError {
    SuccessorCliError {
        kind: SuccessorCliErrorKind::Usage,
        detail: format!("{detail}\nusage: {}", mode.usage()),
    }
}

fn input(detail: impl Into<String>) -> SuccessorCliError {
    SuccessorCliError {
        kind: SuccessorCliErrorKind::Input,
        detail: detail.into(),
    }
}

struct OutputReservation {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl OutputReservation {
    fn create(path: PathBuf) -> Result<Self, SuccessorCliError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| input(format!("reserve output {}: {error}", path.display())))?;
        Ok(Self {
            path,
            file: Some(file),
            committed: false,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn commit(&mut self, bytes: &[u8]) -> Result<(), SuccessorCliError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| input("output reservation is already closed"))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| input(format!("write output {}: {error}", self.path.display())))?;
        self.file.take();
        self.committed = true;
        Ok(())
    }
}

impl Drop for OutputReservation {
    fn drop(&mut self) {
        self.file.take();
        if !self.committed {
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => {}
            }
        }
    }
}
