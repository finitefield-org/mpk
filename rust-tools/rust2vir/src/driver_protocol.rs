use crate::diagnostics::{
    normalize_private_diagnostics, valid_private_diagnostic, PrivateDiagnosticStatus,
    DIAGNOSTIC_TRUNCATION_CODE,
};
use crate::json::{self, JsonValue};
use crate::sha256::{hex, Sha256};
use crate::source_capture::{CapturedInput, InputKind};
use std::collections::{BTreeMap, BTreeSet};

pub const REQUEST_PATH: &str = "/mpk/driver-request.json";
pub const OUTPUT_DIRECTORY: &str = "/mpk/driver-output";
pub const OUTPUT_PARTIAL_PATH: &str = "/mpk/driver-output/result.json.partial";
pub const OUTPUT_FINAL_PATH: &str = "/mpk/driver-output/result.json";
pub const REQUEST_TRANSPORT_MAX: usize =
    crate::limits::RustLimitId::PrivateRequestTransport.maximum() as usize;
pub const OUTPUT_TRANSPORT_MAX: usize =
    crate::limits::RustLimitId::PrivateOutputTransport.maximum() as usize;

const REQUEST_DOMAIN: &[u8] = b"MPK-RUST-DRIVER-REQUEST-1.0";
const INVENTORY_DOMAIN: &[u8] = b"MPK-RUST-SOURCE-INVENTORY-0.1";
const INPUT_SET_DOMAIN: &[u8] = b"MPK-INPUT-SET-0.1";
const PAYLOAD_DOMAIN: &[u8] = b"MPK-RUST-DRIVER-PAYLOAD-1.0";
const VIR_DOMAIN: &[u8] = b"MPK-VIR-1.0";
const SAFE_INTEGER_MAX: i64 = 9_007_199_254_740_991;
const DIAGNOSTIC_COUNT_MAX: usize =
    crate::limits::RustLimitId::NormalizedIssueEntries.maximum() as usize;
const DIAGNOSTIC_MESSAGE_MAX: usize =
    crate::limits::RustLimitId::NormalizedIssueMessage.maximum() as usize;
const DIAGNOSTIC_MESSAGE_BYTES_MAX: usize =
    crate::limits::RustLimitId::NormalizedIssueMessageTotal.maximum() as usize;
const REQUEST_FIELDS: &[&str] = &[
    "argument_profile_id",
    "compiler",
    "environment_profile_id",
    "frontend",
    "input_set_hash",
    "inputs",
    "limit_profile",
    "mir_profile_id",
    "release_registry",
    "request_fingerprint",
    "schema",
    "selection",
    "semantic_context",
    "source_inventory",
    "source_inventory_hash",
    "target_allowlist_id",
    "toolchain",
];

const COMMON_OUTPUT_FIELDS: &[&str] = &[
    "argument_profile_id",
    "compiler",
    "diagnostics",
    "environment_profile_id",
    "frontend",
    "input_set_hash",
    "limit_profile",
    "mir_profile_id",
    "phase",
    "release_registry",
    "request_fingerprint",
    "schema",
    "selection",
    "semantic_context",
    "source_inventory_hash",
    "status",
    "target_allowlist_id",
    "toolchain",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverProtocolCode {
    Transport,
    Shape,
    Canonical,
    Hash,
    Identity,
    Filesystem,
    Process,
    Count,
    OutputLimit,
    ToolchainCommit,
    SourceMapExternal,
    SourceMapRange,
    SourceMapReference,
}

impl DriverProtocolCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT",
            Self::Shape => "RUST_FRONTEND_DRIVER_PROTOCOL_SHAPE",
            Self::Canonical => "RUST_FRONTEND_DRIVER_PROTOCOL_CANONICAL",
            Self::Hash => "RUST_FRONTEND_DRIVER_PROTOCOL_HASH",
            Self::Identity => "RUST_FRONTEND_DRIVER_PROTOCOL_IDENTITY",
            Self::Filesystem => "RUST_FRONTEND_DRIVER_PROTOCOL_FILESYSTEM",
            Self::Process => "RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS",
            Self::Count => "RUST_FRONTEND_DRIVER_PROTOCOL_COUNT",
            Self::OutputLimit => "RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT",
            Self::ToolchainCommit => "RUST_TOOLCHAIN_COMMIT",
            Self::SourceMapExternal => "RUST_FRONTEND_SOURCE_MAP_EXTERNAL",
            Self::SourceMapRange => "RUST_FRONTEND_SOURCE_MAP_RANGE",
            Self::SourceMapReference => "RUST_FRONTEND_SOURCE_MAP_REFERENCE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverProtocolError {
    pub code: DriverProtocolCode,
}

impl From<DriverProtocolCode> for DriverProtocolError {
    fn from(code: DriverProtocolCode) -> Self {
        Self { code }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverStatus {
    Lowered,
    Rejected,
    SourceError,
    FrontendError,
}

impl DriverStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lowered => "lowered",
            Self::Rejected => "rejected",
            Self::SourceError => "source-error",
            Self::FrontendError => "frontend-error",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Lowered => 0,
            Self::Rejected => 3,
            Self::SourceError => 4,
            Self::FrontendError => 1,
        }
    }
}

