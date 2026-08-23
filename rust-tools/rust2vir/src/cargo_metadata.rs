use crate::cargo_check::CheckPhase;
use crate::cli::LowerRequest;
use crate::json::{self, JsonValue};
use crate::manifest::ExpectedManifestSelection;
use crate::metadata_request::MetadataRequest;
use crate::sandbox::{
    CargoInvocation, CargoInvocationKind, InjectedCandidate, PreparedSandbox, SandboxError,
    SandboxExecutor,
};
use crate::snapshot::Snapshot;
use std::collections::BTreeMap;
use std::path::Path;

const METADATA_JSON_BYTES_MAX: usize = 8 * 1024 * 1024;
const SNAPSHOT_ROOT: &str = "/mpk/input";
const TARGET_DIRECTORY: &str = "/mpk/target";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedCargoMetadata {
    package_id: String,
    package_name: String,
    package_version: String,
    crate_name: String,
    manifest_path: &'static str,
}

impl ValidatedCargoMetadata {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn package_version(&self) -> &str {
        &self.package_version
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn manifest_path(&self) -> &'static str {
        self.manifest_path
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataStatus {
    Rejected,
    FrontendError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataCode {
    Sandbox(SandboxError),
    Process,
    Protocol,
    Mismatch,
}

impl MetadataCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox(error) => error.code(),
            Self::Process => "RUST_FRONTEND_METADATA_PROCESS",
            Self::Protocol => "RUST_FRONTEND_METADATA_PROTOCOL",
            Self::Mismatch => "RUST_PREFLIGHT_METADATA_MISMATCH",
        }
    }

    pub fn status(self) -> MetadataStatus {
        match self {
            Self::Mismatch => MetadataStatus::Rejected,
            Self::Sandbox(_) | Self::Process | Self::Protocol => MetadataStatus::FrontendError,
        }
    }

    pub fn phase(self) -> &'static str {
        match self {
            Self::Sandbox(SandboxError::DriverProtocol(_)) => "typecheck",
            Self::Sandbox(_) | Self::Process | Self::Protocol | Self::Mismatch => "metadata",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataError {
    pub code: MetadataCode,
}

impl From<MetadataCode> for MetadataError {
    fn from(code: MetadataCode) -> Self {
        Self { code }
    }
}

impl From<SandboxError> for MetadataError {
    fn from(error: SandboxError) -> Self {
        Self {
            code: MetadataCode::Sandbox(error),
        }
    }
}

pub struct MetadataPhase<'a, E> {
    sandbox: PreparedSandbox<'a, E>,
    expected: ExpectedManifestSelection,
    arguments: Vec<&'static str>,
}

impl<'a, E: SandboxExecutor> MetadataPhase<'a, E> {
    pub fn prepare(
        request: &'a LowerRequest,
        snapshot: &'a Snapshot,
        metadata_request: MetadataRequest,
        candidate: &'a InjectedCandidate,
        private_parent: &Path,
        executor: E,
    ) -> Result<Self, MetadataError> {
        if metadata_request.expected().package_name() != request.selection.package
            || metadata_request.expected().crate_name() != request.selection.crate_name
        {
            return Err(MetadataCode::Mismatch.into());
        }
        let expected = metadata_request.expected().clone();
        let arguments = metadata_request.arguments().to_vec();
        let sandbox =
            PreparedSandbox::prepare(request, snapshot, candidate, private_parent, executor)?;
        Ok(Self {
            sandbox,
            expected,
            arguments,
        })
    }

