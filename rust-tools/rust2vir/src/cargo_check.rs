use crate::cargo_metadata::ValidatedCargoMetadata;
use crate::driver_protocol::{DriverOutput, DriverProtocolCode, DriverStatus};
use crate::environment::INPUT_ROOT;
use crate::json::{self, JsonValue};
use crate::manifest::ExpectedManifestSelection;
use crate::sandbox::{
    CargoInvocation, CargoInvocationKind, PreparedSandbox, SandboxError, SandboxExecutor,
};
use std::collections::BTreeMap;

const MESSAGE_LINE_BYTES_MAX: usize = 4 * 1024 * 1024;
const COMPILER_MESSAGES_MAX: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerMessageLevel {
    Error,
    Warning,
    FailureNote,
    Note,
    Help,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedCompilerMessage {
    level: CompilerMessageLevel,
    code: Option<String>,
}

impl NormalizedCompilerMessage {
    pub fn level(&self) -> CompilerMessageLevel {
        self.level
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoCheckOutput {
    succeeded: bool,
    artifact_count: usize,
    messages: Vec<NormalizedCompilerMessage>,
}

impl CargoCheckOutput {
    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    pub fn artifact_count(&self) -> usize {
        self.artifact_count
    }

    pub fn messages(&self) -> &[NormalizedCompilerMessage] {
        &self.messages
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoCheckCode {
    Sandbox(SandboxError),
    Process,
    Protocol,
    DiagnosticBudget,
}

impl CargoCheckCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox(error) => error.code(),
            Self::Process => "RUST_FRONTEND_COMPILER_CRASH",
            Self::Protocol => "RUST_FRONTEND_COMPILER_CRASH",
            Self::DiagnosticBudget => "RUST_FRONTEND_DIAGNOSTIC_BUDGET",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CargoCheckError {
    pub code: CargoCheckCode,
}

impl From<CargoCheckCode> for CargoCheckError {
    fn from(code: CargoCheckCode) -> Self {
        Self { code }
    }
}

impl From<SandboxError> for CargoCheckError {
    fn from(error: SandboxError) -> Self {
        Self {
            code: CargoCheckCode::Sandbox(error),
        }
    }
}

pub struct CheckPhase<'a, E> {
    sandbox: PreparedSandbox<'a, E>,
    expected: ExpectedManifestSelection,
    metadata: ValidatedCargoMetadata,
}

#[derive(Debug)]
pub struct DriverHandshake {
    cargo: Option<CargoCheckOutput>,
    driver: Option<DriverOutput>,
    local_frontend_error: Option<DriverProtocolCode>,
}

impl DriverHandshake {
    pub fn cargo(&self) -> Option<&CargoCheckOutput> {
        self.cargo.as_ref()
    }

    pub fn driver(&self) -> Option<&DriverOutput> {
        self.driver.as_ref()
    }

    pub fn local_frontend_error(&self) -> Option<DriverProtocolCode> {
        self.local_frontend_error
    }
}

impl<'a, E: SandboxExecutor> CheckPhase<'a, E> {
    pub(crate) fn from_validated_metadata(
        sandbox: PreparedSandbox<'a, E>,
        expected: ExpectedManifestSelection,
        metadata: ValidatedCargoMetadata,
    ) -> Self {
        Self {
            sandbox,
            expected,
            metadata,
        }
    }

    pub fn arguments(&self) -> Vec<String> {
        [
            "check".to_owned(),
            "--lib".to_owned(),
            "--package".to_owned(),
            self.expected.package_name().to_owned(),
            "--target".to_owned(),
            self.sandbox_target().to_owned(),
            "--manifest-path".to_owned(),
            "/mpk/input/Cargo.toml".to_owned(),
            "--locked".to_owned(),
            "--offline".to_owned(),
            "--no-default-features".to_owned(),
            "--jobs".to_owned(),
            "1".to_owned(),
            "--message-format".to_owned(),
            "json".to_owned(),
            "--color".to_owned(),
            "never".to_owned(),
        ]
        .into()
    }

    pub fn run(mut self) -> Result<(CargoCheckOutput, E), CargoCheckError> {
        let arguments = self.arguments();
        let references = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        let invocation = CargoInvocation::new(CargoInvocationKind::Check, &references)?;
        let process = self.sandbox.execute(&invocation)?;
        if process.signaled || process.exit_code.is_none() {
            return Err(CargoCheckCode::Process.into());
        }
        let parsed = parse_message_stream(&process.stdout, &self.metadata, &self.expected)?;
        if (process.exit_code == Some(0)) != parsed.succeeded {
            return Err(CargoCheckCode::Protocol.into());
        }
        let executor = self.sandbox.into_executor();
        Ok((parsed, executor))
    }

    pub fn run_driver_handshake(mut self) -> Result<(DriverHandshake, E), CargoCheckError> {
        let arguments = self.arguments();
        let references = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        let invocation = CargoInvocation::new(CargoInvocationKind::Check, &references)?;
        let process = self.sandbox.execute(&invocation)?;
        if process.signaled || process.exit_code.is_none() {
            let executor = self.sandbox.into_executor();
            return Ok((
                DriverHandshake {
                    cargo: None,
                    driver: None,
                    local_frontend_error: Some(DriverProtocolCode::Process),
                },
                executor,
            ));
        }
        let parsed = parse_message_stream(&process.stdout, &self.metadata, &self.expected)?;
        if (process.exit_code == Some(0)) != parsed.succeeded {
            return Err(CargoCheckCode::Protocol.into());
        }
        let driver = self.sandbox.consume_driver_output_artifact();
        let (driver, local_frontend_error) = match driver {
            Ok(driver)
                if (parsed.succeeded && driver.status() == DriverStatus::Lowered)
                    || (!parsed.succeeded && driver.status() != DriverStatus::Lowered) =>
            {
                (Some(driver), None)
            }
            Ok(driver) if !parsed.succeeded && driver.status() == DriverStatus::Lowered => {
                (None, Some(DriverProtocolCode::Count))
            }
            Ok(_) => (None, Some(DriverProtocolCode::Identity)),
            Err(error)
                if parsed.succeeded
                    && error.code == DriverProtocolCode::Filesystem
                    && self.sandbox.driver_output_is_empty() == Ok(true) =>
            {
                (None, Some(DriverProtocolCode::Count))
            }
            Err(error) => (None, Some(error.code)),
        };
        let executor = self.sandbox.into_executor();
        Ok((
            DriverHandshake {
                cargo: Some(parsed),
                driver,
                local_frontend_error,
            },
            executor,
        ))
    }

    fn sandbox_target(&self) -> &'static str {
        // MetadataPhase checked that the request and manifest selection agree; the target is
        // retained by the same prepared sandbox and cannot be replaced between phases.
        self.sandbox.target_id()
    }
}

fn parse_message_stream(
    bytes: &[u8],
    metadata: &ValidatedCargoMetadata,
    expected: &ExpectedManifestSelection,
) -> Result<CargoCheckOutput, CargoCheckError> {
    if bytes.is_empty() || bytes.last() != Some(&b'\n') || bytes.contains(&b'\r') {
        return Err(CargoCheckCode::Protocol.into());
    }
    let mut artifact_count = 0_usize;
    let mut messages = Vec::new();
    let mut build_finished = None;
    for (index, line) in bytes[..bytes.len() - 1]
        .split(|byte| *byte == b'\n')
        .enumerate()
    {
        if line.is_empty() || line.len() > MESSAGE_LINE_BYTES_MAX || build_finished.is_some() {
            return Err(CargoCheckCode::Protocol.into());
        }
        let value =
            json::parse(line, MESSAGE_LINE_BYTES_MAX).map_err(|_| CargoCheckCode::Protocol)?;
        let object = value.as_object().ok_or(CargoCheckCode::Protocol)?;
        match string(object, "reason")? {
            "compiler-artifact" => {
                validate_artifact(object, metadata, expected)?;
                artifact_count = artifact_count
                    .checked_add(1)
                    .ok_or(CargoCheckCode::Protocol)?;
            }
            "compiler-message" => {
                if messages.len() == COMPILER_MESSAGES_MAX {
                    return Err(CargoCheckCode::DiagnosticBudget.into());
                }
                messages.push(normalize_message(object, metadata, expected)?);
            }
            "build-finished" => {
                if index == 0 {
                    return Err(CargoCheckCode::Protocol.into());
                }
                closed(object, &["reason", "success"])?;
                build_finished = Some(
                    required(object, "success")?
                        .as_bool()
                        .ok_or(CargoCheckCode::Protocol)?,
                );
            }
            _ => return Err(CargoCheckCode::Protocol.into()),
        }
    }
    let succeeded = build_finished.ok_or(CargoCheckCode::Protocol)?;
    if (succeeded && artifact_count != 1) || (!succeeded && artifact_count > 1) {
        return Err(CargoCheckCode::Protocol.into());
    }
    let has_error = messages
        .iter()
        .any(|message| message.level == CompilerMessageLevel::Error);
    if succeeded == has_error {
        return Err(CargoCheckCode::Protocol.into());
    }
    Ok(CargoCheckOutput {
        succeeded,
        artifact_count,
        messages,
    })
}

fn validate_artifact(
    object: &BTreeMap<String, JsonValue>,
    metadata: &ValidatedCargoMetadata,
    expected: &ExpectedManifestSelection,
) -> Result<(), CargoCheckError> {
    closed(
        object,
        &[
            "executable",
            "features",
            "filenames",
            "fresh",
            "manifest_path",
            "package_id",
            "profile",
            "reason",
            "target",
        ],
    )?;
    validate_common_identity(object, metadata, expected)?;
    if !required(object, "executable")?.is_null()
        || required(object, "fresh")?.as_bool() != Some(false)
        || !required(object, "features")?
            .as_array()
            .is_some_and(|features| features.is_empty())
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    let filenames = required(object, "filenames")?
        .as_array()
        .ok_or(CargoCheckCode::Protocol)?;
    if filenames.is_empty()
        || filenames.iter().any(|value| {
            value
                .as_str()
                .is_none_or(|path| !is_normalized_target_output(path))
        })
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    let profile = required(object, "profile")?
        .as_object()
        .ok_or(CargoCheckCode::Protocol)?;
    closed(
        profile,
        &[
            "debug_assertions",
            "debuginfo",
            "opt_level",
            "overflow_checks",
            "test",
        ],
    )?;
    if string(profile, "opt_level")? != "0"
        || required(profile, "debug_assertions")?.as_bool() != Some(true)
        || required(profile, "overflow_checks")?.as_bool() != Some(true)
        || required(profile, "test")?.as_bool() != Some(false)
        || required(profile, "debuginfo")?.integer() != Some(2)
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    Ok(())
}

fn normalize_message(
    object: &BTreeMap<String, JsonValue>,
    metadata: &ValidatedCargoMetadata,
    expected: &ExpectedManifestSelection,
) -> Result<NormalizedCompilerMessage, CargoCheckError> {
    closed(
        object,
        &["manifest_path", "message", "package_id", "reason", "target"],
    )?;
    validate_common_identity(object, metadata, expected)?;
    let message = required(object, "message")?
        .as_object()
        .ok_or(CargoCheckCode::Protocol)?;
    closed(
        message,
        &[
            "$message_type",
            "children",
            "code",
            "level",
            "message",
            "rendered",
            "spans",
        ],
    )?;
    if required(message, "children")?.as_array().is_none()
        || required(message, "spans")?.as_array().is_none()
        || string(message, "$message_type")? != "diagnostic"
        || string(message, "message")?.contains('\0')
        || !matches!(required(message, "rendered")?, value if value.is_null() || value.as_str().is_some())
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    let level = match string(message, "level")? {
        "error" => CompilerMessageLevel::Error,
        "warning" => CompilerMessageLevel::Warning,
        "failure-note" => CompilerMessageLevel::FailureNote,
        "note" => CompilerMessageLevel::Note,
        "help" => CompilerMessageLevel::Help,
        _ => return Err(CargoCheckCode::Protocol.into()),
    };
    let code = match required(message, "code")? {
        value if value.is_null() => None,
        value => {
            let code_object = value.as_object().ok_or(CargoCheckCode::Protocol)?;
            closed(code_object, &["code", "explanation"])?;
            if !matches!(required(code_object, "explanation")?, value if value.is_null() || value.as_str().is_some())
            {
                return Err(CargoCheckCode::Protocol.into());
            }
            let code = string(code_object, "code")?;
            if code.is_empty()
                || code.len() > 128
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            {
                return Err(CargoCheckCode::Protocol.into());
            }
            Some(code.to_owned())
        }
    };
    Ok(NormalizedCompilerMessage { level, code })
}

fn is_normalized_target_output(path: &str) -> bool {
    let Some(relative) = path.strip_prefix("/mpk/target/") else {
        return false;
    };
    !relative.is_empty()
        && !relative.contains(['\\', '\0', '\n', '\r'])
        && relative
            .split('/')
            .all(|component| !component.is_empty() && !matches!(component, "." | ".."))
}

fn validate_common_identity(
    object: &BTreeMap<String, JsonValue>,
    metadata: &ValidatedCargoMetadata,
    expected: &ExpectedManifestSelection,
) -> Result<(), CargoCheckError> {
    if string(object, "package_id")? != metadata.package_id()
        || string(object, "manifest_path")? != metadata.manifest_path()
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    let target = required(object, "target")?
        .as_object()
        .ok_or(CargoCheckCode::Protocol)?;
    closed_optional(
        target,
        &[
            "crate_types",
            "doc",
            "doctest",
            "edition",
            "kind",
            "name",
            "required-features",
            "src_path",
            "test",
        ],
        &["required-features"],
    )?;
    if string_array(required(target, "kind")?)? != ["lib"]
        || string_array(required(target, "crate_types")?)? != ["lib"]
        || string(target, "name")? != expected.crate_name()
        || string(target, "edition")? != "2021"
        || string(target, "src_path")?
            != format!("{INPUT_ROOT}/{}", expected.library_path().as_str())
        || required(target, "doc")?.as_bool() != Some(true)
        || required(target, "doctest")?.as_bool() != Some(true)
        || required(target, "test")?.as_bool() != Some(true)
        || target.get("required-features").is_some_and(|value| {
            !value
                .as_array()
                .is_some_and(|required_features| required_features.is_empty())
        })
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    Ok(())
}

fn required<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a JsonValue, CargoCheckError> {
    object
        .get(name)
        .ok_or_else(|| CargoCheckCode::Protocol.into())
}

fn string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a str, CargoCheckError> {
    required(object, name)?
        .as_str()
        .ok_or_else(|| CargoCheckCode::Protocol.into())
}

fn string_array(value: &JsonValue) -> Result<Vec<&str>, CargoCheckError> {
    value
        .as_array()
        .ok_or(CargoCheckCode::Protocol)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| CargoCheckCode::Protocol.into())
        })
        .collect()
}

fn closed(object: &BTreeMap<String, JsonValue>, names: &[&str]) -> Result<(), CargoCheckError> {
    closed_optional(object, names, &[])
}

fn closed_optional(
    object: &BTreeMap<String, JsonValue>,
    names: &[&str],
    optional: &[&str],
) -> Result<(), CargoCheckError> {
    if object.keys().any(|name| !names.contains(&name.as_str()))
        || names
            .iter()
            .any(|name| !optional.contains(name) && !object.contains_key(*name))
    {
        return Err(CargoCheckCode::Protocol.into());
    }
    Ok(())
}