pub use crate::diagnostics::PrivateDiagnostic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverInputIdentity {
    pub kind: String,
    pub normalized_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

impl DriverInputIdentity {
    pub fn from_captured(input: &CapturedInput) -> Self {
        Self {
            kind: match input.kind {
                InputKind::Source => "source",
                InputKind::Contract => "contract",
                InputKind::BuildManifest => "build_manifest",
                InputKind::Lockfile => "lockfile",
            }
            .to_owned(),
            normalized_path: input.normalized_path.as_str().to_owned(),
            size_bytes: input.bytes.len() as u64,
            sha256: input.sha256_hex(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverComponentIdentity {
    pub kind: String,
    pub name: String,
    pub release: String,
    pub sha256: String,
    pub commit_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverReleaseIdentity {
    pub frontend_bundle_id: String,
    pub frontend_binary_sha256: String,
    pub driver_binary_sha256: String,
    pub toolchain_bundle_id: String,
    pub toolchain_distribution_sha256: String,
    pub toolchain_components: Vec<DriverComponentIdentity>,
}

#[derive(Clone, Debug)]
pub struct DriverRequest {
    value: JsonValue,
    transport: Vec<u8>,
    request_fingerprint: String,
    source_inventory_hash: String,
}

impl DriverRequest {
    pub fn transport(&self) -> &[u8] {
        &self.transport
    }

    pub fn request_fingerprint(&self) -> &str {
        &self.request_fingerprint
    }

    pub fn source_inventory_hash(&self) -> &str {
        &self.source_inventory_hash
    }

    pub fn semantic_context(&self) -> &JsonValue {
        object_member(self.root(), "semantic_context").expect("validated request semantic context")
    }

    pub fn selection(&self) -> (&str, &str, &str) {
        let selection = object_member(self.root(), "selection")
            .and_then(JsonValue::as_object)
            .and_then(|selection| selection.get("value"))
            .and_then(JsonValue::as_object)
            .expect("validated request selection");
        (
            selection["package"].as_str().expect("validated package"),
            selection["crate"].as_str().expect("validated crate"),
            selection["function"].as_str().expect("validated function"),
        )
    }

    pub fn target(&self) -> &str {
        semantic_parameter_value(self.root())
            .and_then(|parameters| parameters.get("target_id"))
            .and_then(JsonValue::as_str)
            .expect("validated target")
    }

    pub fn pointer_width(&self) -> u8 {
        let width = semantic_parameter_value(self.root())
            .and_then(|parameters| parameters.get("pointer_width"))
            .and_then(JsonValue::integer)
            .expect("validated pointer width");
        u8::try_from(width).expect("validated pointer width fits in u8")
    }

    pub fn has_source_path(&self, path: &str) -> bool {
        self.source_size(path).is_some()
    }

    pub fn source_inventory(&self) -> Vec<DriverInputIdentity> {
        object_member(self.root(), "source_inventory")
            .and_then(JsonValue::as_array)
            .expect("validated source inventory")
            .iter()
            .map(|entry| {
                let entry = entry.as_object().expect("validated source entry");
                DriverInputIdentity {
                    kind: "source".to_owned(),
                    normalized_path: entry["normalized_path"]
                        .as_str()
                        .expect("validated source path")
                        .to_owned(),
                    size_bytes: u64::try_from(
                        entry["size_bytes"]
                            .integer()
                            .expect("validated source size"),
                    )
                    .expect("validated source size is nonnegative"),
                    sha256: entry["sha256"]
                        .as_str()
                        .expect("validated source digest")
                        .to_owned(),
                }
            })
            .collect()
    }

    pub fn contract_inventory(&self) -> Vec<DriverInputIdentity> {
        object_member(self.root(), "inputs")
            .and_then(JsonValue::as_array)
            .expect("validated input inventory")
            .iter()
            .filter_map(|entry| {
                let entry = entry.as_object()?;
                (entry.get("kind")?.as_str()? == "contract").then(|| DriverInputIdentity {
                    kind: "contract".to_owned(),
                    normalized_path: entry["normalized_path"]
                        .as_str()
                        .expect("validated contract path")
                        .to_owned(),
                    size_bytes: u64::try_from(
                        entry["size_bytes"]
                            .integer()
                            .expect("validated contract size"),
                    )
                    .expect("validated contract size is nonnegative"),
                    sha256: entry["sha256"]
                        .as_str()
                        .expect("validated contract hash")
                        .to_owned(),
                })
            })
            .collect()
    }

    fn source_size(&self, path: &str) -> Option<i64> {
        object_member(self.root(), "source_inventory")
            .and_then(JsonValue::as_array)
            .and_then(|inventory| {
                inventory.iter().find_map(|entry| {
                    let entry = entry.as_object()?;
                    (entry.get("normalized_path")?.as_str()? == path)
                        .then(|| entry.get("size_bytes")?.integer())?
                })
            })
    }

    pub fn driver_binary_sha256(&self) -> &str {
        object_member(self.root(), "frontend")
            .and_then(JsonValue::as_object)
            .and_then(|frontend| frontend.get("subordinate_binaries"))
            .and_then(JsonValue::as_array)
            .and_then(|subordinate| subordinate.first())
            .and_then(JsonValue::as_object)
            .and_then(|driver| driver.get("binary_sha256"))
            .and_then(JsonValue::as_str)
            .expect("validated driver identity")
    }

    pub fn compiler_binary_sha256(&self) -> &str {
        object_member(self.root(), "compiler")
            .and_then(JsonValue::as_object)
            .and_then(|compiler| compiler.get("binary_sha256"))
            .and_then(JsonValue::as_str)
            .expect("validated compiler identity")
    }

    pub fn value(&self) -> &JsonValue {
        &self.value
    }

    fn root(&self) -> &BTreeMap<String, JsonValue> {
        self.value.as_object().expect("validated request root")
    }
}

#[derive(Clone, Debug)]
pub struct DriverOutput {
    value: JsonValue,
    transport: Vec<u8>,
    status: DriverStatus,
    phase: String,
}

impl DriverOutput {
    pub fn status(&self) -> DriverStatus {
        self.status
    }

    pub fn phase(&self) -> &str {
        &self.phase
    }

    pub fn transport(&self) -> &[u8] {
        &self.transport
    }

    pub fn value(&self) -> &JsonValue {
        &self.value
    }
}

pub fn parse_request_transport(bytes: &[u8]) -> Result<DriverRequest, DriverProtocolError> {
    parse_request_transport_with_distribution(
        bytes,
        crate::successor::TOOLCHAIN_DISTRIBUTION_SHA256,
    )
}

fn parse_request_transport_with_distribution(
    bytes: &[u8],
    expected_toolchain_distribution_sha256: &str,
) -> Result<DriverRequest, DriverProtocolError> {
    let (value, canonical) = parse_transport(bytes, REQUEST_TRANSPORT_MAX, false)?;
    validate_request_value(&value, expected_toolchain_distribution_sha256)?;
    let root = value.as_object().ok_or(DriverProtocolCode::Shape)?;
    let request_fingerprint = string(root, "request_fingerprint")?.to_owned();
    let source_inventory_hash = string(root, "source_inventory_hash")?.to_owned();
    Ok(DriverRequest {
        value,
        transport: canonical,
        request_fingerprint,
        source_inventory_hash,
    })
}

pub fn construct_request(
    request: &crate::cli::LowerRequest,
    inputs: &[DriverInputIdentity],
    release: &DriverReleaseIdentity,
) -> Result<DriverRequest, DriverProtocolError> {
    let inputs = JsonValue::Array(
        inputs
            .iter()
            .map(|input| {
                JsonValue::Object(BTreeMap::from([
                    ("kind".to_owned(), JsonValue::String(input.kind.clone())),
                    (
                        "normalized_path".to_owned(),
                        JsonValue::String(input.normalized_path.clone()),
                    ),
                    ("sha256".to_owned(), JsonValue::String(input.sha256.clone())),
                    (
                        "size_bytes".to_owned(),
                        JsonValue::Number(input.size_bytes.to_string()),
                    ),
                ]))
            })
            .collect(),
    );
    let source_inventory = JsonValue::Array(
        inputs
            .as_array()
            .expect("constructed input array")
            .iter()
            .filter_map(|input| {
                let input = input.as_object()?;
                (input.get("kind")?.as_str()? == "source").then(|| {
                    JsonValue::Object(BTreeMap::from([
                        (
                            "normalized_path".to_owned(),
                            input["normalized_path"].clone(),
                        ),
                        ("sha256".to_owned(), input["sha256"].clone()),
                        ("size_bytes".to_owned(), input["size_bytes"].clone()),
                    ]))
                })
            })
            .collect(),
    );
    let components = JsonValue::Array(
        release
            .toolchain_components
            .iter()
            .map(component_value)
            .collect::<Result<_, _>>()?,
    );
    let rustc = release
        .toolchain_components
        .iter()
        .find(|component| component.kind == "executable" && component.name == "rustc")
        .ok_or(DriverProtocolCode::Identity)?;
    let mut root = BTreeMap::from([
        (
            "argument_profile_id".to_owned(),
            JsonValue::String("mpk.rust.frontend_arguments.v0".to_owned()),
        ),
        (
            "compiler".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "binary_sha256".to_owned(),
                    JsonValue::String(rustc.sha256.clone()),
                ),
                (
                    "commit_hash".to_owned(),
                    JsonValue::String(
                        rustc
                            .commit_hash
                            .clone()
                            .ok_or(DriverProtocolCode::Identity)?,
                    ),
                ),
                ("name".to_owned(), JsonValue::String("rustc".to_owned())),
                (
                    "release".to_owned(),
                    JsonValue::String(rustc.release.clone()),
                ),
                (
                    "target".to_owned(),
                    JsonValue::String(request.target.id().to_owned()),
                ),
            ])),
        ),
        (
            "environment_profile_id".to_owned(),
            JsonValue::String("mpk.rust.frontend_environment.v0".to_owned()),
        ),
        (
            "frontend".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "binary_sha256".to_owned(),
                    JsonValue::String(release.frontend_binary_sha256.clone()),
                ),
                (
                    "bundle_id".to_owned(),
                    JsonValue::String(release.frontend_bundle_id.clone()),
                ),
                ("name".to_owned(), JsonValue::String("rust2vir".to_owned())),
                (
                    "subordinate_binaries".to_owned(),
                    JsonValue::Array(vec![JsonValue::Object(BTreeMap::from([
                        (
                            "binary_sha256".to_owned(),
                            JsonValue::String(release.driver_binary_sha256.clone()),
                        ),
                        (
                            "name".to_owned(),
                            JsonValue::String("rust2vir-driver".to_owned()),
                        ),
                        (
                            "version".to_owned(),
                            JsonValue::String(crate::PACKAGE_VERSION.to_owned()),
                        ),
                    ]))]),
                ),
                (
                    "version".to_owned(),
                    JsonValue::String(crate::PACKAGE_VERSION.to_owned()),
                ),
            ])),
        ),
        ("inputs".to_owned(), inputs),
        (
            "limit_profile".to_owned(),
            JsonValue::String("mpk.rust.limits.v0".to_owned()),
        ),
        (
            "mir_profile_id".to_owned(),
            JsonValue::String("mpk.rust.mir.4d08223c.v0".to_owned()),
        ),
        (
            "release_registry".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "id".to_owned(),
                    JsonValue::String(request.release.release_registry_id.clone()),
                ),
                (
                    "registry_sha256".to_owned(),
                    JsonValue::String(request.release.release_registry_sha256.clone()),
                ),
                (
                    "schema".to_owned(),
                    JsonValue::String("mpk.release.bundle_registry.v1".to_owned()),
                ),
            ])),
        ),
        (
            "schema".to_owned(),
            JsonValue::String("mpk.rust.driver.request.v1".to_owned()),
        ),
        (
            "selection".to_owned(),
            crate::successor::selection_envelope(
                &request.selection.package,
                &request.selection.crate_name,
                &request.selection.function,
            ),
        ),
        (
            "semantic_context".to_owned(),
            crate::successor::semantic_context(request.target.id(), request.target.pointer_width()),
        ),
        ("source_inventory".to_owned(), source_inventory),
        (
            "target_allowlist_id".to_owned(),
            JsonValue::String("mpk.rust.targets.v0".to_owned()),
        ),
        (
            "toolchain".to_owned(),
            JsonValue::Object(BTreeMap::from([
                (
                    "bundle_id".to_owned(),
                    JsonValue::String(release.toolchain_bundle_id.clone()),
                ),
                ("components".to_owned(), components),
                (
                    "distribution_sha256".to_owned(),
                    JsonValue::String(release.toolchain_distribution_sha256.clone()),
                ),
            ])),
        ),
    ]);
    root.insert(
        "input_set_hash".to_owned(),
        JsonValue::String(String::new()),
    );
    root.insert(
        "source_inventory_hash".to_owned(),
        JsonValue::String(String::new()),
    );
    seal_request_value_with_distribution(
        JsonValue::Object(root),
        &release.toolchain_distribution_sha256,
    )
}

