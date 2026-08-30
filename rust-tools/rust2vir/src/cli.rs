use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

pub const SEMANTIC_PROFILE: &str = "mpk.rust.checked.v0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustSelection {
    pub package: String,
    pub crate_name: String,
    pub kind: &'static str,
    pub function: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustTarget {
    I686UnknownLinuxGnu,
    X86_64UnknownLinuxGnu,
}

impl RustTarget {
    pub fn id(self) -> &'static str {
        match self {
            Self::I686UnknownLinuxGnu => "i686-unknown-linux-gnu",
            Self::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
        }
    }

    pub fn pointer_width(self) -> u8 {
        match self {
            Self::I686UnknownLinuxGnu => 32,
            Self::X86_64UnknownLinuxGnu => 64,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseArguments {
    pub frontend_bundle_id: String,
    pub frontend_sha256: String,
    pub release_registry_id: String,
    pub release_registry_sha256: String,
    pub toolchain_bundle_id: String,
    pub toolchain_root: PathBuf,
    pub toolchain_distribution_sha256: String,
    pub driver: PathBuf,
    pub driver_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowerRequest {
    pub source_root: PathBuf,
    pub selection: RustSelection,
    pub semantic_profile: &'static str,
    pub target: RustTarget,
    pub release: ReleaseArguments,
    pub contracts: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonSuccessStatus {
    Rejected,
    SourceError,
    FrontendError,
}

impl NonSuccessStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::SourceError => "source-error",
            Self::FrontendError => "frontend-error",
        }
    }
}

pub fn non_success_envelope(
    request: &LowerRequest,
    status: NonSuccessStatus,
    code: &str,
    message: &str,
) -> String {
    non_success_envelope_at_phase(request, status, "capture", code, message)
}

pub fn non_success_envelope_at_phase(
    request: &LowerRequest,
    status: NonSuccessStatus,
    phase: &str,
    code: &str,
    message: &str,
) -> String {
    let envelope = crate::json::JsonValue::Object(BTreeMap::from([
        (
            "diagnostics".to_owned(),
            crate::json::JsonValue::Array(vec![crate::json::JsonValue::Object(BTreeMap::from([
                (
                    "code".to_owned(),
                    crate::json::JsonValue::String(code.to_owned()),
                ),
                (
                    "message".to_owned(),
                    crate::json::JsonValue::String(message.to_owned()),
                ),
            ]))]),
        ),
        (
            "phase".to_owned(),
            crate::json::JsonValue::String(phase.to_owned()),
        ),
        (
            "rejected_features".to_owned(),
            crate::json::JsonValue::Array(Vec::new()),
        ),
        (
            "schema".to_owned(),
            crate::json::JsonValue::String("mpk.frontend.cli.v1".to_owned()),
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
        (
            "status".to_owned(),
            crate::json::JsonValue::String(status.as_str().to_owned()),
        ),
    ]));
    String::from_utf8(crate::json::canonical(&envelope).expect("constructed JSON"))
        .expect("canonical JSON is UTF-8")
}

pub fn parse_lower_args<I>(arguments: I) -> Result<LowerRequest, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.into_string().map_err(|_| CliError))
        .collect::<Result<Vec<_>, _>>()?;
    let mut cursor = arguments.into_iter();
    if cursor.next().as_deref() != Some("lower") {
        return Err(CliError);
    }
    let source_root = cursor.next().ok_or(CliError)?;
    if source_root.is_empty() || source_root.starts_with("--") {
        return Err(CliError);
    }

    let mut singleton = BTreeMap::<String, String>::new();
    let mut contracts = Vec::new();
    while let Some(option) = cursor.next() {
        if !is_known_option(&option) {
            return Err(CliError);
        }
        let value = cursor.next().ok_or(CliError)?;
        if value.is_empty() {
            return Err(CliError);
        }
        if option == "--contract" {
            contracts.push(value);
        } else if singleton.insert(option, value).is_some() {
            return Err(CliError);
        }
    }

    let manifest_path = take(&mut singleton, "--manifest-path")?;
    if manifest_path != "Cargo.toml" {
        return Err(CliError);
    }
    let package = take(&mut singleton, "--package")?;
    if !is_package_name(&package) {
        return Err(CliError);
    }
    let semantic_profile = take(&mut singleton, "--semantic-profile")?;
    if semantic_profile != SEMANTIC_PROFILE {
        return Err(CliError);
    }
    let target = match take(&mut singleton, "--target")?.as_str() {
        "i686-unknown-linux-gnu" => RustTarget::I686UnknownLinuxGnu,
        "x86_64-unknown-linux-gnu" => RustTarget::X86_64UnknownLinuxGnu,
        _ => return Err(CliError),
    };
    let function = take(&mut singleton, "--function")?;
    let crate_name = validate_function(&function)?;

    if take_identifier(&mut singleton, "--profile-registry-id")?
        != crate::successor::PROFILE_REGISTRY_ID
        || take(&mut singleton, "--profile-registry-revision")?
            != crate::successor::PROFILE_REGISTRY_REVISION.to_string()
        || take_sha256(&mut singleton, "--profile-registry-sha256")?
            != crate::successor::PROFILE_REGISTRY_SHA256
        || take_sha256(&mut singleton, "--profile-entry-sha256")?
            != crate::successor::PROFILE_ENTRY_SHA256
    {
        return Err(CliError);
    }
    let frontend_bundle_id = take_identifier(&mut singleton, "--frontend-bundle-id")?;
    let frontend_sha256 = take_sha256(&mut singleton, "--frontend-sha256")?;
    let release_registry_id = take_identifier(&mut singleton, "--release-registry-id")?;
    let release_registry_sha256 = take_sha256(&mut singleton, "--release-registry-sha256")?;
    let toolchain_bundle_id = take_identifier(&mut singleton, "--toolchain-bundle-id")?;
    let toolchain_root = take_absolute_path(&mut singleton, "--toolchain-root")?;
    let toolchain_distribution_sha256 =
        take_sha256(&mut singleton, "--toolchain-distribution-sha256")?;
    let driver = take_absolute_path(&mut singleton, "--driver")?;
    let driver_sha256 = take_sha256(&mut singleton, "--driver-sha256")?;
    if !singleton.is_empty() || contracts.is_empty() {
        return Err(CliError);
    }

    Ok(LowerRequest {
        source_root: PathBuf::from(source_root),
        selection: RustSelection {
            package,
            crate_name,
            kind: "lib",
            function,
        },
        semantic_profile: SEMANTIC_PROFILE,
        target,
        release: ReleaseArguments {
            frontend_bundle_id,
            frontend_sha256,
            release_registry_id,
            release_registry_sha256,
            toolchain_bundle_id,
            toolchain_root,
            toolchain_distribution_sha256,
            driver,
            driver_sha256,
        },
        contracts,
    })
}

