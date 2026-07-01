use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const POLICY_SCAN_SCHEMA: &str = "mpk.policy.scan.v0";
const GO2GIR_CLI_SCHEMA: &str = "mpk.go2gir.cli.v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyScanRequest {
    pub target: String,
    pub function_id: String,
    pub contract_path: String,
    pub go2gir_path: PathBuf,
}

pub fn run_policy_scan(
    request: &PolicyScanRequest,
) -> Result<PolicyScanReport, PolicyScanRunError> {
    Ok(run_policy_scan_with_artifacts(request)?.report)
}

pub fn run_policy_scan_with_artifacts(
    request: &PolicyScanRequest,
) -> Result<PolicyScanRunOutput, PolicyScanRunError> {
    let current_dir = std::env::current_dir()
        .map_err(|error| PolicyScanRunError::io("resolve current directory", error))?;
    let go2gir_path = resolve_existing_path(&current_dir, &request.go2gir_path, "go2gir binary")?;
    let go2gir_sha256 = file_sha256(&go2gir_path)?;
    let target_layout = PolicyScanTargetLayout::resolve(&current_dir, &request.target);
    let contract = ContractMetadata::load(&current_dir, &request.contract_path);

    let output = Command::new(&go2gir_path)
        .current_dir(&target_layout.working_dir)
        .arg(&target_layout.package_arg)
        .output()
        .map_err(|error| PolicyScanRunError::io("run go2gir", error))?;
    let go2gir = parse_go2gir_output(&output.stdout, output.status, &output.stderr)?;
    let gir_json = raw_gir_json(&output.stdout)?;
    let report =
        build_policy_scan_report(request, &target_layout, &contract, go2gir_sha256, go2gir)?;

    Ok(PolicyScanRunOutput { report, gir_json })
}

fn raw_gir_json(stdout: &[u8]) -> Result<Option<String>, PolicyScanRunError> {
    let value = serde_json::from_slice::<Value>(stdout)
        .map_err(|error| PolicyScanRunError::with_source("parse go2gir raw JSON", error))?;
    value
        .get("gir")
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| PolicyScanRunError::with_source("encode raw GIR artifact", error))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyScanRunOutput {
    pub report: PolicyScanReport,
    pub gir_json: Option<String>,
}

#[derive(Debug)]
pub struct PolicyScanRunError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl PolicyScanRunError {
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

    fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::with_source(context, source)
    }
}

impl fmt::Display for PolicyScanRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PolicyScanRunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

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

impl PolicyScanReadinessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsRefactor => "needs_refactor",
            Self::Unsupported => "unsupported",
        }
    }
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
struct PolicyScanTargetLayout {
    source_root: String,
    working_dir: PathBuf,
    package_arg: String,
}

impl PolicyScanTargetLayout {
    fn resolve(current_dir: &Path, target: &str) -> Self {
        let target_path = Path::new(target);
        let resolved = if target_path.is_absolute() {
            target_path.to_path_buf()
        } else {
            current_dir.join(target_path)
        };
        if resolved.is_dir() {
            Self {
                source_root: normalize_slashes(target.trim_start_matches("./")),
                working_dir: resolved,
                package_arg: ".".to_owned(),
            }
        } else {
            Self {
                source_root: normalize_slashes(target),
                working_dir: current_dir.to_path_buf(),
                package_arg: target.to_owned(),
            }
        }
    }
}

#[derive(Debug)]
struct ContractMetadata {
    path: String,
    schema: Option<String>,
    function_id: Option<String>,
    sha256: Option<String>,
    read_error: Option<String>,
    parse_error: Option<String>,
}