    pub fn run(mut self) -> Result<(ValidatedCargoMetadata, CheckPhase<'a, E>), MetadataError> {
        let invocation = CargoInvocation::new(CargoInvocationKind::Metadata, &self.arguments)?;
        let output = self.sandbox.execute(&invocation)?;
        if !output.succeeded() || output.stderr_observed_bytes != 0 {
            return Err(MetadataCode::Process.into());
        }
        let metadata = parse_and_validate(&output.stdout, &self.expected)?;
        let check =
            CheckPhase::from_validated_metadata(self.sandbox, self.expected, metadata.clone());
        Ok((metadata, check))
    }
}

fn parse_and_validate(
    bytes: &[u8],
    expected: &ExpectedManifestSelection,
) -> Result<ValidatedCargoMetadata, MetadataError> {
    let value = json::parse(bytes, METADATA_JSON_BYTES_MAX).map_err(|_| MetadataCode::Protocol)?;
    let root = object(&value)?;
    closed(
        root,
        &[
            "metadata",
            "packages",
            "resolve",
            "target_directory",
            "version",
            "workspace_default_members",
            "workspace_members",
            "workspace_root",
        ],
    )?;
    if integer(root, "version")? != 1
        || string(root, "target_directory")? != TARGET_DIRECTORY
        || string(root, "workspace_root")? != SNAPSHOT_ROOT
        || !required(root, "resolve")?.is_null()
        || !null_or_empty_object(required(root, "metadata")?)
    {
        return Err(MetadataCode::Mismatch.into());
    }

    let packages = array(root, "packages")?;
    if packages.len() != 1 {
        return Err(MetadataCode::Mismatch.into());
    }
    let package = object(&packages[0])?;
    closed(
        package,
        &[
            "authors",
            "categories",
            "default_run",
            "dependencies",
            "description",
            "documentation",
            "edition",
            "features",
            "homepage",
            "id",
            "keywords",
            "license",
            "license_file",
            "links",
            "manifest_path",
            "metadata",
            "name",
            "publish",
            "readme",
            "repository",
            "rust_version",
            "source",
            "targets",
            "version",
        ],
    )?;
    if string(package, "name")? != expected.package_name()
        || string(package, "version")? != expected.package_version()
        || string(package, "edition")? != expected.edition()
        || string(package, "manifest_path")? != "/mpk/input/Cargo.toml"
        || !required(package, "source")?.is_null()
        || !required(package, "links")?.is_null()
        || !array(package, "dependencies")?.is_empty()
        || !valid_features(required(package, "features")?)
        || !null_or_empty_object(required(package, "metadata")?)
    {
        return Err(MetadataCode::Mismatch.into());
    }
    for name in ["authors", "categories", "keywords"] {
        if required(package, name)?.as_array().is_none() {
            return Err(MetadataCode::Protocol.into());
        }
    }
    for name in [
        "default_run",
        "description",
        "documentation",
        "homepage",
        "license",
        "license_file",
        "readme",
        "repository",
        "rust_version",
    ] {
        if !null_or_string(required(package, name)?) {
            return Err(MetadataCode::Protocol.into());
        }
    }
    let publish = required(package, "publish")?;
    if !(publish.is_null() || publish.as_array().is_some_and(|values| values.is_empty())) {
        return Err(MetadataCode::Mismatch.into());
    }

    let package_id = string(package, "id")?;
    let long_package_id = format!(
        "path+file:///mpk/input#{}@{}",
        expected.package_name(),
        expected.package_version()
    );
    let short_package_id = format!("path+file:///mpk/input#{}", expected.package_version());
    if package_id != long_package_id && package_id != short_package_id {
        return Err(MetadataCode::Mismatch.into());
    }
    let members = array(root, "workspace_members")?;
    let default_members = array(root, "workspace_default_members")?;
    if !single_string_equals(members, package_id)
        || !single_string_equals(default_members, package_id)
    {
        return Err(MetadataCode::Mismatch.into());
    }

    let targets = array(package, "targets")?;
    let mut matching_library = 0_usize;
    for target in targets {
        let target = object(target)?;
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
        let kinds = string_array(required(target, "kind")?)?;
        let crate_types = string_array(required(target, "crate_types")?)?;
        if kinds
            .iter()
            .any(|kind| *kind == "custom-build" || *kind == "proc-macro")
            || crate_types
                .iter()
                .any(|kind| matches!(*kind, "proc-macro" | "dylib" | "cdylib" | "staticlib"))
        {
            return Err(MetadataCode::Mismatch.into());
        }
        if string(target, "edition")? != "2021"
            || required(target, "doc")?.as_bool().is_none()
            || required(target, "doctest")?.as_bool().is_none()
            || required(target, "test")?.as_bool().is_none()
            || target
                .get("required-features")
                .is_some_and(|value| value.as_array().is_none())
        {
            return Err(MetadataCode::Protocol.into());
        }
        let source = string(target, "src_path")?;
        if !is_normalized_input_path(source) {
            return Err(MetadataCode::Mismatch.into());
        }
        if kinds == ["lib"] && crate_types == ["lib"] {
            if string(target, "name")? != expected.crate_name()
                || source != format!("/mpk/input/{}", expected.library_path().as_str())
                || required(target, "doc")?.as_bool() != Some(true)
                || required(target, "doctest")?.as_bool() != Some(true)
                || required(target, "test")?.as_bool() != Some(true)
                || target.get("required-features").is_some_and(|value| {
                    !value.as_array().is_some_and(|features| features.is_empty())
                })
            {
                return Err(MetadataCode::Mismatch.into());
            }
            matching_library += 1;
        } else if kinds.len() != 1
            || !matches!(kinds[0], "bin" | "example" | "test" | "bench")
            || crate_types != ["bin"]
        {
            return Err(MetadataCode::Mismatch.into());
        }
    }
    if matching_library != 1 {
        return Err(MetadataCode::Mismatch.into());
    }

    Ok(ValidatedCargoMetadata {
        package_id: package_id.to_owned(),
        package_name: expected.package_name().to_owned(),
        package_version: expected.package_version().to_owned(),
        crate_name: expected.crate_name().to_owned(),
        manifest_path: "/mpk/input/Cargo.toml",
    })
}

fn object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, MetadataError> {
    value
        .as_object()
        .ok_or_else(|| MetadataCode::Protocol.into())
}

fn required<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a JsonValue, MetadataError> {
    object
        .get(name)
        .ok_or_else(|| MetadataCode::Protocol.into())
}

fn string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a str, MetadataError> {
    required(object, name)?
        .as_str()
        .ok_or_else(|| MetadataCode::Protocol.into())
}

fn integer(object: &BTreeMap<String, JsonValue>, name: &str) -> Result<i64, MetadataError> {
    required(object, name)?
        .integer()
        .ok_or_else(|| MetadataCode::Protocol.into())
}

fn array<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a [JsonValue], MetadataError> {
    required(object, name)?
        .as_array()
        .ok_or_else(|| MetadataCode::Protocol.into())
}

fn closed(object: &BTreeMap<String, JsonValue>, names: &[&str]) -> Result<(), MetadataError> {
    closed_optional(object, names, &[])
}

fn closed_optional(
    object: &BTreeMap<String, JsonValue>,
    names: &[&str],
    optional: &[&str],
) -> Result<(), MetadataError> {
    if object.keys().any(|name| !names.contains(&name.as_str()))
        || names
            .iter()
            .any(|name| !optional.contains(name) && !object.contains_key(*name))
    {
        return Err(MetadataCode::Protocol.into());
    }
    Ok(())
}

fn null_or_empty_object(value: &JsonValue) -> bool {
    value.is_null() || value.as_object().is_some_and(BTreeMap::is_empty)
}

fn null_or_string(value: &JsonValue) -> bool {
    value.is_null() || value.as_str().is_some()
}

fn string_array(value: &JsonValue) -> Result<Vec<&str>, MetadataError> {
    value
        .as_array()
        .ok_or(MetadataCode::Protocol)?
        .iter()
        .map(|value| value.as_str().ok_or_else(|| MetadataCode::Protocol.into()))
        .collect()
}

fn single_string_equals(values: &[JsonValue], expected: &str) -> bool {
    values.len() == 1 && values[0].as_str() == Some(expected)
}

fn valid_features(value: &JsonValue) -> bool {
    let Some(features) = value.as_object() else {
        return false;
    };
    features.is_empty()
        || (features.len() == 1
            && features
                .get("default")
                .and_then(JsonValue::as_array)
                .is_some_and(|values| values.is_empty()))
}

fn is_normalized_input_path(path: &str) -> bool {
    let Some(relative) = path.strip_prefix("/mpk/input/") else {
        return false;
    };
    !relative.is_empty()
        && !relative.contains(['\\', '\0', '\n', '\r'])
        && relative
            .split('/')
            .all(|component| !component.is_empty() && !matches!(component, "." | ".."))
}
