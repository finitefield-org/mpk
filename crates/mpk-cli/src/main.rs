#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use mpk_core::Name;
use mpk_kernel::{
    verify_certificate_bytes, verify_certificate_bytes_axiom_report_json_output,
    verify_certificate_bytes_json_output, VerificationJsonOutput, VerificationReport,
};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

const PACKAGE_SCHEMA: &str = "mpk.package.v0";
const TOP_LEVEL_FIELDS: &[&str] = &["schema", "module", "imports", "certificates", "policy"];
const IMPORT_FIELDS: &[&str] = &["module", "export_hash", "certificate_hash"];
const CERTIFICATE_FIELDS: &[&str] = &[
    "module",
    "path",
    "expected_export_hash",
    "expected_axiom_report_hash",
    "expected_certificate_hash",
];
const POLICY_FIELDS: &[&str] = &[
    "checker_profile",
    "allowed_axiom_profiles",
    "require_reference_checker",
    "require_source_free_check",
];
const CHECKER_PROFILES: &[&str] = &["core-bootstrap", "mvp-structural", "mvp-strict"];

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(RunOutcome::Help) => ExitCode::SUCCESS,
        Ok(RunOutcome::Check(output)) => {
            println!("{}", output.json);
            if output.accepted {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Ok(RunOutcome::AxiomReport(output)) => {
            println!("{}", output.json);
            if output.accepted {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Ok(RunOutcome::Verify(message))
        | Ok(RunOutcome::PackageCheck(message))
        | Ok(RunOutcome::PackageVerifyCerts(message)) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
        Err(CliError::Input(message)) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<RunOutcome, CliError> {
    match args.as_slice() {
        [command, path] if command == "check" => check_path(Path::new(path)),
        [command, path] if command == "axiom-report" => axiom_report_path(Path::new(path)),
        [command, path] if command == "verify" => verify_path(Path::new(path)),
        [command, subcommand, path] if command == "package" && subcommand == "check" => {
            package_check_path(Path::new(path))
        }
        [command, subcommand, path] if command == "package" && subcommand == "verify-certs" => {
            package_verify_certs_path(Path::new(path))
        }
        [command] if command == "--help" || command == "-h" || command == "help" => {
            print_usage();
            Ok(RunOutcome::Help)
        }
        _ => Err(CliError::Usage(usage_text())),
    }
}

fn check_path(path: &Path) -> Result<RunOutcome, CliError> {
    let bytes = read_certificate_input(path)?;
    Ok(RunOutcome::Check(verify_certificate_bytes_json_output(
        &bytes,
    )))
}

fn axiom_report_path(path: &Path) -> Result<RunOutcome, CliError> {
    let bytes = read_certificate_input(path)?;
    Ok(RunOutcome::AxiomReport(
        verify_certificate_bytes_axiom_report_json_output(&bytes),
    ))
}

fn verify_path(path: &Path) -> Result<RunOutcome, CliError> {
    let bytes = read_certificate_input(path)?;
    let report = verify_certificate_bytes(&bytes).map_err(|error| {
        CliError::Input(format!(
            "verification failed: {:?}: {}",
            error.kind(),
            error.detail()
        ))
    })?;

    Ok(RunOutcome::Verify(format!(
        "ok module={} declarations={} axioms={}",
        report.module, report.declaration_count, report.axiom_count
    )))
}

fn package_check_path(path: &Path) -> Result<RunOutcome, CliError> {
    let package = validate_package_manifest(path)
        .map_err(|error| CliError::Input(format!("package check failed: {error}")))?;

    Ok(RunOutcome::PackageCheck(format!(
        "ok package={} imports={} certificates={}",
        package.module,
        package.import_count,
        package.certificates.len()
    )))
}

fn package_verify_certs_path(path: &Path) -> Result<RunOutcome, CliError> {
    let package = validate_package_manifest(path)
        .map_err(|error| CliError::Input(format!("package verify-certs failed: {error}")))?;
    let reference_count = if package.policy.require_reference_checker {
        verify_reference_certificates(&package)
            .map_err(|error| CliError::Input(format!("package verify-certs failed: {error}")))?
    } else {
        0
    };

    Ok(RunOutcome::PackageVerifyCerts(format!(
        "ok package={} source_free={} reference={}",
        package.module,
        package.certificates.len(),
        reference_count
    )))
}

fn read_certificate_input(path: &Path) -> Result<Vec<u8>, CliError> {
    let bytes = fs::read(path)
        .map_err(|error| CliError::Input(format!("failed to read {}: {error}", path.display())))?;
    if path.extension().is_some_and(|extension| extension == "hex") {
        decode_hex(&bytes)
    } else {
        Ok(bytes)
    }
}

fn decode_hex(bytes: &[u8]) -> Result<Vec<u8>, CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| CliError::Input(format!("hex fixture is not UTF-8: {error}")))?;
    let hex = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if hex.len() % 2 != 0 {
        return Err(CliError::Input(
            "hex fixture has an odd number of digits".to_owned(),
        ));
    }

    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk)
                .map_err(|error| CliError::Input(format!("hex chunk is not UTF-8: {error}")))?;
            u8::from_str_radix(byte, 16)
                .map_err(|error| CliError::Input(format!("invalid hex byte `{byte}`: {error}")))
        })
        .collect()
}