impl ContractMetadata {
    fn load(current_dir: &Path, contract_path: &str) -> Self {
        let path = Path::new(contract_path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };
        let bytes = match fs::read(&resolved) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Self {
                    path: normalize_slashes(contract_path),
                    schema: None,
                    function_id: None,
                    sha256: None,
                    read_error: Some(error.to_string()),
                    parse_error: None,
                };
            }
        };
        let sha256 = Some(sha256_hex(&bytes));
        let value = match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => value,
            Err(error) => {
                return Self {
                    path: normalize_slashes(contract_path),
                    schema: None,
                    function_id: None,
                    sha256,
                    read_error: None,
                    parse_error: Some(error.to_string()),
                };
            }
        };

        Self {
            path: normalize_slashes(contract_path),
            schema: value
                .get("schema")
                .and_then(Value::as_str)
                .map(str::to_owned),
            function_id: value
                .get("function")
                .and_then(Value::as_str)
                .map(str::to_owned),
            sha256,
            read_error: None,
            parse_error: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Go2GirCliOutput {
    schema: String,
    status: String,
    package_path: String,
    #[serde(default)]
    packages: Vec<Go2GirPackage>,
    #[serde(default)]
    gir: Option<Go2GirModule>,
    #[serde(default)]
    source_manifest: Option<Go2GirSourceManifest>,
    #[serde(default)]
    rejected_features: Vec<Go2GirRejectedFeature>,
}

#[derive(Debug, Deserialize)]
struct Go2GirPackage {
    package_path: String,
    #[serde(default)]
    go_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Go2GirModule {
    #[serde(default)]
    packages: Vec<Go2GirGirPackage>,
    #[serde(default)]
    gir_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Go2GirGirPackage {
    #[serde(default)]
    functions: Vec<Go2GirFunction>,
}

#[derive(Debug, Deserialize)]
struct Go2GirFunction {
    id: String,
    package: String,
    #[serde(default)]
    contracts: Go2GirContracts,
    #[serde(default)]
    supported_features: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Go2GirContracts {
    #[serde(default)]
    requires: Vec<Value>,
    #[serde(default)]
    ensures: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct Go2GirRejectedFeature {
    #[serde(default)]
    location: Option<String>,
    feature: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct Go2GirSourceManifest {
    go_version: String,
    frontend: Go2GirSourceManifestFrontend,
    #[serde(default)]
    source_files: Vec<Go2GirSourceManifestFile>,
    source_hash: String,
    #[serde(default)]
    gir_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Go2GirSourceManifestFrontend {
    binary_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Go2GirSourceManifestFile {
    path: String,
    sha256: String,
}

fn build_policy_scan_report(
    request: &PolicyScanRequest,
    target_layout: &PolicyScanTargetLayout,
    contract: &ContractMetadata,
    go2gir_sha256: String,
    go2gir: Go2GirCliOutput,
) -> Result<PolicyScanReport, PolicyScanRunError> {
    let target_function = go2gir
        .gir
        .as_ref()
        .and_then(|gir| find_gir_function(gir, &request.function_id));
    let package_path = target_function
        .map(|function| function.package.as_str())
        .or_else(|| package_path_for_requested_function(&go2gir, &request.function_id))
        .unwrap_or(go2gir.package_path.as_str())
        .to_owned();
    let source = scan_source(target_layout, &go2gir, go2gir_sha256)?;
    let readiness = scan_readiness(&go2gir, target_function);
    let contract_status = scan_contract_status(&readiness.status, contract, &go2gir);
    let scan_contract = PolicyScanContract {
        path: Some(contract.path.clone()),
        schema: contract.schema.clone(),
        sha256: contract.sha256.clone(),
        status: contract_status,
        function_id: if contract_status == PolicyScanContractStatus::FunctionResolved {
            Some(request.function_id.clone())
        } else {
            contract.function_id.clone()
        },
    };

    let mut report = PolicyScanReport::new(
        PolicyScanTarget::new(package_path, request.function_id.clone()),
        source,
        scan_contract,
        readiness,
    );

    if let Some(function) = target_function {
        let source_path = report.source.files.first().map(|file| file.path.clone());
        for feature in &function.supported_features {
            report.supported_features.push(PolicyScanFeature::helper(
                feature_code("GO2GIR_SUPPORTED", feature),
                format!("{feature} is supported by go2gir"),
                source_path.clone(),
                Some(function.id.clone()),
            ));
        }
        for (index, requires) in function.contracts.requires.iter().enumerate() {
            report.preconditions.push(PolicyScanPrecondition::helper(
                format!("requires[{index}]"),
                format_contract_expr(requires),
                PolicyScanPreconditionSource::ContractRequires,
                Some(contract.path.clone()),
                Some(function.id.clone()),
            ));
        }
    }

    for rejected in &go2gir.rejected_features {
        let (source_path, location) =
            rejected_location(target_layout, rejected.location.as_deref());
        let mut feature = PolicyScanFeature::helper(
            feature_code("GO2GIR_REJECTED", &rejected.feature),
            rejected.reason.clone(),
            source_path,
            Some(request.function_id.clone()),
        );
        feature.location = location;
        report.rejected_features.push(feature);
    }

    Ok(report)
}

fn scan_source(
    target_layout: &PolicyScanTargetLayout,
    go2gir: &Go2GirCliOutput,
    go2gir_sha256: String,
) -> Result<PolicyScanSource, PolicyScanRunError> {
    if let Some(manifest) = &go2gir.source_manifest {
        let mut source = PolicyScanSource::new(
            target_layout.source_root.clone(),
            manifest.go_version.clone(),
            manifest.frontend.binary_sha256.clone(),
            manifest.source_hash.clone(),
        );
        source.gir_sha256 = manifest.gir_hash.clone();
        source.files = manifest
            .source_files
            .iter()
            .map(|file| {
                PolicyScanSourceFile::new(
                    source_path(target_layout, &file.path),
                    file.sha256.clone(),
                )
            })
            .collect();
        return Ok(source);
    }

    let source_files = compute_source_files(target_layout, go2gir)?;
    let source_hash = source_manifest_hash(&source_files)?;
    let mut source = PolicyScanSource::new(
        target_layout.source_root.clone(),
        "unknown",
        go2gir_sha256,
        source_hash,
    );
    source.files = source_files
        .into_iter()
        .map(|file| PolicyScanSourceFile::new(source_path(target_layout, &file.path), file.sha256))
        .collect();
    source.gir_sha256 = go2gir.gir.as_ref().and_then(|gir| gir.gir_hash.clone());
    Ok(source)
}

fn scan_readiness(
    go2gir: &Go2GirCliOutput,
    target_function: Option<&Go2GirFunction>,
) -> PolicyScanReadiness {
    if go2gir.status == "gir-lowered" {
        let Some(function) = target_function else {
            return PolicyScanReadiness::new(
                PolicyScanReadinessStatus::NeedsRefactor,
                "target function was not lowered by go2gir",
            );
        };
        if function.contracts.ensures.is_empty() {
            return PolicyScanReadiness::new(
                PolicyScanReadinessStatus::NeedsRefactor,
                "target function lacks a usable contract",
            );
        }
        return PolicyScanReadiness::new(
            PolicyScanReadinessStatus::Ready,
            "function is within Go subset v0 and the contract resolves",
        );
    }

    if !go2gir.rejected_features.is_empty()
        && go2gir
            .rejected_features
            .iter()
            .all(|feature| feature.feature == "contract sidecar")
    {
        return PolicyScanReadiness::new(
            PolicyScanReadinessStatus::NeedsRefactor,
            "contract sidecar is not usable for target function",
        );
    }

    PolicyScanReadiness::new(
        PolicyScanReadinessStatus::Unsupported,
        "go2gir rejected unsupported Go subset features",
    )
}

fn scan_contract_status(
    status: &PolicyScanReadinessStatus,
    contract: &ContractMetadata,
    go2gir: &Go2GirCliOutput,
) -> PolicyScanContractStatus {
    if *status == PolicyScanReadinessStatus::Ready {
        return PolicyScanContractStatus::FunctionResolved;
    }
    if contract.read_error.is_some() {
        return PolicyScanContractStatus::NotProvided;
    }
    if contract.parse_error.is_some() {
        return PolicyScanContractStatus::Unsupported;
    }
    if go2gir.rejected_features.iter().any(|feature| {
        feature.feature == "contract sidecar" && feature.reason.contains("does not resolve")
    }) {
        return PolicyScanContractStatus::FunctionNotFound;
    }
    if go2gir
        .rejected_features
        .iter()
        .any(|feature| feature.feature == "contract sidecar")
    {
        return PolicyScanContractStatus::Unsupported;
    }
    PolicyScanContractStatus::Parsed
}

fn parse_go2gir_output(
    stdout: &[u8],
    status: ExitStatus,
    stderr: &[u8],
) -> Result<Go2GirCliOutput, PolicyScanRunError> {
    let result = serde_json::from_slice::<Go2GirCliOutput>(stdout).map_err(|error| {
        if status.success() {
            PolicyScanRunError::with_source("go2gir produced invalid JSON", error)
        } else {
            PolicyScanRunError::with_source(
                format!(
                    "go2gir failed without valid JSON: status={} stderr={}",
                    status,
                    compact_bytes(stderr)
                ),
                error,
            )
        }
    })?;
    if result.schema != GO2GIR_CLI_SCHEMA {
        return Err(PolicyScanRunError::new(format!(
            "go2gir schema = {:?}, want {:?}",
            result.schema, GO2GIR_CLI_SCHEMA
        )));
    }
    Ok(result)
}

fn find_gir_function<'a>(gir: &'a Go2GirModule, function_id: &str) -> Option<&'a Go2GirFunction> {
    gir.packages
        .iter()
        .flat_map(|package| package.functions.iter())
        .find(|function| function.id == function_id)
}

fn package_path_for_requested_function<'a>(
    go2gir: &'a Go2GirCliOutput,
    function_id: &str,
) -> Option<&'a str> {
    go2gir
        .packages
        .iter()
        .find(|package| function_id.starts_with(&format!("{}.", package.package_path)))
        .map(|package| package.package_path.as_str())
        .or_else(|| {
            go2gir
                .packages
                .first()
                .map(|package| package.package_path.as_str())
        })
}

fn compute_source_files(
    target_layout: &PolicyScanTargetLayout,
    go2gir: &Go2GirCliOutput,
) -> Result<Vec<Go2GirSourceManifestFile>, PolicyScanRunError> {
    let mut source_files = Vec::new();
    for file in go2gir
        .packages
        .iter()
        .flat_map(|package| package.go_files.iter())
    {
        if source_files
            .iter()
            .any(|existing: &Go2GirSourceManifestFile| existing.path == *file)
        {
            continue;
        }
        let path = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            target_layout.working_dir.join(file)
        };
        source_files.push(Go2GirSourceManifestFile {
            path: normalize_slashes(file),
            sha256: file_sha256(&path)?,
        });
    }
    source_files.sort_by(|lhs, rhs| lhs.path.cmp(&rhs.path));
    Ok(source_files)
}

#[derive(Serialize)]
struct SourceHashPayload<'a> {
    source_files: &'a [Go2GirSourceManifestFile],
}

fn source_manifest_hash(
    source_files: &[Go2GirSourceManifestFile],
) -> Result<String, PolicyScanRunError> {
    let encoded = serde_json::to_vec(&SourceHashPayload { source_files })
        .map_err(|error| PolicyScanRunError::with_source("encode source manifest hash", error))?;
    Ok(sha256_hex(&encoded))
}

fn rejected_location(
    target_layout: &PolicyScanTargetLayout,
    location: Option<&str>,
) -> (Option<String>, Option<PolicyScanLocation>) {
    let Some(location) = location else {
        return (None, None);
    };
    let mut parts = location.rsplitn(3, ':');
    let column = parts.next();
    let line = parts.next();
    let path = parts.next();
    match (path, line, column) {
        (Some(path), Some(line), Some(column)) => {
            let parsed = line
                .parse::<u32>()
                .ok()
                .zip(column.parse::<u32>().ok())
                .map(|(line, column)| PolicyScanLocation::new(line, column));
            (Some(source_path(target_layout, path)), parsed)
        }
        _ => (Some(source_path(target_layout, location)), None),
    }
}

fn feature_code(prefix: &str, feature: &str) -> String {
    let mut suffix = String::new();
    let mut previous_underscore = false;
    for character in feature.chars() {
        if character.is_ascii_alphanumeric() {
            suffix.push(character.to_ascii_uppercase());
            previous_underscore = false;
        } else if !previous_underscore {
            suffix.push('_');
            previous_underscore = true;
        }
    }
    while suffix.ends_with('_') {
        suffix.pop();
    }
    if suffix.is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix}_{suffix}")
    }
}