fn component_value(component: &DriverComponentIdentity) -> Result<JsonValue, DriverProtocolError> {
    let mut value = BTreeMap::from([
        ("kind".to_owned(), JsonValue::String(component.kind.clone())),
        ("name".to_owned(), JsonValue::String(component.name.clone())),
        (
            "release".to_owned(),
            JsonValue::String(component.release.clone()),
        ),
    ]);
    match component.kind.as_str() {
        "executable" => {
            value.insert(
                "binary_sha256".to_owned(),
                JsonValue::String(component.sha256.clone()),
            );
            if let Some(commit) = &component.commit_hash {
                value.insert("commit_hash".to_owned(), JsonValue::String(commit.clone()));
            }
        }
        "content" if component.commit_hash.is_none() => {
            value.insert(
                "content_sha256".to_owned(),
                JsonValue::String(component.sha256.clone()),
            );
        }
        _ => return Err(DriverProtocolCode::Identity.into()),
    }
    Ok(JsonValue::Object(value))
}

pub fn validate_transport_size(
    json_bytes: usize,
    output: bool,
) -> Result<usize, DriverProtocolError> {
    let maximum = if output {
        OUTPUT_TRANSPORT_MAX
    } else {
        REQUEST_TRANSPORT_MAX
    };
    let total = json_bytes.checked_add(1).ok_or_else(|| {
        DriverProtocolError::from(if output {
            DriverProtocolCode::OutputLimit
        } else {
            DriverProtocolCode::Transport
        })
    })?;
    if total > maximum {
        return Err(if output {
            DriverProtocolCode::OutputLimit
        } else {
            DriverProtocolCode::Transport
        }
        .into());
    }
    Ok(total)
}

pub fn seal_request_value(value: JsonValue) -> Result<DriverRequest, DriverProtocolError> {
    seal_request_value_with_distribution(value, crate::successor::TOOLCHAIN_DISTRIBUTION_SHA256)
}

fn seal_request_value_with_distribution(
    mut value: JsonValue,
    expected_toolchain_distribution_sha256: &str,
) -> Result<DriverRequest, DriverProtocolError> {
    {
        let root = value.as_object_mut().ok_or(DriverProtocolCode::Shape)?;
        root.remove("request_fingerprint");
        let input_hash = domain_hash(
            INPUT_SET_DOMAIN,
            &json::canonical(root.get("inputs").ok_or(DriverProtocolCode::Shape)?)
                .map_err(|_| DriverProtocolCode::Canonical)?,
        );
        root.insert("input_set_hash".to_owned(), JsonValue::String(input_hash));
        let inventory_hash = domain_hash(
            INVENTORY_DOMAIN,
            &json::canonical(
                root.get("source_inventory")
                    .ok_or(DriverProtocolCode::Shape)?,
            )
            .map_err(|_| DriverProtocolCode::Canonical)?,
        );
        root.insert(
            "source_inventory_hash".to_owned(),
            JsonValue::String(inventory_hash),
        );
    }
    let preimage = json::canonical_bounded(&value, REQUEST_TRANSPORT_MAX - 1)
        .map_err(|_| DriverProtocolCode::Transport)?;
    value.as_object_mut().expect("request object").insert(
        "request_fingerprint".to_owned(),
        JsonValue::String(domain_hash(REQUEST_DOMAIN, &preimage)),
    );
    let mut transport = json::canonical_bounded(&value, REQUEST_TRANSPORT_MAX - 1)
        .map_err(|_| DriverProtocolCode::Transport)?;
    transport.push(b'\n');
    parse_request_transport_with_distribution(&transport, expected_toolchain_distribution_sha256)
}

pub fn encode_non_success(
    request: &DriverRequest,
    status: DriverStatus,
    phase: &str,
    diagnostics: &[PrivateDiagnostic],
) -> Result<Vec<u8>, DriverProtocolError> {
    if status == DriverStatus::Lowered || diagnostics.is_empty() {
        return Err(DriverProtocolCode::Shape.into());
    }
    let diagnostic_status = private_diagnostic_status(status).ok_or(DriverProtocolCode::Shape)?;
    let diagnostics = normalize_private_diagnostics(diagnostics, diagnostic_status, phase)?;
    let mut root = common_output_root(request)?;
    root.insert(
        "status".to_owned(),
        JsonValue::String(status.as_str().to_owned()),
    );
    root.insert("phase".to_owned(), JsonValue::String(phase.to_owned()));
    root.insert("diagnostics".to_owned(), diagnostics_json(&diagnostics));
    let mut bytes = json::canonical_bounded(&JsonValue::Object(root), OUTPUT_TRANSPORT_MAX - 1)
        .map_err(|_| DriverProtocolCode::OutputLimit)?;
    bytes.push(b'\n');
    parse_output_transport(&bytes, request, status.exit_code(), false)?;
    Ok(bytes)
}