fn validate_package_manifest(path: &Path) -> Result<PackageValidation, PackageError> {
    let text = fs::read_to_string(path)
        .map_err(|error| PackageError(format!("failed to read {}: {error}", path.display())))?;
    let value = parse_strict_json(&text)
        .map_err(|error| PackageError(format!("{}: invalid JSON: {error}", path.display())))?;
    let manifest = value.as_object().ok_or_else(|| {
        PackageError(format!(
            "{}: manifest must be a JSON object",
            path.display()
        ))
    })?;

    require_exact_fields(manifest, TOP_LEVEL_FIELDS, "manifest")?;
    require_equal(manifest.get("schema"), PACKAGE_SCHEMA, "schema")?;
    let module = require_name(manifest.get("module"), "module")?.to_owned();
    let import_count = validate_imports(manifest.get("imports"))?;
    let certificates = validate_certificates(manifest.get("certificates"))?;
    let policy = validate_policy(manifest.get("policy"))?;

    Ok(PackageValidation {
        module,
        import_count,
        certificates,
        policy,
    })
}

fn parse_strict_json(text: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str::<NoDuplicateValue>(text).map(|value| value.0)
}

fn validate_imports(value: Option<&Value>) -> Result<usize, PackageError> {
    let imports = require_array(value, "imports")?;
    let mut previous_key: Option<(String, String, String)> = None;
    let mut seen_identity = HashSet::new();

    for (index, entry) in imports.iter().enumerate() {
        let field = format!("imports[{index}]");
        let object = require_object(Some(entry), &field)?;
        require_required_fields(object, &["module", "export_hash"], &field)?;
        require_allowed_fields(object, IMPORT_FIELDS, &field)?;

        let module = require_name(object.get("module"), &format!("{field}.module"))?;
        let export_hash = require_hash(object.get("export_hash"), &format!("{field}.export_hash"))?;
        let certificate_hash = match object.get("certificate_hash") {
            Some(value) => require_hash(Some(value), &format!("{field}.certificate_hash"))?,
            None => "",
        };

        let identity = (module.to_owned(), export_hash.to_owned());
        if !seen_identity.insert(identity) {
            return Err(PackageError(format!(
                "{field} duplicates import {module}:{export_hash}"
            )));
        }

        let key = (
            module.to_owned(),
            export_hash.to_owned(),
            certificate_hash.to_owned(),
        );
        if previous_key
            .as_ref()
            .is_some_and(|previous| &key <= previous)
        {
            return Err(PackageError(format!(
                "{field} is not in canonical import order"
            )));
        }
        previous_key = Some(key);
    }

    Ok(imports.len())
}