fn format_contract_expr(value: &Value) -> String {
    let Some(op) = value.get("op").and_then(Value::as_str) else {
        return compact_json(value);
    };
    let binary = value
        .get("lhs")
        .zip(value.get("rhs"))
        .and_then(|(lhs, rhs)| Some((format_contract_atom(lhs)?, format_contract_atom(rhs)?)));
    if let Some((lhs, rhs)) = binary {
        let symbol = match op {
            "eq" => "==",
            "signed_ge" | "unsigned_ge" => ">=",
            "signed_gt" | "unsigned_gt" => ">",
            "signed_le" | "unsigned_le" => "<=",
            "signed_lt" | "unsigned_lt" => "<",
            _ => return compact_json(value),
        };
        return format!("{lhs} {symbol} {rhs}");
    }
    compact_json(value)
}

fn format_contract_atom(value: &Value) -> Option<String> {
    if let Some(var) = value.get("var").and_then(Value::as_str) {
        return Some(var.to_owned());
    }
    if let Some(result) = value.get("result").and_then(Value::as_u64) {
        return Some(format!("result[{result}]"));
    }
    if let Some(int) = value.get("int") {
        if let Some(raw) = int.get("value").and_then(Value::as_str) {
            return Some(raw.to_owned());
        }
    }
    if let Some(bool_value) = value.get("bool").and_then(Value::as_bool) {
        return Some(bool_value.to_string());
    }
    None
}