pub fn encode_lowered(
    request: &DriverRequest,
    raw_lowering: JsonValue,
    raw_source_map: JsonValue,
) -> Result<Vec<u8>, DriverProtocolError> {
    let mut root = common_output_root(request)?;
    root.insert("status".to_owned(), JsonValue::String("lowered".to_owned()));
    root.insert("phase".to_owned(), JsonValue::String("lowering".to_owned()));
    root.insert("diagnostics".to_owned(), JsonValue::Array(Vec::new()));
    root.insert(
        "source_inventory".to_owned(),
        request
            .root()
            .get("source_inventory")
            .ok_or(DriverProtocolCode::Shape)?
            .clone(),
    );
    root.insert("raw_lowering".to_owned(), raw_lowering);
    root.insert("raw_source_map".to_owned(), raw_source_map);
    let mut output = JsonValue::Object(root);
    let preimage = json::canonical_bounded(&output, OUTPUT_TRANSPORT_MAX - 1)
        .map_err(|_| DriverProtocolCode::OutputLimit)?;
    output
        .as_object_mut()
        .expect("constructed driver output object")
        .insert(
            "payload_hash".to_owned(),
            JsonValue::String(domain_hash(PAYLOAD_DOMAIN, &preimage)),
        );
    let mut bytes = json::canonical_bounded(&output, OUTPUT_TRANSPORT_MAX - 1)
        .map_err(|_| DriverProtocolCode::OutputLimit)?;
    bytes.push(b'\n');
    parse_output_transport(&bytes, request, DriverStatus::Lowered.exit_code(), false)?;
    Ok(bytes)
}

fn common_output_root(
    request: &DriverRequest,
) -> Result<BTreeMap<String, JsonValue>, DriverProtocolError> {
    let mut root = BTreeMap::new();
    for field in COMMON_OUTPUT_FIELDS {
        if matches!(*field, "diagnostics" | "phase" | "schema" | "status") {
            continue;
        }
        root.insert(
            (*field).to_owned(),
            request
                .root()
                .get(*field)
                .ok_or(DriverProtocolCode::Shape)?
                .clone(),
        );
    }
    root.insert(
        "schema".to_owned(),
        JsonValue::String("mpk.rust.driver.v1".to_owned()),
    );
    Ok(root)
}

fn diagnostics_json(diagnostics: &[PrivateDiagnostic]) -> JsonValue {
    JsonValue::Array(
        diagnostics
            .iter()
            .map(|diagnostic| {
                let mut issue = BTreeMap::from([
                    (
                        "code".to_owned(),
                        JsonValue::String(diagnostic.code.clone()),
                    ),
                    (
                        "message".to_owned(),
                        JsonValue::String(diagnostic.message.clone()),
                    ),
                ]);
                if let Some(function) = &diagnostic.function_id {
                    issue.insert(
                        "function_id".to_owned(),
                        JsonValue::String(function.clone()),
                    );
                }
                JsonValue::Object(issue)
            })
            .collect(),
    )
}

pub fn parse_output_transport(
    bytes: &[u8],
    request: &DriverRequest,
    exit_code: i32,
    signaled: bool,
) -> Result<DriverOutput, DriverProtocolError> {
    parse_output_transport_inner(bytes, request, Some(exit_code), signaled)
}

pub fn parse_output_artifact(
    bytes: &[u8],
    request: &DriverRequest,
) -> Result<DriverOutput, DriverProtocolError> {
    parse_output_transport_inner(bytes, request, None, false)
}

fn parse_output_transport_inner(
    bytes: &[u8],
    request: &DriverRequest,
    exit_code: Option<i32>,
    signaled: bool,
) -> Result<DriverOutput, DriverProtocolError> {
    if signaled {
        return Err(DriverProtocolCode::Process.into());
    }
    let (value, canonical) = parse_transport(bytes, OUTPUT_TRANSPORT_MAX, true)?;
    let root = value.as_object().ok_or(DriverProtocolCode::Shape)?;
    let status = match string(root, "status")? {
        "lowered" => DriverStatus::Lowered,
        "rejected" => DriverStatus::Rejected,
        "source-error" => DriverStatus::SourceError,
        "frontend-error" => DriverStatus::FrontendError,
        _ => return Err(DriverProtocolCode::Shape.into()),
    };
    let mut fields = COMMON_OUTPUT_FIELDS.to_vec();
    if status == DriverStatus::Lowered {
        fields.extend([
            "payload_hash",
            "raw_lowering",
            "raw_source_map",
            "source_inventory",
        ]);
    }
    closed(root, &fields)?;
    if string(root, "schema")? != "mpk.rust.driver.v1"
        || exit_code.is_some_and(|exit_code| exit_code != status.exit_code())
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    let phase = string(root, "phase")?.to_owned();
    validate_status_phase(status, &phase)?;
    validate_output_common(root, request)?;
    let diagnostics = root
        .get("diagnostics")
        .and_then(JsonValue::as_array)
        .ok_or(DriverProtocolCode::Shape)?;
    validate_diagnostics(diagnostics, request, status, &phase)?;
    if status != DriverStatus::Lowered && diagnostics.is_empty() {
        return Err(DriverProtocolCode::Shape.into());
    }
    if status == DriverStatus::Lowered {
        validate_success(root, request)?;
    }
    Ok(DriverOutput {
        value,
        transport: canonical,
        status,
        phase,
    })
}

fn parse_transport(
    bytes: &[u8],
    maximum: usize,
    output: bool,
) -> Result<(JsonValue, Vec<u8>), DriverProtocolError> {
    if bytes.len() > maximum {
        return Err(if output {
            DriverProtocolCode::OutputLimit
        } else {
            DriverProtocolCode::Transport
        }
        .into());
    }
    if bytes.len() < 2
        || bytes.last() != Some(&b'\n')
        || bytes[..bytes.len() - 1].contains(&b'\n')
        || bytes.contains(&b'\r')
    {
        return Err(DriverProtocolCode::Transport.into());
    }
    let body = &bytes[..bytes.len() - 1];
    let value = json::parse(body, maximum - 1).map_err(|_| DriverProtocolCode::Canonical)?;
    let encoded = json::canonical(&value).map_err(|_| DriverProtocolCode::Canonical)?;
    if encoded != body {
        return Err(DriverProtocolCode::Canonical.into());
    }
    let mut canonical = encoded;
    canonical.push(b'\n');
    Ok((value, canonical))
}