fn validate_certificates(value: Option<&Value>) -> Result<Vec<VerifiedCertificate>, PackageError> {
    let certificates = require_array(value, "certificates")?;
    if certificates.is_empty() {
        return Err(PackageError(
            "certificates must be a nonempty list".to_owned(),
        ));
    }

    let package_root = std::env::current_dir()
        .map_err(|error| PackageError(format!("failed to resolve package root: {error}")))?;
    let mut seen_paths = HashSet::new();
    let mut verified = Vec::with_capacity(certificates.len());

    for (index, entry) in certificates.iter().enumerate() {
        let field = format!("certificates[{index}]");
        let object = require_object(Some(entry), &field)?;
        require_exact_fields(object, CERTIFICATE_FIELDS, &field)?;

        let module = require_name(object.get("module"), &format!("{field}.module"))?;
        let manifest_path = require_manifest_path(object.get("path"), &format!("{field}.path"))?;
        if !seen_paths.insert(manifest_path.to_owned()) {
            return Err(PackageError(format!(
                "{field}.path duplicates {manifest_path}"
            )));
        }

        let expected_export_hash = require_hash(
            object.get("expected_export_hash"),
            &format!("{field}.expected_export_hash"),
        )?;
        let expected_axiom_report_hash = require_hash(
            object.get("expected_axiom_report_hash"),
            &format!("{field}.expected_axiom_report_hash"),
        )?;
        let expected_certificate_hash = require_hash(
            object.get("expected_certificate_hash"),
            &format!("{field}.expected_certificate_hash"),
        )?;

        let certificate_path = resolve_package_path(&package_root, manifest_path, &field)?;
        let report = verify_manifest_certificate(&certificate_path, &field)?;
        require_report_field(&report.module, module, &format!("{field}.module"))?;
        require_report_field(
            &hash_hex(&report.export_hash),
            expected_export_hash,
            &format!("{field}.expected_export_hash"),
        )?;
        require_report_field(
            &hash_hex(&report.axiom_report_hash),
            expected_axiom_report_hash,
            &format!("{field}.expected_axiom_report_hash"),
        )?;
        require_report_field(
            &hash_hex(&report.certificate_hash),
            expected_certificate_hash,
            &format!("{field}.expected_certificate_hash"),
        )?;

        verified.push(VerifiedCertificate {
            manifest_path: manifest_path.to_owned(),
            resolved_path: certificate_path,
            module: report.module,
            export_hash: hash_hex(&report.export_hash),
            axiom_report_hash: hash_hex(&report.axiom_report_hash),
            certificate_hash: hash_hex(&report.certificate_hash),
        });
    }

    Ok(verified)
}

fn validate_policy(value: Option<&Value>) -> Result<PackagePolicy, PackageError> {
    let policy = require_object(value, "policy")?;
    require_exact_fields(policy, POLICY_FIELDS, "policy")?;

    let checker_profile = require_string(policy.get("checker_profile"), "policy.checker_profile")?;
    if !CHECKER_PROFILES.contains(&checker_profile) {
        return Err(PackageError(format!(
            "policy.checker_profile is unknown: {checker_profile:?}"
        )));
    }

    let axiom_profiles = require_array(
        policy.get("allowed_axiom_profiles"),
        "policy.allowed_axiom_profiles",
    )?;
    if axiom_profiles.is_empty() {
        return Err(PackageError(
            "policy.allowed_axiom_profiles must be a nonempty list".to_owned(),
        ));
    }
    for (index, value) in axiom_profiles.iter().enumerate() {
        let field = format!("policy.allowed_axiom_profiles[{index}]");
        let profile = require_string(Some(value), &field)?;
        if profile.is_empty() {
            return Err(PackageError(format!("{field} must be a nonempty string")));
        }
    }

    let require_reference_checker = require_bool(
        policy.get("require_reference_checker"),
        "policy.require_reference_checker",
    )?;
    if policy.get("require_source_free_check") != Some(&Value::Bool(true)) {
        return Err(PackageError(
            "policy.require_source_free_check must be true".to_owned(),
        ));
    }

    Ok(PackagePolicy {
        require_reference_checker,
    })
}

fn verify_reference_certificates(package: &PackageValidation) -> Result<usize, PackageError> {
    let package_root = std::env::current_dir()
        .map_err(|error| PackageError(format!("failed to resolve package root: {error}")))?;
    let checker_dir = package_root.join("go-tools/mpk-checker-ref");
    if !checker_dir.join("go.mod").is_file() {
        return Err(PackageError(format!(
            "reference checker is required, but {} is missing",
            checker_dir.display()
        )));
    }

    for certificate in &package.certificates {
        verify_reference_certificate(&checker_dir, certificate)?;
    }

    Ok(package.certificates.len())
}