fn source_path(target_layout: &PolicyScanTargetLayout, path: &str) -> String {
    let normalized = normalize_slashes(path);
    if normalized.is_empty()
        || normalized.starts_with('/')
        || target_layout.source_root.is_empty()
        || target_layout.source_root == "."
        || normalized == target_layout.source_root
        || normalized.starts_with(&format!("{}/", target_layout.source_root))
    {
        normalized
    } else {
        format!("{}/{}", target_layout.source_root, normalized)
    }
}

fn resolve_existing_path(
    current_dir: &Path,
    path: &Path,
    label: &str,
) -> Result<PathBuf, PolicyScanRunError> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    if !candidate.is_file() {
        return Err(PolicyScanRunError::new(format!(
            "{label} not found: {}",
            normalize_slashes(&candidate.display().to_string())
        )));
    }
    candidate
        .canonicalize()
        .map_err(|error| PolicyScanRunError::io(format!("canonicalize {label}"), error))
}

fn file_sha256(path: &Path) -> Result<String, PolicyScanRunError> {
    let bytes = fs::read(path).map_err(|error| {
        PolicyScanRunError::io(format!("read {} for SHA-256", path.display()), error)
    })?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

fn compact_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
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

    #[test]
    fn non_lowered_go2gir_output_without_rejections_is_unsupported() {
        let go2gir = Go2GirCliOutput {
            schema: GO2GIR_CLI_SCHEMA.to_owned(),
            status: "rejected".to_owned(),
            package_path: ".".to_owned(),
            packages: Vec::new(),
            gir: None,
            source_manifest: None,
            rejected_features: Vec::new(),
        };

        let readiness = scan_readiness(&go2gir, None);
        assert_eq!(readiness.status, PolicyScanReadinessStatus::Unsupported);
    }
}