fn validate_request_value(
    value: &JsonValue,
    expected_toolchain_distribution_sha256: &str,
) -> Result<(), DriverProtocolError> {
    let root = value.as_object().ok_or(DriverProtocolCode::Shape)?;
    closed(root, REQUEST_FIELDS)?;
    if string(root, "schema")? != "mpk.rust.driver.request.v1"
        || string(root, "limit_profile")? != "mpk.rust.limits.v0"
        || string(root, "target_allowlist_id")? != "mpk.rust.targets.v0"
        || string(root, "environment_profile_id")? != "mpk.rust.frontend_environment.v0"
        || string(root, "argument_profile_id")? != "mpk.rust.frontend_arguments.v0"
        || string(root, "mir_profile_id")? != "mpk.rust.mir.4d08223c.v0"
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    validate_semantic_context(object(root, "semantic_context")?)?;
    validate_selection_envelope(object(root, "selection")?)?;
    validate_release_registry(object(root, "release_registry")?)?;
    validate_frontend(object(root, "frontend")?)?;
    validate_toolchain(
        object(root, "toolchain")?,
        expected_toolchain_distribution_sha256,
    )?;
    validate_compiler(root)?;
    let inputs = array(root, "inputs")?;
    validate_inputs(inputs)?;
    let inventory = array(root, "source_inventory")?;
    validate_inventory(inventory, inputs)?;
    let inventory_jcs = json::canonical(root.get("source_inventory").expect("closed root"))
        .map_err(|_| DriverProtocolCode::Canonical)?;
    if !sha256(string(root, "source_inventory_hash")?)
        || string(root, "source_inventory_hash")? != domain_hash(INVENTORY_DOMAIN, &inventory_jcs)
    {
        return Err(DriverProtocolCode::Hash.into());
    }
    let input_jcs = json::canonical(root.get("inputs").expect("closed root"))
        .map_err(|_| DriverProtocolCode::Canonical)?;
    if !sha256(string(root, "input_set_hash")?)
        || string(root, "input_set_hash")? != domain_hash(INPUT_SET_DOMAIN, &input_jcs)
    {
        return Err(DriverProtocolCode::Identity.into());
    }
    if !sha256(string(root, "request_fingerprint")?) {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut preimage = value.clone();
    preimage
        .as_object_mut()
        .expect("request object")
        .remove("request_fingerprint");
    let preimage = json::canonical(&preimage).map_err(|_| DriverProtocolCode::Canonical)?;
    if string(root, "request_fingerprint")? != domain_hash(REQUEST_DOMAIN, &preimage) {
        return Err(DriverProtocolCode::Hash.into());
    }
    Ok(())
}

fn semantic_parameter_value(
    root: &BTreeMap<String, JsonValue>,
) -> Option<&BTreeMap<String, JsonValue>> {
    root.get("semantic_context")?
        .as_object()?
        .get("semantic_parameters")?
        .as_object()?
        .get("value")?
        .as_object()
}

fn validate_semantic_context(
    context: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(
        context,
        &[
            "profile_entry_sha256",
            "profile_registry",
            "semantic_parameters",
            "semantic_profile",
            "source_language",
        ],
    )?;
    let parameters = object(context, "semantic_parameters")?;
    closed(parameters, &["schema", "value"])?;
    if string(parameters, "schema")? != crate::successor::PARAMETERS_SCHEMA {
        return Err(DriverProtocolCode::Shape.into());
    }
    let value = object(parameters, "value")?;
    validate_semantic_parameters(value)?;
    let width =
        u8::try_from(integer(value, "pointer_width")?).map_err(|_| DriverProtocolCode::Shape)?;
    let expected = crate::successor::semantic_context(string(value, "target_id")?, width);
    if JsonValue::Object(context.clone()) != expected {
        return Err(DriverProtocolCode::Identity.into());
    }
    Ok(())
}

fn validate_semantic_parameters(
    parameters: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(
        parameters,
        &["overflow_mode", "panic_mode", "pointer_width", "target_id"],
    )?;
    let target = string(parameters, "target_id")?;
    let width = integer(parameters, "pointer_width")?;
    if !matches!(
        (target, width),
        ("i686-unknown-linux-gnu", 32) | ("x86_64-unknown-linux-gnu", 64)
    ) || string(parameters, "overflow_mode")? != "checked"
        || string(parameters, "panic_mode")? != "abort"
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn validate_selection_envelope(
    selection: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(selection, &["schema", "value"])?;
    if string(selection, "schema")? != crate::successor::SELECTION_SCHEMA {
        return Err(DriverProtocolCode::Shape.into());
    }
    validate_selection(object(selection, "value")?)
}

fn validate_selection(selection: &BTreeMap<String, JsonValue>) -> Result<(), DriverProtocolError> {
    closed(selection, &["crate", "function", "kind", "package"])?;
    let package = string(selection, "package")?;
    let crate_name = string(selection, "crate")?;
    let function = string(selection, "function")?;
    if string(selection, "kind")? != "lib"
        || !package_name(package)
        || !identifier(crate_name)
        || function.split("::").next() != Some(crate_name)
        || function.split("::").count() < 2
        || !function.split("::").all(identifier)
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn validate_release_registry(
    registry: &BTreeMap<String, JsonValue>,
) -> Result<(), DriverProtocolError> {
    closed(registry, &["id", "registry_sha256", "schema"])?;
    if string(registry, "schema")? != "mpk.release.bundle_registry.v1"
        || string(registry, "id")? != "mpk.release.registry.v1"
        || !sha256(string(registry, "registry_sha256")?)
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn validate_frontend(frontend: &BTreeMap<String, JsonValue>) -> Result<(), DriverProtocolError> {
    closed(
        frontend,
        &[
            "binary_sha256",
            "bundle_id",
            "name",
            "subordinate_binaries",
            "version",
        ],
    )?;
    let subordinate = array(frontend, "subordinate_binaries")?;
    if string(frontend, "name")? != "rust2vir"
        || string(frontend, "version")? != crate::PACKAGE_VERSION
        || string(frontend, "bundle_id")? != crate::successor::FRONTEND_ID
        || !sha256(string(frontend, "binary_sha256")?)
        || subordinate.len() != 1
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    let subordinate = subordinate[0]
        .as_object()
        .ok_or(DriverProtocolCode::Shape)?;
    closed(subordinate, &["binary_sha256", "name", "version"])?;
    if string(subordinate, "name")? != "rust2vir-driver"
        || string(subordinate, "version")? != crate::PACKAGE_VERSION
        || !sha256(string(subordinate, "binary_sha256")?)
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn validate_toolchain(
    toolchain: &BTreeMap<String, JsonValue>,
    expected_distribution_sha256: &str,
) -> Result<(), DriverProtocolError> {
    closed(
        toolchain,
        &["bundle_id", "components", "distribution_sha256"],
    )?;
    if string(toolchain, "bundle_id")? != crate::successor::TOOLCHAIN_ID
        || string(toolchain, "distribution_sha256")? != expected_distribution_sha256
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    let components = array(toolchain, "components")?;
    if components.is_empty() || components.len() > 64 {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut previous = None;
    let mut cargo = false;
    let mut rustc = None;
    for component in components {
        let component = component.as_object().ok_or(DriverProtocolCode::Shape)?;
        let kind = string(component, "kind")?;
        let name = string(component, "name")?;
        let release = string(component, "release")?;
        let expected_kind = if matches!(name, "cargo" | "rustc") {
            "executable"
        } else {
            "content"
        };
        if previous.is_some_and(|previous: &str| previous.as_bytes() >= name.as_bytes()) {
            return Err(DriverProtocolCode::Shape.into());
        }
        if kind != expected_kind {
            return Err(DriverProtocolCode::Shape.into());
        }
        previous = Some(name);
        match kind {
            "executable" => {
                let fields: &[&str] = if name == "rustc" {
                    &["binary_sha256", "commit_hash", "kind", "name", "release"]
                } else {
                    &["binary_sha256", "kind", "name", "release"]
                };
                closed(component, fields)?;
                if !sha256(string(component, "binary_sha256")?) {
                    return Err(DriverProtocolCode::Shape.into());
                }
                if name == "rustc" {
                    rustc = Some(component);
                } else if name == "cargo" {
                    cargo = true;
                }
            }
            "content" => {
                closed(component, &["content_sha256", "kind", "name", "release"])?;
                if !sha256(string(component, "content_sha256")?) {
                    return Err(DriverProtocolCode::Shape.into());
                }
            }
            _ => return Err(DriverProtocolCode::Shape.into()),
        }
        if !release_id(name) || !version(release) {
            return Err(DriverProtocolCode::Shape.into());
        }
    }
    if !cargo {
        return Err(DriverProtocolCode::Shape.into());
    }
    let rustc = rustc.ok_or(DriverProtocolCode::Shape)?;
    if string(rustc, "release")? != crate::EXPECTED_RUSTC_RELEASE
        || string(rustc, "commit_hash")? != crate::EXPECTED_RUSTC_COMMIT
    {
        return Err(DriverProtocolCode::ToolchainCommit.into());
    }
    Ok(())
}

fn validate_compiler(root: &BTreeMap<String, JsonValue>) -> Result<(), DriverProtocolError> {
    let compiler = object(root, "compiler")?;
    closed(
        compiler,
        &["binary_sha256", "commit_hash", "name", "release", "target"],
    )?;
    if string(compiler, "name")? != "rustc"
        || string(compiler, "release")? != crate::EXPECTED_RUSTC_RELEASE
        || !sha256(string(compiler, "binary_sha256")?)
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    if string(compiler, "commit_hash")? != crate::EXPECTED_RUSTC_COMMIT {
        return Err(DriverProtocolCode::ToolchainCommit.into());
    }
    let parameters = semantic_parameter_value(root).ok_or(DriverProtocolCode::Shape)?;
    if string(compiler, "target")? != string(parameters, "target_id")? {
        return Err(DriverProtocolCode::Identity.into());
    }
    let rustc = array(object(root, "toolchain")?, "components")?
        .iter()
        .filter_map(JsonValue::as_object)
        .find(|component| component.get("name").and_then(JsonValue::as_str) == Some("rustc"))
        .ok_or(DriverProtocolCode::Shape)?;
    for field in ["binary_sha256", "commit_hash", "release"] {
        if compiler.get(field) != rustc.get(field) {
            return Err(DriverProtocolCode::Identity.into());
        }
    }
    Ok(())
}

fn validate_inputs(inputs: &[JsonValue]) -> Result<(), DriverProtocolError> {
    if inputs.is_empty() || inputs.len() > 512 {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut previous: Option<(&str, &str)> = None;
    let mut paths = BTreeSet::new();
    for input in inputs {
        let input = input.as_object().ok_or(DriverProtocolCode::Shape)?;
        closed(input, &["kind", "normalized_path", "sha256", "size_bytes"])?;
        let kind = string(input, "kind")?;
        let path = string(input, "normalized_path")?;
        let size = integer(input, "size_bytes")?;
        if !matches!(kind, "source" | "contract" | "build_manifest" | "lockfile")
            || !portable_path(path)
            || !(0..=4_294_967_296).contains(&size)
            || !sha256(string(input, "sha256")?)
            || !paths.insert(path.to_ascii_lowercase())
            || previous.is_some_and(|(old_path, old_kind)| {
                (old_path.as_bytes(), old_kind.as_bytes()) >= (path.as_bytes(), kind.as_bytes())
            })
        {
            return Err(DriverProtocolCode::Shape.into());
        }
        if kind == "source" && size == 0 {
            return Err(DriverProtocolCode::Shape.into());
        }
        previous = Some((path, kind));
    }
    Ok(())
}

fn validate_inventory(
    inventory: &[JsonValue],
    inputs: &[JsonValue],
) -> Result<(), DriverProtocolError> {
    if inventory.is_empty() {
        return Err(DriverProtocolCode::Shape.into());
    }
    let projected = inputs
        .iter()
        .filter_map(|input| {
            let input = input.as_object()?;
            (input.get("kind")?.as_str()? == "source").then(|| {
                JsonValue::Object(BTreeMap::from([
                    (
                        "normalized_path".to_owned(),
                        input["normalized_path"].clone(),
                    ),
                    ("sha256".to_owned(), input["sha256"].clone()),
                    ("size_bytes".to_owned(), input["size_bytes"].clone()),
                ]))
            })
        })
        .collect::<Vec<_>>();
    if projected != inventory {
        return Err(DriverProtocolCode::Identity.into());
    }
    Ok(())
}

fn validate_output_common(
    root: &BTreeMap<String, JsonValue>,
    request: &DriverRequest,
) -> Result<(), DriverProtocolError> {
    for field in COMMON_OUTPUT_FIELDS {
        if matches!(*field, "diagnostics" | "phase" | "schema" | "status") {
            continue;
        }
        let actual = root.get(*field).ok_or(DriverProtocolCode::Shape)?;
        let expected = request
            .root()
            .get(*field)
            .ok_or(DriverProtocolCode::Shape)?;
        if actual != expected {
            return Err(match *field {
                "source_inventory_hash" => DriverProtocolCode::Hash,
                "compiler" => {
                    let commit = actual
                        .as_object()
                        .and_then(|compiler| compiler.get("commit_hash"))
                        .and_then(JsonValue::as_str);
                    if commit != Some(crate::EXPECTED_RUSTC_COMMIT) {
                        DriverProtocolCode::ToolchainCommit
                    } else {
                        DriverProtocolCode::Identity
                    }
                }
                _ => DriverProtocolCode::Identity,
            }
            .into());
        }
    }
    Ok(())
}

fn validate_status_phase(status: DriverStatus, phase: &str) -> Result<(), DriverProtocolError> {
    let valid = match status {
        DriverStatus::Lowered => phase == "lowering",
        DriverStatus::Rejected => matches!(phase, "source" | "subset" | "lowering"),
        DriverStatus::SourceError => matches!(phase, "source" | "typecheck"),
        DriverStatus::FrontendError => {
            matches!(phase, "source" | "typecheck" | "lowering" | "emission")
        }
    };
    if valid {
        Ok(())
    } else {
        Err(DriverProtocolCode::Shape.into())
    }
}

fn validate_diagnostics(
    diagnostics: &[JsonValue],
    request: &DriverRequest,
    status: DriverStatus,
    phase: &str,
) -> Result<(), DriverProtocolError> {
    let diagnostic_status = private_diagnostic_status(status);
    if diagnostics.len() > DIAGNOSTIC_COUNT_MAX {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut message_bytes = 0_usize;
    let mut previous: Option<(String, i64, String, String, String, i64)> = None;
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let issue = diagnostic.as_object().ok_or(DriverProtocolCode::Shape)?;
        let allowed: &[&str] = match (
            issue.contains_key("function_id"),
            issue.contains_key("span"),
        ) {
            (false, false) => &["code", "message"],
            (true, false) => &["code", "function_id", "message"],
            (false, true) => &["code", "message", "span"],
            (true, true) => &["code", "function_id", "message", "span"],
        };
        closed(issue, allowed)?;
        let code = string(issue, "code")?;
        let message = string(issue, "message")?;
        if code.is_empty()
            || code.len() > 128
            || !code.starts_with("RUST_")
            || !code
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
            || message.is_empty()
            || message.len() > DIAGNOSTIC_MESSAGE_MAX
            || message.chars().any(char::is_control)
        {
            return Err(DriverProtocolCode::Shape.into());
        }
        message_bytes = message_bytes
            .checked_add(message.len())
            .ok_or(DriverProtocolCode::Shape)?;
        if message_bytes > DIAGNOSTIC_MESSAGE_BYTES_MAX {
            return Err(DriverProtocolCode::Shape.into());
        }
        let marker = code == DIAGNOSTIC_TRUNCATION_CODE;
        if diagnostic_status.is_none_or(|status| !valid_private_diagnostic(code, status, phase)) {
            return Err(DriverProtocolCode::Shape.into());
        }
        if marker
            && (index + 1 != diagnostics.len()
                || issue.contains_key("function_id")
                || issue.contains_key("span")
                || !valid_truncation_message(message))
        {
            return Err(DriverProtocolCode::Shape.into());
        }
        if let Some(function) = issue.get("function_id") {
            if function
                .as_str()
                .is_none_or(|function| !same_crate_function_id(function, request.selection().1))
            {
                return Err(DriverProtocolCode::Shape.into());
            }
        }
        let (span_path, span_start, span_end) = if let Some(span) = issue.get("span") {
            validate_span(span, request)?;
            let span = span.as_object().expect("validated span");
            (
                string(span, "normalized_path")?.to_owned(),
                integer(span, "start")?,
                integer(span, "end")?,
            )
        } else {
            (String::new(), 0, 0)
        };
        let function = issue
            .get("function_id")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_owned();
        let key = (
            span_path,
            span_start,
            code.to_owned(),
            message.to_owned(),
            function,
            span_end,
        );
        if !marker && previous.as_ref().is_some_and(|previous| previous > &key) {
            return Err(DriverProtocolCode::Shape.into());
        }
        if !marker {
            previous = Some(key);
        }
    }
    Ok(())
}

fn private_diagnostic_status(status: DriverStatus) -> Option<PrivateDiagnosticStatus> {
    match status {
        DriverStatus::Lowered => None,
        DriverStatus::Rejected => Some(PrivateDiagnosticStatus::Rejected),
        DriverStatus::SourceError => Some(PrivateDiagnosticStatus::SourceError),
        DriverStatus::FrontendError => Some(PrivateDiagnosticStatus::FrontendError),
    }
}

fn same_crate_function_id(function: &str, crate_name: &str) -> bool {
    let mut segments = function.split("::");
    if segments.next() != Some(crate_name) {
        return false;
    }
    let Some(member) = segments.next() else {
        return false;
    };
    identifier(member) && segments.all(identifier)
}

fn valid_truncation_message(message: &str) -> bool {
    let Some(omitted) = message.strip_suffix(" normalized issues omitted") else {
        return false;
    };
    !omitted.is_empty()
        && omitted != "0"
        && !omitted.starts_with('0')
        && omitted.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_span(value: &JsonValue, request: &DriverRequest) -> Result<(), DriverProtocolError> {
    let span = value.as_object().ok_or(DriverProtocolCode::Shape)?;
    closed(span, &["end", "normalized_path", "start"])?;
    let start = integer(span, "start")?;
    let end = integer(span, "end")?;
    let path = string(span, "normalized_path")?;
    if !portable_path(path)
        || start < 0
        || start >= end
        || request.source_size(path).is_none_or(|size| end > size)
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn validate_success(
    root: &BTreeMap<String, JsonValue>,
    request: &DriverRequest,
) -> Result<(), DriverProtocolError> {
    if root.get("source_inventory") != request.root().get("source_inventory") {
        return Err(DriverProtocolCode::Identity.into());
    }
    let lowering = object(root, "raw_lowering")?;
    closed(lowering, &["mir_profile_id", "schema", "vir"])?;
    if string(lowering, "schema")? != "mpk.rust.driver.lowering.v1"
        || lowering.get("mir_profile_id") != root.get("mir_profile_id")
    {
        return Err(DriverProtocolCode::Shape.into());
    }
    let vir = object(lowering, "vir")?;
    let vir_hash = string(vir, "vir_hash")?;
    if !sha256(vir_hash) {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut vir_preimage = JsonValue::Object(vir.clone());
    vir_preimage
        .as_object_mut()
        .expect("vir object")
        .remove("vir_hash");
    let vir_preimage = json::canonical(&vir_preimage).map_err(|_| DriverProtocolCode::Canonical)?;
    if vir_hash != domain_hash(VIR_DOMAIN, &vir_preimage) {
        return Err(DriverProtocolCode::Identity.into());
    }
    if vir.get("semantic_context") != root.get("semantic_context") {
        return Err(DriverProtocolCode::Identity.into());
    }
    validate_raw_source_map(object(root, "raw_source_map")?, request, vir, vir_hash)?;
    let payload_hash = string(root, "payload_hash")?;
    if !sha256(payload_hash) {
        return Err(DriverProtocolCode::Shape.into());
    }
    let mut payload = JsonValue::Object(root.clone());
    payload
        .as_object_mut()
        .expect("output object")
        .remove("payload_hash");
    let payload = json::canonical(&payload).map_err(|_| DriverProtocolCode::Canonical)?;
    if payload_hash != domain_hash(PAYLOAD_DOMAIN, &payload) {
        return Err(DriverProtocolCode::Hash.into());
    }
    Ok(())
}

fn validate_raw_source_map(
    map: &BTreeMap<String, JsonValue>,
    request: &DriverRequest,
    vir: &BTreeMap<String, JsonValue>,
    vir_hash: &str,
) -> Result<(), DriverProtocolError> {
    closed(
        map,
        &[
            "entries",
            "schema",
            "semantic_context",
            "source_ir_hash",
            "source_ir_schema",
        ],
    )?;
    if string(map, "schema")? != "mpk.rust.driver.raw_source_map.v1"
        || string(map, "source_ir_schema")? != "mpk.vir.v1"
        || string(map, "source_ir_hash")? != vir_hash
        || map.get("semantic_context") != request.root().get("semantic_context")
        || map.get("semantic_context") != vir.get("semantic_context")
    {
        return Err(DriverProtocolCode::Identity.into());
    }
    let inventory = request
        .root()
        .get("source_inventory")
        .and_then(JsonValue::as_array)
        .ok_or(DriverProtocolCode::Shape)?;
    let sizes = inventory
        .iter()
        .map(|entry| {
            let entry = entry.as_object().ok_or(DriverProtocolCode::Shape)?;
            Ok((
                string(entry, "normalized_path")?,
                integer(entry, "size_bytes")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, DriverProtocolError>>()?;
    let mut actual = BTreeSet::new();
    let mut previous = None;
    for entry in array(map, "entries")? {
        let entry = entry.as_object().ok_or(DriverProtocolCode::Shape)?;
        closed(entry, &["origin", "reference"])?;
        let reference = reference_key(entry.get("reference").ok_or(DriverProtocolCode::Shape)?)?;
        if previous
            .as_ref()
            .is_some_and(|previous| previous >= &reference)
            || !actual.insert(reference.clone())
        {
            return Err(DriverProtocolCode::SourceMapReference.into());
        }
        previous = Some(reference);
        let origin = entry
            .get("origin")
            .and_then(JsonValue::as_object)
            .ok_or(DriverProtocolCode::Shape)?;
        if string(origin, "kind")? == "source" {
            closed(
                origin,
                &["end", "input_kind", "kind", "normalized_path", "start"],
            )?;
            let path = string(origin, "normalized_path")?;
            if string(origin, "input_kind")? != "source" || !sizes.contains_key(path) {
                return Err(DriverProtocolCode::SourceMapExternal.into());
            }
            let start = integer(origin, "start")?;
            let end = integer(origin, "end")?;
            if start < 0 || start >= end || end > sizes[path] {
                return Err(DriverProtocolCode::SourceMapRange.into());
            }
        } else if string(origin, "kind")? == "synthetic" {
            closed(origin, &["kind", "reason"])?;
            // Rust v0 has an empty synthetic-origin allowlist. A compiler-created or
            // expansion-only node must be rejected before output rather than assigned an
            // invented nearest-neighbour span.
            return Err(DriverProtocolCode::SourceMapExternal.into());
        } else {
            return Err(DriverProtocolCode::Shape.into());
        }
    }
    let expected = vir_reference_keys(vir)?;
    if actual != expected {
        return Err(DriverProtocolCode::SourceMapReference.into());
    }
    Ok(())
}

fn vir_reference_keys(
    vir: &BTreeMap<String, JsonValue>,
) -> Result<BTreeSet<ReferenceKey>, DriverProtocolError> {
    let mut references = BTreeSet::new();
    let units = array(vir, "units")?;
    for unit in units {
        let unit = unit.as_object().ok_or(DriverProtocolCode::Shape)?;
        let unit_id = string(unit, "id")?;
        for function in array(unit, "functions")? {
            let function = function.as_object().ok_or(DriverProtocolCode::Shape)?;
            let function_id = string(function, "id")?;
            references.insert(ReferenceKey::function(unit_id, function_id));
            for block in array(function, "blocks")? {
                let block = block.as_object().ok_or(DriverProtocolCode::Shape)?;
                let label = string(block, "label")?;
                let block_index = dense_index(label, "bb")?;
                for instruction in array(block, "instructions")? {
                    let instruction = instruction.as_object().ok_or(DriverProtocolCode::Shape)?;
                    references.insert(ReferenceKey::instruction(
                        unit_id,
                        function_id,
                        block_index,
                        dense_index(string(instruction, "id")?, "t")?,
                    ));
                }
                references.insert(ReferenceKey::terminator(unit_id, function_id, block_index));
            }
        }
    }
    Ok(references)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ReferenceKey {
    unit_id: String,
    function_id: String,
    rank: u8,
    block_index: i64,
    instruction_index: i64,
}

impl ReferenceKey {
    fn function(unit_id: &str, function_id: &str) -> Self {
        Self::new(unit_id, function_id, 0, -1, -1)
    }

    fn instruction(
        unit_id: &str,
        function_id: &str,
        block_index: i64,
        instruction_index: i64,
    ) -> Self {
        Self::new(unit_id, function_id, 1, block_index, instruction_index)
    }

    fn terminator(unit_id: &str, function_id: &str, block_index: i64) -> Self {
        Self::new(unit_id, function_id, 2, block_index, -1)
    }

    fn new(
        unit_id: &str,
        function_id: &str,
        rank: u8,
        block_index: i64,
        instruction_index: i64,
    ) -> Self {
        Self {
            unit_id: unit_id.to_owned(),
            function_id: function_id.to_owned(),
            rank,
            block_index,
            instruction_index,
        }
    }
}

fn reference_key(value: &JsonValue) -> Result<ReferenceKey, DriverProtocolError> {
    let reference = value.as_object().ok_or(DriverProtocolCode::Shape)?;
    match string(reference, "kind")? {
        "function" => {
            closed(reference, &["function_id", "kind", "unit_id"])?;
            Ok(ReferenceKey::function(
                string(reference, "unit_id")?,
                string(reference, "function_id")?,
            ))
        }
        "instruction" => {
            closed(
                reference,
                &["block", "function_id", "instruction", "kind", "unit_id"],
            )?;
            Ok(ReferenceKey::instruction(
                string(reference, "unit_id")?,
                string(reference, "function_id")?,
                dense_index(string(reference, "block")?, "bb")?,
                dense_index(string(reference, "instruction")?, "t")?,
            ))
        }
        "terminator" => {
            closed(reference, &["block", "function_id", "kind", "unit_id"])?;
            Ok(ReferenceKey::terminator(
                string(reference, "unit_id")?,
                string(reference, "function_id")?,
                dense_index(string(reference, "block")?, "bb")?,
            ))
        }
        _ => Err(DriverProtocolCode::Shape.into()),
    }
}

fn domain_hash(domain: &[u8], payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(payload);
    hex(&hasher.finish())
}

fn closed(
    object: &BTreeMap<String, JsonValue>,
    fields: &[&str],
) -> Result<(), DriverProtocolError> {
    if object.len() != fields.len() || fields.iter().any(|field| !object.contains_key(*field)) {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(())
}

fn object<'a>(
    root: &'a BTreeMap<String, JsonValue>,
    field: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, DriverProtocolError> {
    root.get(field)
        .and_then(JsonValue::as_object)
        .ok_or_else(|| DriverProtocolCode::Shape.into())
}

fn array<'a>(
    root: &'a BTreeMap<String, JsonValue>,
    field: &str,
) -> Result<&'a [JsonValue], DriverProtocolError> {
    root.get(field)
        .and_then(JsonValue::as_array)
        .ok_or_else(|| DriverProtocolCode::Shape.into())
}

fn string<'a>(
    root: &'a BTreeMap<String, JsonValue>,
    field: &str,
) -> Result<&'a str, DriverProtocolError> {
    root.get(field)
        .and_then(JsonValue::as_str)
        .ok_or_else(|| DriverProtocolCode::Shape.into())
}

fn integer(root: &BTreeMap<String, JsonValue>, field: &str) -> Result<i64, DriverProtocolError> {
    let value = root
        .get(field)
        .and_then(JsonValue::integer)
        .ok_or(DriverProtocolCode::Shape)?;
    if value.unsigned_abs() > SAFE_INTEGER_MAX as u64 {
        return Err(DriverProtocolCode::Shape.into());
    }
    Ok(value)
}

fn object_member<'a>(root: &'a BTreeMap<String, JsonValue>, field: &str) -> Option<&'a JsonValue> {
    root.get(field)
}

fn sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn identifier(value: &str) -> bool {
    if value == "_" || !value.is_ascii() {
        return false;
    }
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn package_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn release_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let mut expect_alphanumeric = true;
    for byte in value.bytes() {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            expect_alphanumeric = false;
        } else if matches!(byte, b'.' | b'_' | b'-') && !expect_alphanumeric {
            expect_alphanumeric = true;
        } else {
            return false;
        }
    }
    !expect_alphanumeric
}

fn version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.is_ascii()
        && !value.starts_with(char::is_whitespace)
        && !value.ends_with(char::is_whitespace)
        && value
            .bytes()
            .all(|byte| (0x20..=0x7e).contains(&byte) && !matches!(byte, b'\\' | b'/'))
}

fn portable_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1_024
        && value.is_ascii()
        && value.split('/').all(portable_component)
}

fn portable_component(component: &str) -> bool {
    if component.is_empty()
        || component.len() > 255
        || matches!(component, "." | "..")
        || component.ends_with('.')
        || !component
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return false;
    }
    let stem = component.split('.').next().unwrap_or_default();
    let stem = stem.to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !matches!(stem.as_bytes(), [b'C', b'O', b'M', b'1'..=b'9'])
        && !matches!(stem.as_bytes(), [b'L', b'P', b'T', b'1'..=b'9'])
}

fn dense_index(value: &str, prefix: &str) -> Result<i64, DriverProtocolError> {
    let digits = value
        .strip_prefix(prefix)
        .ok_or(DriverProtocolCode::SourceMapReference)?;
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DriverProtocolCode::SourceMapReference.into());
    }
    digits
        .parse::<i64>()
        .ok()
        .filter(|index| *index <= SAFE_INTEGER_MAX)
        .ok_or_else(|| DriverProtocolCode::SourceMapReference.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_limits_and_framing_precede_json() {
        assert_eq!(
            parse_request_transport(b"{}").unwrap_err().code,
            DriverProtocolCode::Transport
        );
        assert_eq!(
            parse_request_transport(b"{}\r\n").unwrap_err().code,
            DriverProtocolCode::Transport
        );
        assert_eq!(
            parse_request_transport(b"{ \"x\": 1 }\n").unwrap_err().code,
            DriverProtocolCode::Canonical
        );
    }
}