fn verify_reference_certificate(
    checker_dir: &Path,
    certificate: &VerifiedCertificate,
) -> Result<(), PackageError> {
    let output = Command::new("go")
        .current_dir(checker_dir)
        .args(["run", "./cmd/mpk-checker-ref", "verify"])
        .arg(&certificate.resolved_path)
        .output()
        .map_err(|error| {
            PackageError(format!(
                "failed to run reference checker for {}: {error}",
                certificate.manifest_path
            ))
        })?;

    if !output.status.success() {
        return Err(PackageError(format!(
            "reference checker rejected {}: status={} stdout={} stderr={}",
            certificate.manifest_path,
            output.status,
            compact_output(&output.stdout),
            compact_output(&output.stderr)
        )));
    }

    let value = serde_json::from_slice::<Value>(&output.stdout).map_err(|error| {
        PackageError(format!(
            "reference checker output for {} is not valid JSON: {error}",
            certificate.manifest_path
        ))
    })?;
    let field_prefix = format!("reference checker output for {}", certificate.manifest_path);
    let object = require_object(Some(&value), &field_prefix)?;
    require_equal(
        object.get("verdict"),
        "accepted",
        &format!("{field_prefix}.verdict"),
    )?;
    require_report_field(
        require_string(object.get("module"), &format!("{field_prefix}.module"))?,
        &certificate.module,
        &format!("{field_prefix}.module"),
    )?;

    let hashes = require_object(object.get("hashes"), &format!("{field_prefix}.hashes"))?;
    require_report_field(
        require_hash(
            hashes.get("export"),
            &format!("{field_prefix}.hashes.export"),
        )?,
        &certificate.export_hash,
        &format!("{field_prefix}.hashes.export"),
    )?;
    require_report_field(
        require_hash(
            hashes.get("axiom_report"),
            &format!("{field_prefix}.hashes.axiom_report"),
        )?,
        &certificate.axiom_report_hash,
        &format!("{field_prefix}.hashes.axiom_report"),
    )?;
    require_report_field(
        require_hash(
            hashes.get("certificate"),
            &format!("{field_prefix}.hashes.certificate"),
        )?,
        &certificate.certificate_hash,
        &format!("{field_prefix}.hashes.certificate"),
    )?;

    Ok(())
}

fn compact_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn verify_manifest_certificate(
    path: &Path,
    field: &str,
) -> Result<VerificationReport, PackageError> {
    let bytes = read_certificate_input(path)
        .map_err(|error| PackageError(format!("{field}.path: {error}")))?;
    verify_certificate_bytes(&bytes).map_err(|error| {
        PackageError(format!(
            "{field}.path rejected by source-free checker: {:?}: {}",
            error.kind(),
            error.detail()
        ))
    })
}

fn resolve_package_path(
    package_root: &Path,
    manifest_path: &str,
    field: &str,
) -> Result<PathBuf, PackageError> {
    let root = package_root.canonicalize().map_err(|error| {
        PackageError(format!(
            "failed to normalize package root {}: {error}",
            package_root.display()
        ))
    })?;
    let candidate = root.join(manifest_path);
    let normalized = candidate.canonicalize().map_err(|error| {
        PackageError(format!(
            "{field}.path does not exist: {manifest_path}: {error}"
        ))
    })?;
    if !normalized.starts_with(&root) {
        return Err(PackageError(format!(
            "{field}.path escapes the package root"
        )));
    }
    if !normalized.is_file() {
        return Err(PackageError(format!(
            "{field}.path is not a file: {manifest_path}"
        )));
    }
    Ok(normalized)
}

fn require_exact_fields(
    object: &Map<String, Value>,
    expected: &[&str],
    field: &str,
) -> Result<(), PackageError> {
    require_allowed_fields(object, expected, field)?;
    require_required_fields(object, expected, field)
}

