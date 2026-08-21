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
    if let Some(error) = &contract.parse_error {
        return Err(PolicyScanRunError::new(format!(
            "contract JSON failed to parse: {}: {error}",
            contract.path
        )));
    }

    let output = Command::new(&go2gir_path)
        .current_dir(&target_layout.working_dir)
        .arg(&target_layout.package_arg)
        .output()
        .map_err(|error| PolicyScanRunError::io("run go2gir", error))?;
    let go2gir = parse_go2gir_output(&output.stdout, output.status, &output.stderr)?;
    let gir_json = raw_gir_json(&output.stdout)?;
    let report =
        build_policy_scan_report(request, &target_layout, &contract, go2gir_sha256, go2gir)?;
    if report.readiness.status == PolicyScanReadinessStatus::Ready {
        if let Some(error) = &contract.read_error {
            return Err(PolicyScanRunError::new(format!(
                "contract path not found: {}: {error}",
                contract.path
            )));
        }
    }

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

/// Staged policy v1 scan path. The released command tree remains wired to the
/// v0 implementation above until the atomic GO-VIR-02-T12 cutover.
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod v1 {
    use crate::frontend_protocol::AcceptedFrontendArtifacts;
    use crate::frontend_runner::{
        prepare_installed_frontend, run_prepared_frontend, AcceptedFrontendRun,
        FrontendReleaseIdentity, FrontendRunRequest,
    };
    use crate::policy_schema::{
        canonical_policy_scan_v1_json, import_policy_scan_v1_json, PolicyGoSelection,
        PolicyGoSemanticParameters, PolicyHelperArtifact, PolicyIssue, PolicyRustSelection,
        PolicyRustSemanticParameters, PolicyScanLinkageContext, PolicyScanV1, PolicySelection,
        PolicySemanticParameters, ValidatedPolicyScanV1, POLICY_SCAN_V1_SCHEMA,
    };
    use mpk_vc::{
        canonical_json_bytes, parse_strict_json, CapturedInput, InputKind, ReleaseSelectionRequest,
        StrictJsonLimits,
    };
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::collections::{BTreeMap, BTreeSet};
    use std::error::Error;
    use std::fmt;
    use std::fs::{self, OpenOptions};
    use std::io::{self, Write};
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    const SCAN_OPTIONS: [&str; 11] = [
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
        "--json-out",
    ];
    const FORBIDDEN_LOCATORS: [&str; 9] = [
        "--frontend",
        "--frontend-helper",
        "--driver",
        "--go2gir",
        "--toolchain-root",
        "--toolchain-path",
        "--registry",
        "--registry-path",
        "--release-registry-path",
    ];
    const REQUIRED_OPTIONS: [&str; 10] = [
        "--language",
        "--semantic-profile",
        "--require-release-registry-id",
        "--require-release-registry-sha256",
        "--frontend-bundle",
        "--toolchain-bundle",
        "--target",
        "--package",
        "--function",
        "--json-out",
    ];

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub(crate) struct PolicyScanV1Invocation {
        pub(crate) source_root: String,
        pub(crate) source_language: String,
        pub(crate) semantic_profile: String,
        pub(crate) registry_id: String,
        pub(crate) registry_sha256: String,
        pub(crate) frontend_bundle_id: String,
        pub(crate) toolchain_bundle_id: String,
        pub(crate) target_id: String,
        pub(crate) package: String,
        pub(crate) function: String,
        pub(crate) contracts: Vec<String>,
        pub(crate) json_out: String,
    }

    impl PolicyScanV1Invocation {
        pub(crate) fn release_request(&self) -> ReleaseSelectionRequest {
            ReleaseSelectionRequest {
                registry_id: self.registry_id.clone(),
                registry_sha256: self.registry_sha256.clone(),
                source_language: self.source_language.clone(),
                semantic_profile: self.semantic_profile.clone(),
                target_id: self.target_id.clone(),
                frontend_bundle_id: Some(self.frontend_bundle_id.clone()),
                toolchain_bundle_id: Some(self.toolchain_bundle_id.clone()),
            }
        }

        pub(crate) fn semantic_parameters(&self) -> PolicySemanticParameters {
            match self.source_language.as_str() {
                "go" => PolicySemanticParameters::Go(PolicyGoSemanticParameters {
                    target_id: self.target_id.clone(),
                    pointer_width: 64,
                }),
                "rust" => PolicySemanticParameters::Rust(PolicyRustSemanticParameters {
                    target_id: self.target_id.clone(),
                    pointer_width: 64,
                    overflow_mode: "checked".to_owned(),
                    panic_mode: "abort".to_owned(),
                }),
                _ => unreachable!("the parser closes the source-language set"),
            }
        }

        pub(crate) fn selection(&self) -> PolicySelection {
            match self.source_language.as_str() {
                "go" => PolicySelection::Go(PolicyGoSelection {
                    package: self.package.clone(),
                    function: self.function.clone(),
                }),
                "rust" => PolicySelection::Rust(PolicyRustSelection {
                    package: self.package.clone(),
                    crate_name: self
                        .function
                        .split("::")
                        .next()
                        .expect("the parser requires a Rust crate segment")
                        .to_owned(),
                    kind: "lib".to_owned(),
                    function: self.function.clone(),
                }),
                _ => unreachable!("the parser closes the source-language set"),
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct OwnedCapturedInput {
        pub(crate) kind: InputKind,
        pub(crate) normalized_path: String,
        pub(crate) bytes: Vec<u8>,
    }

    impl OwnedCapturedInput {
        pub(crate) fn as_ref(&self) -> CapturedInput<'_> {
            CapturedInput {
                kind: self.kind,
                normalized_path: &self.normalized_path,
                bytes: &self.bytes,
            }
        }
    }

    #[derive(Debug)]
    pub(crate) struct PolicyScanV1RunOutput {
        pub(crate) invocation: PolicyScanV1Invocation,
        pub(crate) scan: ValidatedPolicyScanV1,
        pub(crate) frontend: AcceptedFrontendRun,
        pub(crate) captured_inputs: Vec<OwnedCapturedInput>,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct PolicyScanV1Error {
        code: &'static str,
        detail: String,
    }

    impl PolicyScanV1Error {
        pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
            Self {
                code,
                detail: detail.into(),
            }
        }

        pub(crate) const fn code(&self) -> &'static str {
            self.code
        }
    }

    impl fmt::Display for PolicyScanV1Error {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}: {}", self.code, self.detail)
        }
    }

    impl Error for PolicyScanV1Error {}

    pub(crate) fn parse_policy_scan_v1_argv(
        argv: &[String],
    ) -> Result<Option<PolicyScanV1Invocation>, PolicyScanV1Error> {
        let Some(invocation) = parse_policy_scan_v1_argv_through_scalars(argv)? else {
            return Ok(None);
        };
        validate_policy_scan_v1_profile(&invocation)?;
        Ok(Some(invocation))
    }

    /// Shared CLI prefix used by staged verify so its evidence-path scalar
    /// checks remain before language/profile tuple validation.
    pub(crate) fn parse_policy_scan_v1_argv_through_scalars(
        argv: &[String],
    ) -> Result<Option<PolicyScanV1Invocation>, PolicyScanV1Error> {
        if argv.first().map(String::as_str) != Some("mpk")
            || argv.get(1).map(String::as_str) != Some("policy")
            || argv.get(2).map(String::as_str) != Some("scan")
        {
            return Err(cli_error(
                "POLICY_CLI_ARGUMENT",
                "expected the exact mpk policy scan route",
            ));
        }
        let source_root = argv
            .get(3)
            .ok_or_else(|| cli_error("POLICY_CLI_ARGUMENT", "source-root positional is missing"))?;
        if argv.len() == 4 && matches!(source_root.as_str(), "help" | "-h" | "--help") {
            return Ok(None);
        }
        if source_root.starts_with("--") {
            recognize_option_tokens(&argv[3..])?;
        } else {
            recognize_option_tokens(&argv[4..])?;
        }
        if matches!(source_root.as_str(), "help" | "-h" | "--help") {
            return Err(cli_error(
                if source_root.starts_with('-') {
                    "POLICY_CLI_UNKNOWN_OPTION"
                } else {
                    "POLICY_CLI_ARGUMENT"
                },
                "help cannot be mixed with scan arguments",
            ));
        }
        if source_root.is_empty() || source_root.starts_with("--") {
            return Err(cli_error(
                "POLICY_CLI_ARGUMENT",
                "source-root must be one non-option positional",
            ));
        }
        let mut singleton = BTreeMap::<&str, String>::new();
        let mut contracts = Vec::new();
        let mut position = 4;
        while position < argv.len() {
            let option = argv[position].as_str();
            if !SCAN_OPTIONS.contains(&option) {
                return Err(cli_error(
                    "POLICY_CLI_ARGUMENT",
                    "unexpected extra positional argument",
                ));
            }
            let value = argv.get(position + 1).ok_or_else(|| {
                cli_error("POLICY_CLI_ARGUMENT", "option requires a separate value")
            })?;
            if value.is_empty() || value.starts_with('-') {
                return Err(cli_error(
                    "POLICY_CLI_ARGUMENT",
                    "option requires a nonempty separate value",
                ));
            }
            if option == "--contract" {
                if contracts.iter().any(|contract| contract == value) {
                    return Err(cli_error(
                        "POLICY_CLI_ARGUMENT",
                        "duplicate identical contract option",
                    ));
                }
                contracts.push(value.clone());
            } else if singleton.insert(option, value.clone()).is_some() {
                return Err(cli_error(
                    "POLICY_CLI_ARGUMENT",
                    "duplicate singleton option",
                ));
            }
            position += 2;
        }

        if REQUIRED_OPTIONS
            .iter()
            .any(|option| !singleton.contains_key(option))
            || contracts.is_empty()
        {
            return Err(cli_error(
                "POLICY_CLI_REQUIRED",
                "a mandatory scan option or contract is missing",
            ));
        }

        let mut invocation = PolicyScanV1Invocation {
            source_root: source_root.clone(),
            source_language: take(&mut singleton, "--language"),
            semantic_profile: take(&mut singleton, "--semantic-profile"),
            registry_id: take(&mut singleton, "--require-release-registry-id"),
            registry_sha256: take(&mut singleton, "--require-release-registry-sha256"),
            frontend_bundle_id: take(&mut singleton, "--frontend-bundle"),
            toolchain_bundle_id: take(&mut singleton, "--toolchain-bundle"),
            target_id: take(&mut singleton, "--target"),
            package: take(&mut singleton, "--package"),
            function: take(&mut singleton, "--function"),
            contracts,
            json_out: take(&mut singleton, "--json-out"),
        };
        validate_invocation_scalars(&mut invocation)?;
        Ok(Some(invocation))
    }

    fn recognize_option_tokens(arguments: &[String]) -> Result<(), PolicyScanV1Error> {
        for token in arguments.iter().filter(|token| token.starts_with('-')) {
            if FORBIDDEN_LOCATORS.contains(&token.as_str()) {
                return Err(cli_error(
                    "POLICY_CLI_FORBIDDEN_LOCATOR",
                    "raw frontend, helper, toolchain, or registry locators are forbidden",
                ));
            }
            if !SCAN_OPTIONS.contains(&token.as_str()) {
                return Err(cli_error(
                    "POLICY_CLI_UNKNOWN_OPTION",
                    "option is not accepted by policy scan v1",
                ));
            }
        }
        Ok(())
    }

    fn take(values: &mut BTreeMap<&str, String>, name: &str) -> String {
        values
            .remove(name)
            .expect("presence was checked before constructing the invocation")
    }

    fn validate_invocation_scalars(
        invocation: &mut PolicyScanV1Invocation,
    ) -> Result<(), PolicyScanV1Error> {
        if !matches!(invocation.source_language.as_str(), "go" | "rust") {
            return Err(cli_error(
                "POLICY_CLI_SCALAR",
                "source language is not go or rust",
            ));
        }
        if !profile_id(&invocation.semantic_profile)
            || !profile_id(&invocation.registry_id)
            || !profile_id(&invocation.frontend_bundle_id)
            || !profile_id(&invocation.toolchain_bundle_id)
            || !public_identity(&invocation.package)
            || !public_identity(&invocation.function)
            || !lower_sha256(&invocation.registry_sha256)
        {
            return Err(cli_error(
                "POLICY_CLI_SCALAR",
                "one or more scan selection values are malformed",
            ));
        }
        match invocation.source_language.as_str() {
            "go" if !valid_go_target(&invocation.target_id)
                || !valid_go_selection(&invocation.package, &invocation.function) =>
            {
                return Err(cli_error(
                    "POLICY_CLI_SCALAR",
                    "Go target, package, or function identity is not canonical",
                ));
            }
            "rust"
                if !valid_rust_target(&invocation.target_id)
                    || !valid_rust_selection(&invocation.function) =>
            {
                return Err(cli_error(
                    "POLICY_CLI_SCALAR",
                    "Rust target or function identity is not canonical",
                ));
            }
            _ => {}
        }
        let mut folded = BTreeSet::new();
        if invocation.contracts.len() > 128 {
            return Err(cli_error(
                "POLICY_CLI_SCALAR",
                "contract count exceeds the registered frontend profile",
            ));
        }
        for contract in &invocation.contracts {
            if mpk_vc::validate_manifest_normalized_path(contract).is_err()
                || !folded.insert(contract.to_ascii_lowercase())
            {
                return Err(cli_error(
                    "POLICY_CLI_SCALAR",
                    "contract paths must be portable and case-fold unique",
                ));
            }
        }
        if mpk_vc::validate_manifest_normalized_path(&invocation.json_out).is_err() {
            return Err(cli_error(
                "POLICY_CLI_SCALAR",
                "json output must be a normalized relative path",
            ));
        }
        invocation
            .contracts
            .sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        Ok(())
    }

    pub(crate) fn validate_policy_scan_v1_profile(
        invocation: &PolicyScanV1Invocation,
    ) -> Result<(), PolicyScanV1Error> {
        if !matches!(
            invocation.semantic_profile.as_str(),
            "mpk.go.fixed.v0" | "mpk.rust.checked.v0"
        ) {
            return Err(cli_error(
                "POLICY_PROFILE_UNKNOWN",
                "semantic profile is not registered",
            ));
        }
        let expected = match invocation.source_language.as_str() {
            "go" => ("mpk.go.fixed.v0", "linux/amd64"),
            "rust" => ("mpk.rust.checked.v0", "x86_64-unknown-linux-gnu"),
            _ => unreachable!("source language was validated first"),
        };
        if (
            invocation.semantic_profile.as_str(),
            invocation.target_id.as_str(),
        ) != expected
        {
            return Err(cli_error(
                "POLICY_PROFILE_TUPLE",
                "language, semantic profile, and target form a crossed tuple",
            ));
        }
        Ok(())
    }

    fn profile_id(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 128
            && value
                .split(['.', '_', '-'])
                .all(|segment| !segment.is_empty() && segment.bytes().all(is_lower_id_byte))
    }

    fn is_lower_id_byte(byte: u8) -> bool {
        byte.is_ascii_lowercase() || byte.is_ascii_digit()
    }

    fn lower_sha256(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    fn valid_go_target(value: &str) -> bool {
        let mut segments = value.split('/');
        let valid = |segment: &str| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        };
        segments.next().is_some_and(valid)
            && segments.next().is_some_and(valid)
            && segments.next().is_none()
    }

    fn valid_rust_target(value: &str) -> bool {
        let bytes = value.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= 255
            && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
            && (bytes[bytes.len() - 1].is_ascii_lowercase()
                || bytes[bytes.len() - 1].is_ascii_digit())
            && bytes.iter().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'.' | b'-')
            })
    }

    fn public_identity(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 1_024
            && !value.chars().any(char::is_control)
            && !value.chars().any(char::is_whitespace)
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(byte, b'.' | b'_' | b'~' | b':' | b'#' | b'/' | b'-')
            })
            && !value.starts_with('/')
            && !value.contains("://")
    }

    fn valid_go_selection(package: &str, function: &str) -> bool {
        let valid_segment = |segment: &str| {
            let bytes = segment.as_bytes();
            !bytes.is_empty()
                && !matches!(segment, "." | "..")
                && (bytes[0].is_ascii_alphanumeric() || bytes[0] == b'_')
                && bytes.iter().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-')
                })
        };
        if matches!(package, "main" | "all" | "std" | "cmd")
            || package.contains("...")
            || !package.split('/').all(valid_segment)
        {
            return false;
        }
        let Some(items) = function
            .strip_prefix(package)
            .and_then(|suffix| suffix.strip_prefix('.'))
        else {
            return false;
        };
        let items = items.split('.').collect::<Vec<_>>();
        (1..=2).contains(&items.len()) && items.iter().all(|item| valid_ascii_identifier(item, 255))
    }

    fn valid_rust_selection(function: &str) -> bool {
        let mut segments = function.split("::");
        segments
            .next()
            .is_some_and(|segment| valid_ascii_identifier(segment, 255))
            && segments.clone().next().is_some()
            && segments.all(|segment| valid_ascii_identifier(segment, 255))
    }

    fn valid_ascii_identifier(value: &str, maximum: usize) -> bool {
        let bytes = value.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= maximum
            && (bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
            && value != "_"
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    }

    fn cli_error(code: &'static str, detail: impl Into<String>) -> PolicyScanV1Error {
        PolicyScanV1Error::new(code, detail)
    }

    pub(crate) fn run_policy_scan_v1(
        argv: &[String],
        working_directory: &Path,
        captured_inputs: Vec<OwnedCapturedInput>,
    ) -> Result<Option<PolicyScanV1RunOutput>, PolicyScanV1Error> {
        run_policy_scan_v1_with(
            argv,
            working_directory,
            captured_inputs,
            |release| {
                prepare_installed_frontend(release).map_err(|error| {
                    PolicyScanV1Error::new(
                        error.code().as_str(),
                        "generic frontend release preflight failed",
                    )
                })
            },
            |prepared, request| {
                run_prepared_frontend(prepared, request).map_err(|error| {
                    PolicyScanV1Error::new(error.code().as_str(), "generic frontend runner failed")
                })
            },
        )
    }

    pub(crate) fn run_policy_scan_v1_with<P, F, R>(
        argv: &[String],
        working_directory: &Path,
        captured_inputs: Vec<OwnedCapturedInput>,
        mut prepare: F,
        mut runner: R,
    ) -> Result<Option<PolicyScanV1RunOutput>, PolicyScanV1Error>
    where
        F: FnMut(&ReleaseSelectionRequest) -> Result<P, PolicyScanV1Error>,
        R: for<'a> FnMut(
            P,
            FrontendRunRequest<'a>,
        ) -> Result<AcceptedFrontendRun, PolicyScanV1Error>,
    {
        let Some(invocation) = parse_policy_scan_v1_argv(argv)? else {
            return Ok(None);
        };
        let prepared = prepare(&invocation.release_request())?;
        let output_target = preflight_scan_output(working_directory, &invocation.json_out)?;
        validate_owned_captured_inputs(&captured_inputs)?;
        let policy_parameters = invocation.semantic_parameters();
        let policy_selection = invocation.selection();
        let semantic_parameters = serde_json::to_value(&policy_parameters)
            .map_err(|error| internal_linkage(error.to_string()))?;
        let selection = serde_json::to_value(&policy_selection)
            .map_err(|error| internal_linkage(error.to_string()))?;
        let captured_refs = captured_inputs
            .iter()
            .map(OwnedCapturedInput::as_ref)
            .collect::<Vec<_>>();
        let frontend = runner(
            prepared,
            FrontendRunRequest {
                release: invocation.release_request(),
                semantic_parameters: &semantic_parameters,
                selection: &selection,
                captured_inputs: &captured_refs,
                contracts: &invocation.contracts,
            },
        )?;
        let output = build_policy_scan_v1_output(invocation, frontend, captured_inputs)?;
        safe_create_scan(&output_target, output.scan.canonical_bytes())?;
        Ok(Some(output))
    }

    /// Builds the exact validated scan projection from a retained frontend run
    /// without writing a scan artifact. The staged v1 verify path uses this
    /// after its single frontend launch so it cannot drift into a hidden scan.
    pub(crate) fn build_policy_scan_v1_output(
        invocation: PolicyScanV1Invocation,
        frontend: AcceptedFrontendRun,
        captured_inputs: Vec<OwnedCapturedInput>,
    ) -> Result<PolicyScanV1RunOutput, PolicyScanV1Error> {
        validate_owned_captured_inputs(&captured_inputs)?;
        validate_runner_selection(&invocation, &frontend.release)?;
        let policy_parameters = invocation.semantic_parameters();
        let policy_selection = invocation.selection();
        let context = scan_linkage_context(
            &invocation,
            &policy_parameters,
            &policy_selection,
            &frontend,
            &captured_inputs,
        )?;
        let document = scan_document(&context)?;
        let canonical = canonical_policy_scan_v1_json(&document)
            .map_err(|error| PolicyScanV1Error::new(error.code(), error.to_string()))?;
        let scan = import_policy_scan_v1_json(&canonical, &context)
            .map_err(|error| PolicyScanV1Error::new(error.code(), error.to_string()))?;
        Ok(PolicyScanV1RunOutput {
            invocation,
            scan,
            frontend,
            captured_inputs,
        })
    }

    pub(crate) fn validate_owned_captured_inputs(
        inputs: &[OwnedCapturedInput],
    ) -> Result<(), PolicyScanV1Error> {
        let mut paths = BTreeSet::new();
        let mut folded = BTreeSet::new();
        for input in inputs {
            if mpk_vc::validate_manifest_normalized_path(&input.normalized_path).is_err()
                || !paths.insert(input.normalized_path.as_str())
                || !folded.insert(input.normalized_path.to_ascii_lowercase())
            {
                return Err(cli_error(
                    "POLICY_CLI_SCALAR",
                    "captured inputs are not portable and unique",
                ));
            }
        }
        Ok(())
    }

    fn validate_runner_selection(
        invocation: &PolicyScanV1Invocation,
        release: &FrontendReleaseIdentity,
    ) -> Result<(), PolicyScanV1Error> {
        if release.release_registry.schema != "mpk.release.bundle_registry.v0"
            || release.release_registry.id != invocation.registry_id
            || release.release_registry.registry_sha256 != invocation.registry_sha256
            || release.frontend.bundle_id != invocation.frontend_bundle_id
            || release.toolchain.bundle_id != invocation.toolchain_bundle_id
            || release.limit_profile != "mpk.vir.limits.v0"
        {
            return Err(PolicyScanV1Error::new(
                "POLICY_RELEASE_LINKAGE",
                "runner returned a release other than the selected tuple",
            ));
        }
        Ok(())
    }

    fn scan_linkage_context(
        invocation: &PolicyScanV1Invocation,
        semantic_parameters: &PolicySemanticParameters,
        selection: &PolicySelection,
        frontend: &AcceptedFrontendRun,
        captured_inputs: &[OwnedCapturedInput],
    ) -> Result<PolicyScanLinkageContext, PolicyScanV1Error> {
        let rejected_features = issues(&frontend.envelope.value, "rejected_features")?;
        let diagnostics = issues(&frontend.envelope.value, "diagnostics")?;
        let success = frontend.envelope.status == "ir-lowered";
        let artifacts = match (success, frontend.envelope.artifacts.as_ref()) {
            (true, Some(artifacts)) => Some(artifacts),
            (false, None) => None,
            _ => return Err(internal_linkage("frontend artifact branch is inconsistent")),
        };
        let (
            limit_profile,
            frontend_manifest_hash,
            input_set_hash,
            source_map_hash,
            source_ir_schema,
            source_ir_hash,
            helper_artifacts,
        ) = if let Some(artifacts) = artifacts {
            let manifest = artifacts.source_manifest.manifest();
            if manifest.limit_profile != frontend.release.limit_profile {
                return Err(internal_linkage(
                    "successful manifest limit profile differs from the release tuple",
                ));
            }
            let manifest_contracts = manifest
                .inputs
                .iter()
                .filter(|input| input.kind == InputKind::Contract)
                .map(|input| input.normalized_path.as_str())
                .collect::<Vec<_>>();
            if manifest_contracts
                != invocation
                    .contracts
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            {
                return Err(internal_linkage(
                    "successful manifest contract set differs from the invocation",
                ));
            }
            (
                Some(manifest.limit_profile.clone()),
                Some(artifacts.source_manifest.hash().as_str().to_owned()),
                Some(manifest.input_set_hash.clone()),
                Some(artifacts.source_map.hash().as_str().to_owned()),
                Some(artifacts.vir.schema.clone()),
                Some(artifacts.vir.vir_hash.as_str().to_owned()),
                Some(success_helpers(artifacts, captured_inputs)?),
            )
        } else {
            (None, None, None, None, None, None, None)
        };
        Ok(PolicyScanLinkageContext {
            frontend_status: frontend.envelope.status.clone(),
            frontend_phase: frontend.envelope.phase.clone(),
            source_language: invocation.source_language.clone(),
            semantic_profile: invocation.semantic_profile.clone(),
            semantic_parameters: semantic_parameters.clone(),
            selection: selection.clone(),
            release_registry: frontend.release.release_registry.clone(),
            frontend: frontend.release.frontend.clone(),
            toolchain: frontend.release.toolchain.clone(),
            rejected_features,
            diagnostics,
            limit_profile,
            frontend_source_manifest_hash: frontend_manifest_hash,
            input_set_hash,
            source_map_hash,
            source_ir_schema,
            source_ir_hash,
            helper_artifacts,
        })
    }

    fn issues(value: &Value, field: &str) -> Result<Vec<PolicyIssue>, PolicyScanV1Error> {
        serde_json::from_value(
            value
                .get(field)
                .cloned()
                .ok_or_else(|| internal_linkage(format!("validated envelope omitted {field}")))?,
        )
        .map_err(|error| internal_linkage(error.to_string()))
    }

    fn success_helpers(
        artifacts: &AcceptedFrontendArtifacts,
        captured_inputs: &[OwnedCapturedInput],
    ) -> Result<Vec<PolicyHelperArtifact>, PolicyScanV1Error> {
        let manifest = artifacts.source_manifest.manifest();
        let mut helpers = manifest
            .inputs
            .iter()
            .filter(|input| input.kind == InputKind::Source)
            .map(|input| PolicyHelperArtifact::Source {
                id: format!("source:{}", input.normalized_path),
                normalized_path: input.normalized_path.clone(),
                sha256: input.sha256.clone(),
            })
            .collect::<Vec<_>>();
        for input in manifest
            .inputs
            .iter()
            .filter(|input| input.kind == InputKind::Contract)
        {
            let captured = captured_inputs
                .iter()
                .find(|captured| {
                    captured.kind == InputKind::Contract
                        && captured.normalized_path == input.normalized_path
                })
                .ok_or_else(|| internal_linkage("manifest contract bytes are not retained"))?;
            let (schema, raw_function) = contract_identity(&captured.bytes)?;
            let function = resolve_contract_function(&artifacts.vir, &raw_function)?;
            helpers.push(PolicyHelperArtifact::Contract {
                id: format!("contract:{}", function.id),
                normalized_path: input.normalized_path.clone(),
                schema,
                raw_input_sha256: input.sha256.clone(),
                function_id: function.id.clone(),
                contract_hash: function.contracts.contract_hash.as_str().to_owned(),
            });
        }
        helpers.push(PolicyHelperArtifact::VerificationIr {
            id: "verification_ir".to_owned(),
            schema: artifacts.vir.schema.clone(),
            sha256: artifacts.vir.vir_hash.as_str().to_owned(),
        });
        helpers.sort_by(|left, right| {
            (helper_rank(left), left.id().as_bytes())
                .cmp(&(helper_rank(right), right.id().as_bytes()))
        });
        Ok(helpers)
    }

    fn helper_rank(helper: &PolicyHelperArtifact) -> u8 {
        match helper {
            PolicyHelperArtifact::Source { .. } => 0,
            PolicyHelperArtifact::Contract { .. } => 1,
            PolicyHelperArtifact::VerificationIr { .. } => 2,
            PolicyHelperArtifact::Vc { .. } => 3,
            PolicyHelperArtifact::AiAnalysis { .. } => 4,
            PolicyHelperArtifact::CiStatus { .. } => 5,
        }
    }

    fn contract_identity(bytes: &[u8]) -> Result<(String, String), PolicyScanV1Error> {
        let strict = parse_strict_json(
            bytes,
            StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576),
        )
        .map_err(|error| internal_linkage(format!("validated contract: {error}")))?;
        let canonical = canonical_json_bytes(&strict)
            .map_err(|error| internal_linkage(format!("validated contract: {error}")))?;
        let value: Value = serde_json::from_slice(&canonical)
            .map_err(|error| internal_linkage(format!("validated contract: {error}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| internal_linkage("validated contract is not an object"))?;
        let schema = object
            .get("schema")
            .and_then(Value::as_str)
            .ok_or_else(|| internal_linkage("validated contract schema is absent"))?;
        let function = object
            .get("function")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|function| !function.is_empty())
            .ok_or_else(|| internal_linkage("validated contract function is absent"))?;
        Ok((schema.to_owned(), function.to_owned()))
    }

    fn resolve_contract_function<'a>(
        vir: &'a mpk_vc::VirModule,
        raw: &str,
    ) -> Result<&'a mpk_vc::VirFunction, PolicyScanV1Error> {
        let mut matches = vir
            .units
            .iter()
            .flat_map(|unit| &unit.functions)
            .filter(|function| {
                function.id == raw
                    || function
                        .id
                        .rsplit_once('/')
                        .is_some_and(|(_, suffix)| suffix == raw)
                    || function.id.ends_with(&format!(".{raw}"))
            });
        let function = matches
            .next()
            .ok_or_else(|| internal_linkage("contract function is absent from validated VIR"))?;
        if matches.next().is_some() {
            return Err(internal_linkage(
                "contract function is ambiguous in validated VIR",
            ));
        }
        Ok(function)
    }

    fn scan_document(
        context: &PolicyScanLinkageContext,
    ) -> Result<PolicyScanV1, PolicyScanV1Error> {
        let readiness = match context.frontend_status.as_str() {
            "ir-lowered" => "ready",
            "rejected" => "unsupported",
            "source-error" => "source_error",
            "frontend-error" => "frontend_error",
            _ => return Err(internal_linkage("validated frontend status is unknown")),
        };
        Ok(PolicyScanV1 {
            schema: POLICY_SCAN_V1_SCHEMA.to_owned(),
            frontend_status: context.frontend_status.clone(),
            frontend_phase: context.frontend_phase.clone(),
            source_language: context.source_language.clone(),
            semantic_profile: context.semantic_profile.clone(),
            semantic_parameters: context.semantic_parameters.clone(),
            selection: context.selection.clone(),
            release_registry: context.release_registry.clone(),
            frontend: context.frontend.clone(),
            toolchain: context.toolchain.clone(),
            readiness: readiness.to_owned(),
            rejected_features: context.rejected_features.clone(),
            diagnostics: context.diagnostics.clone(),
            limit_profile: context.limit_profile.clone(),
            frontend_source_manifest_hash: context.frontend_source_manifest_hash.clone(),
            input_set_hash: context.input_set_hash.clone(),
            source_map_hash: context.source_map_hash.clone(),
            source_ir_schema: context.source_ir_schema.clone(),
            source_ir_hash: context.source_ir_hash.clone(),
            helper_artifacts: context.helper_artifacts.clone(),
        })
    }

    fn internal_linkage(detail: impl Into<String>) -> PolicyScanV1Error {
        PolicyScanV1Error::new("POLICY_SOURCE_LINKAGE", detail)
    }

    struct ScanOutputTarget {
        path: PathBuf,
        parent: PathBuf,
        parent_identity: DirectoryIdentity,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct DirectoryIdentity {
        canonical_path: PathBuf,
        #[cfg(unix)]
        device: u64,
        #[cfg(unix)]
        inode: u64,
    }

    fn preflight_scan_output(
        working_directory: &Path,
        relative: &str,
    ) -> Result<ScanOutputTarget, PolicyScanV1Error> {
        let root_metadata = fs::symlink_metadata(working_directory)
            .map_err(|error| output_error(format!("working directory: {error}")))?;
        if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
            return Err(output_error(
                "working directory is not a retained directory",
            ));
        }
        let relative_path = Path::new(relative);
        if relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(output_error("output path is not normalized relative"));
        }
        let output = working_directory.join(relative_path);
        let parent = output
            .parent()
            .ok_or_else(|| output_error("output parent is absent"))?
            .to_path_buf();
        let mut current = working_directory.to_path_buf();
        if let Some(parent_relative) = relative_path.parent() {
            for component in parent_relative.components() {
                let Component::Normal(component) = component else {
                    return Err(output_error("output parent is not normalized"));
                };
                current.push(component);
                let metadata = fs::symlink_metadata(&current)
                    .map_err(|error| output_error(format!("output parent: {error}")))?;
                if !metadata.is_dir() || metadata.file_type().is_symlink() {
                    return Err(output_error("output parent is not a regular directory"));
                }
            }
        }
        if current != parent {
            return Err(output_error("output parent escaped the retained root"));
        }
        let root_identity = directory_identity(working_directory)?;
        let parent_identity = directory_identity(&parent)?;
        if !parent_identity
            .canonical_path
            .starts_with(&root_identity.canonical_path)
        {
            return Err(output_error("output parent escaped the retained root"));
        }
        match fs::symlink_metadata(&output) {
            Ok(_) => Err(output_error("scan output already exists")),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ScanOutputTarget {
                path: output,
                parent,
                parent_identity,
            }),
            Err(error) => Err(output_error(format!("inspect scan output: {error}"))),
        }
    }

    fn directory_identity(path: &Path) -> Result<DirectoryIdentity, PolicyScanV1Error> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| output_error(format!("inspect output directory: {error}")))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(output_error("output directory identity is not retained"));
        }
        let canonical_path = fs::canonicalize(path)
            .map_err(|error| output_error(format!("resolve output directory: {error}")))?;
        Ok(DirectoryIdentity {
            canonical_path,
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
        })
    }

    fn revalidate_output_parent(target: &ScanOutputTarget) -> Result<(), PolicyScanV1Error> {
        if directory_identity(&target.parent)? != target.parent_identity {
            return Err(output_error("output parent changed after preflight"));
        }
        Ok(())
    }

    fn safe_create_scan(target: &ScanOutputTarget, bytes: &[u8]) -> Result<(), PolicyScanV1Error> {
        revalidate_output_parent(target)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&target.parent)
            .map_err(|error| output_error(format!("create scan temporary: {error}")))?;
        revalidate_output_parent(target)?;
        temporary
            .write_all(bytes)
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| output_error(format!("write scan temporary: {error}")))?;
        let published_identity = temporary
            .as_file()
            .metadata()
            .map_err(|error| output_error(format!("inspect scan temporary: {error}")))?;
        revalidate_output_parent(target)?;
        let persisted = temporary
            .persist_noclobber(&target.path)
            .map_err(|error| output_error(format!("publish scan output: {}", error.error)))?;
        if let Err(error) = revalidate_published_scan(target, &published_identity).and_then(|()| {
            OpenOptions::new()
                .read(true)
                .open(&target.parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| {
                    output_error(format!("synchronize scan output directory: {error}"))
                })
        }) {
            rollback_published_scan(target, &published_identity).map_err(|rollback| {
                output_error(format!("{error}; output recovery required: {rollback}"))
            })?;
            return Err(error);
        }
        drop(persisted);
        Ok(())
    }

    fn revalidate_published_scan(
        target: &ScanOutputTarget,
        expected: &fs::Metadata,
    ) -> Result<(), PolicyScanV1Error> {
        revalidate_output_parent(target)?;
        let actual = fs::symlink_metadata(&target.path)
            .map_err(|error| output_error(format!("inspect published scan: {error}")))?;
        if !actual.is_file() || actual.file_type().is_symlink() || !same_file(&actual, expected) {
            return Err(output_error("published scan identity changed"));
        }
        #[cfg(unix)]
        if actual.nlink() != 1 {
            return Err(output_error("published scan has a hard-link alias"));
        }
        Ok(())
    }

    fn rollback_published_scan(
        target: &ScanOutputTarget,
        expected: &fs::Metadata,
    ) -> Result<(), PolicyScanV1Error> {
        let actual = fs::symlink_metadata(&target.path)
            .map_err(|error| output_error(format!("inspect scan during rollback: {error}")))?;
        if !actual.is_file() || actual.file_type().is_symlink() || !same_file(&actual, expected) {
            return Err(output_error("scan identity changed before rollback"));
        }
        fs::remove_file(&target.path)
            .map_err(|error| output_error(format!("remove scan during rollback: {error}")))?;
        OpenOptions::new()
            .read(true)
            .open(&target.parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| output_error(format!("synchronize scan rollback: {error}")))
    }

    #[cfg(unix)]
    fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
        left.dev() == right.dev() && left.ino() == right.ino()
    }

    #[cfg(not(unix))]
    fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
        left.is_file() && right.is_file() && left.len() == right.len()
    }

    fn output_error(detail: impl Into<String>) -> PolicyScanV1Error {
        PolicyScanV1Error::new("POLICY_CLI_OUTPUT", detail)
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use super::*;
        use crate::frontend_protocol::{
            validate_frontend_process, FrontendProcessFacts, FrontendProtocolRequest,
        };
        use mpk_vc::{
            validate_release_registry, FrontendIdentity, ReleaseRegistryIdentity, ToolchainIdentity,
        };
        use std::cell::Cell;

        #[test]
        fn policy_scan_v1_parser_help_is_side_effect_free() {
            for help in ["help", "-h", "--help"] {
                let argv = ["mpk", "policy", "scan", help]
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                assert_eq!(parse_policy_scan_v1_argv(&argv).unwrap(), None);
            }

            let mut argv = go_scan_argv();
            argv[3] = "-fixture".to_owned();
            assert_eq!(
                parse_policy_scan_v1_argv(&argv)
                    .unwrap()
                    .unwrap()
                    .source_root,
                "-fixture"
            );
        }

        #[test]
        fn policy_scan_v1_parser_normalizes_contract_order_and_forbids_old_locators() {
            let mut argv = go_scan_argv();
            replace_option(&mut argv, "--contract", "contracts/z.json");
            argv.extend(["--contract".to_owned(), "contracts/a.json".to_owned()]);
            let parsed = parse_policy_scan_v1_argv(&argv).unwrap().unwrap();
            assert_eq!(parsed.contracts, ["contracts/a.json", "contracts/z.json"]);

            for option in FORBIDDEN_LOCATORS {
                let mut forbidden = go_scan_argv();
                forbidden.extend([option.to_owned(), "/tmp/raw-locator".to_owned()]);
                assert_eq!(
                    parse_policy_scan_v1_argv(&forbidden).unwrap_err().code(),
                    "POLICY_CLI_FORBIDDEN_LOCATOR",
                    "{option}"
                );
            }
        }

        #[test]
        fn policy_scan_v1_parser_rejects_unknown_and_crossed_profiles_before_release_selection() {
            let mut argv = go_scan_argv();
            replace_option(&mut argv, "--semantic-profile", "mpk.future.fixed.v0");
            assert_eq!(
                parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
                "POLICY_PROFILE_UNKNOWN"
            );
            replace_option(&mut argv, "--semantic-profile", "mpk.rust.checked.v0");
            assert_eq!(
                parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
                "POLICY_PROFILE_TUPLE"
            );
        }

        #[test]
        fn policy_scan_v1_release_preflight_precedes_output_and_launch() {
            let temporary = tempfile::tempdir().unwrap();
            fs::create_dir(temporary.path().join("out")).unwrap();
            fs::write(temporary.path().join("out/scan.json"), b"old").unwrap();
            let launches = Cell::new(0);
            let error = run_policy_scan_v1_with(
                &go_scan_argv(),
                temporary.path(),
                Vec::new(),
                |_| {
                    Err::<(), _>(PolicyScanV1Error::new(
                        "FRONTEND_BUNDLE_UNKNOWN",
                        "unknown preflight bundle",
                    ))
                },
                |(), _| {
                    launches.set(launches.get() + 1);
                    unreachable!("release preflight failure must prevent launch")
                },
            )
            .unwrap_err();
            assert_eq!(error.code(), "FRONTEND_BUNDLE_UNKNOWN");
            assert_eq!(launches.get(), 0);
            assert_eq!(
                fs::read(temporary.path().join("out/scan.json")).unwrap(),
                b"old"
            );
        }

        #[test]
        fn policy_scan_v1_executes_every_normative_scan_cli_case() {
            let recipes: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/policy-recipes-v1.json"
            ))
            .unwrap();
            let registry = synthetic_registry();
            let invocations = recipes["invocations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|invocation| {
                    (
                        invocation["id"].as_str().unwrap().to_owned(),
                        invocation.clone(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let mut executed = 0;
            for case in recipes["cli_cases"].as_array().unwrap() {
                let base = case
                    .get("input_from")
                    .and_then(Value::as_str)
                    .or_else(|| case.pointer("/construction/base").and_then(Value::as_str));
                if base != Some("invocation.go_scan") {
                    continue;
                }
                executed += 1;
                let mut argv = invocations["invocation.go_scan"]["argv"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                if let Some(operations) = case
                    .pointer("/construction/operations")
                    .and_then(Value::as_array)
                {
                    for operation in operations {
                        apply_cli_operation(&mut argv, operation);
                    }
                }

                let mut launch_count = 0;
                let result = parse_policy_scan_v1_argv(&argv).and_then(|parsed| {
                    let parsed = parsed.ok_or_else(|| {
                        PolicyScanV1Error::new(
                            "POLICY_CLI_ARGUMENT",
                            "vector invocation unexpectedly selected help",
                        )
                    })?;
                    if parsed.frontend_bundle_id == "frontend.rust.synthetic.v0"
                        || parsed.toolchain_bundle_id == "toolchain.rust.synthetic.v0"
                    {
                        return Err(PolicyScanV1Error::new(
                            "FRONTEND_BUNDLE_INCOMPATIBLE",
                            "known synthetic bundle belongs to the crossed language tuple",
                        ));
                    }
                    registry
                        .resolve(&parsed.release_request())
                        .map_err(|error| {
                            PolicyScanV1Error::new(error.code(), "synthetic release selection")
                        })?;
                    launch_count = 1;
                    Ok(parsed)
                });
                let expected = &case["expect"];
                assert_eq!(
                    launch_count,
                    expected["launch_count"].as_i64().unwrap(),
                    "{}",
                    case["id"]
                );
                match expected["outcome"].as_str().unwrap() {
                    "accept" => {
                        let parsed = result.unwrap_or_else(|error| {
                            panic!("{} unexpectedly rejected: {error}", case["id"])
                        });
                        assert_eq!(
                            serde_json::to_value(parsed).unwrap(),
                            invocations["invocation.go_scan"]["parsed"],
                            "{}",
                            case["id"]
                        );
                    }
                    "reject" => assert_eq!(
                        result.unwrap_err().code(),
                        expected["code"].as_str().unwrap(),
                        "{}",
                        case["id"]
                    ),
                    outcome => panic!("unknown vector outcome {outcome}"),
                }
            }
            assert!(executed > 30, "the scan CLI vector subset was not executed");
        }

        #[test]
        fn policy_scan_v1_ready_pipeline_is_single_launch_canonical_and_safe_write() {
            let temporary = tempfile::tempdir().unwrap();
            fs::create_dir(temporary.path().join("out")).unwrap();
            let captured = go_identity_inputs();
            let accepted = successful_frontend_run(&captured);
            let launches = Cell::new(0);
            let argv = go_scan_argv();
            let first = run_policy_scan_v1_with(
                &argv,
                temporary.path(),
                captured.clone(),
                |_| Ok(()),
                |(), _| {
                    launches.set(launches.get() + 1);
                    Ok(accepted.clone())
                },
            )
            .unwrap()
            .unwrap();
            assert_eq!(launches.get(), 1);
            assert_eq!(first.scan.document().readiness, "ready");
            assert_eq!(
                first
                    .scan
                    .document()
                    .helper_artifacts
                    .as_ref()
                    .unwrap()
                    .len(),
                3
            );
            let first_bytes = fs::read(temporary.path().join("out/scan.json")).unwrap();
            assert_eq!(first_bytes, first.scan.canonical_bytes());
            assert!(first_bytes.ends_with(b"\n"));
            let document_value: Value = serde_json::from_slice(&first_bytes).unwrap();
            for forbidden in [
                "strategy_profile",
                "checker_profile",
                "axiom_profile",
                "proof_accepted",
            ] {
                assert!(document_value.get(forbidden).is_none());
            }

            let existing = run_policy_scan_v1_with(
                &argv,
                temporary.path(),
                captured.clone(),
                |_| Ok(()),
                |(), _| {
                    launches.set(launches.get() + 1);
                    Ok(accepted.clone())
                },
            )
            .unwrap_err();
            assert_eq!(existing.code(), "POLICY_CLI_OUTPUT");
            assert_eq!(launches.get(), 1, "existing output rejects before launch");

            let mut second_argv = argv;
            replace_option(&mut second_argv, "--json-out", "out/scan-2.json");
            let second = run_policy_scan_v1_with(
                &second_argv,
                temporary.path(),
                captured,
                |_| Ok(()),
                |(), _| {
                    launches.set(launches.get() + 1);
                    Ok(accepted.clone())
                },
            )
            .unwrap()
            .unwrap();
            assert_eq!(launches.get(), 2);
            assert_eq!(first.scan.canonical_bytes(), second.scan.canonical_bytes());
            assert_eq!(
                first_bytes,
                fs::read(temporary.path().join("out/scan-2.json")).unwrap()
            );
        }

        #[test]
        fn policy_scan_v1_rejects_directory_output_before_launch() {
            let temporary = tempfile::tempdir().unwrap();
            fs::create_dir_all(temporary.path().join("out/scan.json")).unwrap();
            let launches = Cell::new(0);
            let error = run_policy_scan_v1_with(
                &go_scan_argv(),
                temporary.path(),
                go_identity_inputs(),
                |_| Ok(()),
                |(), _| {
                    launches.set(launches.get() + 1);
                    unreachable!("directory output must reject before launch")
                },
            )
            .unwrap_err();
            assert_eq!(error.code(), "POLICY_CLI_OUTPUT");
            assert_eq!(launches.get(), 0);
            assert!(temporary.path().join("out/scan.json").is_dir());
        }

        #[test]
        fn policy_scan_v1_malformed_or_partial_success_cannot_become_ready() {
            let temporary = tempfile::tempdir().unwrap();
            fs::create_dir(temporary.path().join("out")).unwrap();
            let captured = go_identity_inputs();
            let mut partial = successful_frontend_run(&captured);
            partial.envelope.artifacts = None;
            let error = run_policy_scan_v1_with(
                &go_scan_argv(),
                temporary.path(),
                captured,
                |_| Ok(()),
                |(), _| Ok(partial.clone()),
            )
            .unwrap_err();
            assert_eq!(error.code(), "POLICY_SOURCE_LINKAGE");
            assert!(!temporary.path().join("out/scan.json").exists());

            let malformed = br#"{"schema":"mpk.frontend.cli.v0","status":"ir-lowered"}\n"#;
            let parameters = json!({"target_id":"linux/amd64","pointer_width":64});
            let selection = json!({
                "package":"example.com/mpk/vector",
                "function":"example.com/mpk/vector.Identity"
            });
            assert!(validate_frontend_process(
                FrontendProtocolRequest {
                    source_language: "go",
                    semantic_profile: "mpk.go.fixed.v0",
                    semantic_parameters: &parameters,
                    selection: &selection,
                    release_registry: None,
                    captured_inputs: &[],
                },
                FrontendProcessFacts {
                    exit_code: Some(0),
                    signaled: false,
                    stdout: malformed,
                    stderr_observed_bytes: 0,
                },
            )
            .is_err());
        }

        #[test]
        fn policy_scan_v1_non_success_statuses_map_to_deterministic_readiness() {
            for (status, phase, exit, readiness) in [
                ("rejected", "subset", 3, "unsupported"),
                ("source-error", "source", 4, "source_error"),
                ("frontend-error", "capture", 1, "frontend_error"),
            ] {
                let temporary = tempfile::tempdir().unwrap();
                fs::create_dir(temporary.path().join("out")).unwrap();
                let accepted = non_success_frontend_run(status, phase, exit);
                let output = run_policy_scan_v1_with(
                    &go_scan_argv(),
                    temporary.path(),
                    Vec::new(),
                    |_| Ok(()),
                    |(), _| Ok(accepted.clone()),
                )
                .unwrap()
                .unwrap();
                assert_eq!(output.scan.document().readiness, readiness);
                assert!(output.scan.document().helper_artifacts.is_none());
            }
        }

        pub(crate) fn go_scan_argv() -> Vec<String> {
            vec![
                "mpk",
                "policy",
                "scan",
                "examples/go-policy",
                "--language",
                "go",
                "--semantic-profile",
                "mpk.go.fixed.v0",
                "--require-release-registry-id",
                "mpk.release.registry.v0",
                "--require-release-registry-sha256",
                "47f80ab09e8cde24af73ddc198aef254ff1dbd18c1423a2e7e0ebb69f8c787a7",
                "--frontend-bundle",
                "frontend.go.synthetic.v0",
                "--toolchain-bundle",
                "toolchain.go.synthetic.v0",
                "--target",
                "linux/amd64",
                "--package",
                "example.com/mpk/vector",
                "--function",
                "example.com/mpk/vector.Identity",
                "--contract",
                "contracts/identity.json",
                "--json-out",
                "out/scan.json",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        }

        fn replace_option(argv: &mut [String], name: &str, value: &str) {
            let position = argv.iter().position(|argument| argument == name).unwrap();
            argv[position + 1] = value.to_owned();
        }

        fn apply_cli_operation(argv: &mut Vec<String>, operation: &Value) {
            let name = operation.get("name").and_then(Value::as_str);
            match operation["op"].as_str().unwrap() {
                "remove_option" => {
                    let position = argv
                        .iter()
                        .position(|value| Some(value.as_str()) == name)
                        .unwrap();
                    argv.drain(position..=position + 1);
                }
                "remove_all_options" => {
                    while let Some(position) =
                        argv.iter().position(|value| Some(value.as_str()) == name)
                    {
                        argv.drain(position..=position + 1);
                    }
                }
                "remove_option_value" => {
                    let position = argv
                        .iter()
                        .position(|value| Some(value.as_str()) == name)
                        .unwrap();
                    argv.remove(position + 1);
                }
                "append_option" => argv.extend([
                    name.unwrap().to_owned(),
                    operation["value"].as_str().unwrap().to_owned(),
                ]),
                "replace_option" => {
                    replace_option(argv, name.unwrap(), operation["value"].as_str().unwrap())
                }
                "append_flag" => argv.push(name.unwrap().to_owned()),
                "remove_source_root" => {
                    argv.remove(3);
                }
                "append_positional" => {
                    argv.push(operation["value"].as_str().unwrap().to_owned());
                }
                operation => panic!("unknown CLI vector operation {operation}"),
            }
        }

        pub(crate) fn synthetic_registry() -> mpk_vc::ValidatedReleaseRegistry {
            let vector: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/release-bundles-v0.json"
            ))
            .unwrap();
            let bytes = canonical_transport(&vector["fixtures"]["valid_registry"]);
            validate_release_registry(&bytes).unwrap()
        }

        fn canonical_transport(value: &Value) -> Vec<u8> {
            let raw = serde_json::to_vec(value).unwrap();
            let strict = parse_strict_json(
                &raw,
                StrictJsonLimits::new(268_435_456, 67_108_865, 256, 1_048_576),
            )
            .unwrap();
            let mut bytes = canonical_json_bytes(&strict).unwrap();
            bytes.push(b'\n');
            bytes
        }

        pub(crate) fn go_identity_inputs() -> Vec<OwnedCapturedInput> {
            vec![
                OwnedCapturedInput {
                    kind: InputKind::Contract,
                    normalized_path: "contracts/identity.json".to_owned(),
                    bytes: b"{\"schema\":\"mpk.go.contract.v0\",\"function\":\"example.com/mpk/vector.Identity\",\"requires\":[],\"ensures\":[{\"op\":\"eq\",\"lhs\":{\"result\":0},\"rhs\":{\"var\":\"value\"}}],\"modifies\":[],\"loops\":[]}\n".to_vec(),
                },
                OwnedCapturedInput {
                    kind: InputKind::BuildManifest,
                    normalized_path: "go.mod".to_owned(),
                    bytes: b"module example.com/mpk/vector\n\ngo 1.25\n".to_vec(),
                },
                OwnedCapturedInput {
                    kind: InputKind::Lockfile,
                    normalized_path: "go.sum".to_owned(),
                    bytes: Vec::new(),
                },
                OwnedCapturedInput {
                    kind: InputKind::Source,
                    normalized_path: "identity.go".to_owned(),
                    bytes: b"package vector\n\nfunc Identity(value int8) int8 { return value }\n".to_vec(),
                },
            ]
        }

        pub(crate) fn successful_frontend_run(
            inputs: &[OwnedCapturedInput],
        ) -> AcceptedFrontendRun {
            let vir_vectors: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/vir-v0.json"
            ))
            .unwrap();
            let map_vectors: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/source-map-v0.json"
            ))
            .unwrap();
            let manifest_vectors: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/source-manifest-v0.json"
            ))
            .unwrap();
            let vir = case_input(&vir_vectors, "module_cases", "module.valid_go_identity");
            let source_map = case_input(&map_vectors, "map_cases", "map.valid_go_identity");
            let manifest = case_input(
                &manifest_vectors,
                "manifest_cases",
                "manifest.valid_go_frontend_stage",
            );
            let parameters = json!({"target_id":"linux/amd64","pointer_width":64});
            let selection = json!({
                "package":"example.com/mpk/vector",
                "function":"example.com/mpk/vector.Identity"
            });
            let envelope = json!({
                "schema":"mpk.frontend.cli.v0",
                "status":"ir-lowered",
                "phase":"emission",
                "source_language":"go",
                "semantic_profile":"mpk.go.fixed.v0",
                "semantic_parameters":parameters,
                "selection":selection,
                "ir":{
                    "schema":"mpk.vir.v0",
                    "sha256":vir["vir_hash"],
                    "value":vir,
                },
                "source_manifest":manifest,
                "source_map":source_map,
                "rejected_features":[],
                "diagnostics":[],
            });
            let bytes = canonical_transport(&envelope);
            let registry = synthetic_registry();
            let captured = inputs
                .iter()
                .map(OwnedCapturedInput::as_ref)
                .collect::<Vec<_>>();
            let accepted = validate_frontend_process(
                FrontendProtocolRequest {
                    source_language: "go",
                    semantic_profile: "mpk.go.fixed.v0",
                    semantic_parameters: &parameters,
                    selection: &selection,
                    release_registry: Some(&registry),
                    captured_inputs: &captured,
                },
                FrontendProcessFacts {
                    exit_code: Some(0),
                    signaled: false,
                    stdout: &bytes,
                    stderr_observed_bytes: 0,
                },
            )
            .unwrap();
            AcceptedFrontendRun {
                envelope: accepted,
                release: release_from_manifest(&envelope["source_manifest"]),
                registry,
            }
        }

        pub(crate) fn non_success_frontend_run(
            status: &str,
            phase: &str,
            exit: i32,
        ) -> AcceptedFrontendRun {
            let parameters = json!({"target_id":"linux/amd64","pointer_width":64});
            let selection = json!({
                "package":"example.com/mpk/vector",
                "function":"example.com/mpk/vector.Identity"
            });
            let rejected = if status == "rejected" {
                json!([{
                    "code":"GO_SUBSET_MAP",
                    "message":"map is unsupported",
                    "function_id":"example.com/mpk/vector.Identity"
                }])
            } else {
                json!([])
            };
            let diagnostics = if status == "rejected" {
                json!([])
            } else {
                json!([{"code":"GO_SOURCE_PARSE","message":"source failed"}])
            };
            let envelope = json!({
                "schema":"mpk.frontend.cli.v0",
                "status":status,
                "phase":phase,
                "source_language":"go",
                "semantic_profile":"mpk.go.fixed.v0",
                "semantic_parameters":parameters,
                "selection":selection,
                "rejected_features":rejected,
                "diagnostics":diagnostics,
            });
            let bytes = canonical_transport(&envelope);
            let accepted = validate_frontend_process(
                FrontendProtocolRequest {
                    source_language: "go",
                    semantic_profile: "mpk.go.fixed.v0",
                    semantic_parameters: &parameters,
                    selection: &selection,
                    release_registry: None,
                    captured_inputs: &[],
                },
                FrontendProcessFacts {
                    exit_code: Some(exit),
                    signaled: false,
                    stdout: &bytes,
                    stderr_observed_bytes: 0,
                },
            )
            .unwrap();
            let scan_vectors: Value = serde_json::from_slice(include_bytes!(
                "../../../develop/specs/vectors/policy-scan-v1.json"
            ))
            .unwrap();
            let context = scan_vectors["linkage_contexts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|context| context["id"] == "context.go_identity_ready")
                .unwrap();
            AcceptedFrontendRun {
                envelope: accepted,
                release: FrontendReleaseIdentity {
                    release_registry: serde_json::from_value(context["release_registry"].clone())
                        .unwrap(),
                    frontend: serde_json::from_value(context["frontend"].clone()).unwrap(),
                    toolchain: serde_json::from_value(context["toolchain"].clone()).unwrap(),
                    limit_profile: "mpk.vir.limits.v0".to_owned(),
                },
                registry: synthetic_registry(),
            }
        }

        fn case_input(vector: &Value, collection: &str, id: &str) -> Value {
            vector[collection]
                .as_array()
                .unwrap()
                .iter()
                .find(|case| case["id"] == id)
                .unwrap()["input"]
                .clone()
        }

        fn release_from_manifest(manifest: &Value) -> FrontendReleaseIdentity {
            FrontendReleaseIdentity {
                release_registry: serde_json::from_value::<ReleaseRegistryIdentity>(
                    manifest["release_registry"].clone(),
                )
                .unwrap(),
                frontend: serde_json::from_value::<FrontendIdentity>(manifest["frontend"].clone())
                    .unwrap(),
                toolchain: serde_json::from_value::<ToolchainIdentity>(
                    manifest["toolchain"].clone(),
                )
                .unwrap(),
                limit_profile: manifest["limit_profile"].as_str().unwrap().to_owned(),
            }
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