fn is_known_option(value: &str) -> bool {
    matches!(
        value,
        "--manifest-path"
            | "--package"
            | "--semantic-profile"
            | "--target"
            | "--function"
            | "--profile-registry-id"
            | "--profile-registry-revision"
            | "--profile-registry-sha256"
            | "--profile-entry-sha256"
            | "--frontend-bundle-id"
            | "--frontend-sha256"
            | "--release-registry-id"
            | "--release-registry-sha256"
            | "--toolchain-bundle-id"
            | "--toolchain-root"
            | "--toolchain-distribution-sha256"
            | "--driver"
            | "--driver-sha256"
            | "--contract"
    )
}

fn take(values: &mut BTreeMap<String, String>, option: &str) -> Result<String, CliError> {
    values.remove(option).ok_or(CliError)
}

fn take_identifier(
    values: &mut BTreeMap<String, String>,
    option: &str,
) -> Result<String, CliError> {
    let value = take(values, option)?;
    if !is_release_identifier(&value) {
        return Err(CliError);
    }
    Ok(value)
}

fn take_sha256(values: &mut BTreeMap<String, String>, option: &str) -> Result<String, CliError> {
    let value = take(values, option)?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CliError);
    }
    Ok(value)
}

fn take_absolute_path(
    values: &mut BTreeMap<String, String>,
    option: &str,
) -> Result<PathBuf, CliError> {
    let path = PathBuf::from(take(values, option)?);
    if !path.is_absolute() {
        return Err(CliError);
    }
    Ok(path)
}

fn is_release_identifier(value: &str) -> bool {
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

fn is_package_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn validate_function(value: &str) -> Result<String, CliError> {
    let segments = value.split("::").collect::<Vec<_>>();
    if segments.len() < 2 || segments.iter().any(|segment| !is_rust_identifier(segment)) {
        return Err(CliError);
    }
    Ok(segments[0].to_owned())
}

fn is_rust_identifier(value: &str) -> bool {
    if value == "_" || !value.is_ascii() {
        return false;
    }
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_identifier_uses_the_frozen_grammar() {
        assert!(is_release_identifier("frontend.rust-rust2vir_v0"));
        for value in ["", ".x", "x.", "x..y", "Upper", "x/y"] {
            assert!(!is_release_identifier(value), "{value}");
        }
    }
}