fn require_required_fields(
    object: &Map<String, Value>,
    required: &[&str],
    field: &str,
) -> Result<(), PackageError> {
    let missing = required
        .iter()
        .copied()
        .filter(|name| !object.contains_key(*name))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(PackageError(format!(
            "{field} missing required fields: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn require_allowed_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
    field: &str,
) -> Result<(), PackageError> {
    let unknown = object
        .keys()
        .filter(|name| !allowed.contains(&name.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(PackageError(format!(
            "{field} has unknown fields: {}",
            unknown.join(", ")
        )));
    }
    Ok(())
}

fn require_equal(value: Option<&Value>, expected: &str, field: &str) -> Result<(), PackageError> {
    let actual = require_string(value, field)?;
    if actual != expected {
        return Err(PackageError(format!(
            "{field} = {actual:?}, want {expected:?}"
        )));
    }
    Ok(())
}

fn require_object<'a>(
    value: Option<&'a Value>,
    field: &str,
) -> Result<&'a Map<String, Value>, PackageError> {
    value
        .and_then(Value::as_object)
        .ok_or_else(|| PackageError(format!("{field} must be an object")))
}

fn require_array<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a [Value], PackageError> {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| PackageError(format!("{field} must be a list")))
}

fn require_string<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a str, PackageError> {
    value
        .and_then(Value::as_str)
        .ok_or_else(|| PackageError(format!("{field} must be a string")))
}

fn require_bool(value: Option<&Value>, field: &str) -> Result<bool, PackageError> {
    value
        .and_then(Value::as_bool)
        .ok_or_else(|| PackageError(format!("{field} must be boolean")))
}

fn require_name<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a str, PackageError> {
    let name = require_string(value, field)?;
    Name::parse(name).map(|_| name).map_err(|error| {
        PackageError(format!(
            "{field} must be a canonical MPK name: {}",
            error.code()
        ))
    })
}

fn require_hash<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a str, PackageError> {
    let hash = require_string(value, field)?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(PackageError(format!(
            "{field} must be lowercase 64-character hex"
        )));
    }
    if hash.bytes().all(|byte| byte == b'0') {
        return Err(PackageError(format!("{field} must not be all zeroes")));
    }
    Ok(hash)
}

fn require_manifest_path<'a>(
    value: Option<&'a Value>,
    field: &str,
) -> Result<&'a str, PackageError> {
    let path = require_string(value, field)?;
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(PackageError(format!(
            "{field} must be a package-root relative POSIX path"
        )));
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(PackageError(format!(
            "{field} must not contain empty, ., or .. components"
        )));
    }
    if !(path.ends_with(".mpcert") || path.ends_with(".hex")) {
        return Err(PackageError(format!(
            "{field} must point to .mpcert or .hex"
        )));
    }
    Ok(path)
}

fn require_report_field(actual: &str, expected: &str, field: &str) -> Result<(), PackageError> {
    if actual != expected {
        return Err(PackageError(format!(
            "{field} = {actual:?}, want {expected:?}"
        )));
    }
    Ok(())
}

fn hash_hex(hash: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in hash {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn print_usage() {
    println!("{}", usage_text());
}

fn usage_text() -> String {
    "usage: mpk <check|axiom-report|verify> <certificate.mpcert|fixture.hex>\n       mpk package <check|verify-certs> <package-manifest.json>".to_owned()
}

enum RunOutcome {
    Help,
    Check(VerificationJsonOutput),
    AxiomReport(VerificationJsonOutput),
    Verify(String),
    PackageCheck(String),
    PackageVerifyCerts(String),
}

enum CliError {
    Usage(String),
    Input(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Input(message) => formatter.write_str(message),
        }
    }
}

struct PackageValidation {
    module: String,
    import_count: usize,
    certificates: Vec<VerifiedCertificate>,
    policy: PackagePolicy,
}

struct VerifiedCertificate {
    manifest_path: String,
    resolved_path: PathBuf,
    module: String,
    export_hash: String,
    axiom_report_hash: String,
    certificate_hash: String,
}

struct PackagePolicy {
    require_reference_checker: bool,
}

#[derive(Debug)]
struct PackageError(String);

impl fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

struct NoDuplicateValue(Value);

impl<'de> Deserialize<'de> for NoDuplicateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NoDuplicateVisitor)
    }
}

struct NoDuplicateVisitor;

impl<'de> Visitor<'de> for NoDuplicateVisitor {
    type Value = NoDuplicateValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("valid JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(NoDuplicateValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(NoDuplicateValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicateValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        NoDuplicateValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<NoDuplicateValue>()? {
            values.push(value.0);
        }
        Ok(NoDuplicateValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
            }
            let value = map.next_value::<NoDuplicateValue>()?;
            values.insert(key, value.0);
        }
        Ok(NoDuplicateValue(Value::Object(values)))
    }
}
