//! Policy scan v1 over the registry-pinned generic frontend.

pub mod v1 {
    pub const USAGE: &str = "mpk policy scan <source-root> --language <go|rust> --semantic-profile <profile> --require-release-registry-id <id> --require-release-registry-sha256 <sha256> --frontend-bundle <id> --toolchain-bundle <id> --target <target> --package <package> --function <function-id> --contract <contract.json> [--contract <contract.json> ...] --json-out <scan.json>";

    use crate::frontend_protocol::AcceptedFrontendArtifacts;
    use crate::frontend_runner::{
        prepare_installed_frontend, run_prepared_frontend, rust_function_id, rust_package_name,
        rust_pointer_width, AcceptedFrontendRun, FrontendReleaseIdentity, FrontendRunRequest,
    };
    use crate::policy_schema::{
        canonical_policy_scan_v1_json, import_policy_scan_v1_json, PolicyGoSelection,
        PolicyGoSemanticParameters, PolicyHelperArtifact, PolicyIssue, PolicyRustSelection,
        PolicyRustSemanticParameters, PolicyScanLinkageContext, PolicyScanV1, PolicySelection,
        PolicySemanticParameters, ValidatedPolicyScanV1, POLICY_SCAN_V1_SCHEMA,
    };
    use mpk_vc::{
        canonical_json_bytes, parse_strict_json, CapturedInput, InputKind, ReleaseSelectionRequest,
        SourceLanguage, StrictJsonLimits,
    };
    use serde::Serialize;
    #[cfg(test)]
    use serde_json::json;
    use serde_json::Value;
    use std::collections::{BTreeMap, BTreeSet};
    use std::error::Error;
    use std::fmt;
    use std::fs::{self, OpenOptions};
    #[cfg(target_os = "linux")]
    use std::io::Read;
    use std::io::{self, Write};
    #[cfg(target_os = "linux")]
    use std::mem::MaybeUninit;
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    #[cfg(target_os = "linux")]
    use rustix::fs::{openat2, Mode, OFlags, RawDir, ResolveFlags, CWD};

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
        "--removed-frontend",
        "--toolchain-root",
        "--toolchain-path",
        "--registry",
        "--registry-path",
        "--release-registry-path",
    ];
    #[cfg(target_os = "linux")]
    const STAGING_FILES_MAX: usize = 65_536;
    #[cfg(target_os = "linux")]
    const STAGING_FILE_BYTES_MAX: u64 = 33_554_432;
    #[cfg(target_os = "linux")]
    const STAGING_TOTAL_BYTES_MAX: u64 = 536_870_912;
    #[cfg(target_os = "linux")]
    const STAGING_DIRECTORIES_MAX: usize = 65_536;
    #[cfg(target_os = "linux")]
    const STAGING_DIRECTORY_ENTRIES_MAX: usize = 262_144;
    const STAGING_PATH_BYTES_MAX: usize = 1_024;
    const GO_AUXILIARY_SUFFIXES: [&str; 19] = [
        ".c", ".cc", ".cpp", ".cxx", ".m", ".h", ".hh", ".hpp", ".hxx", ".f", ".F", ".for", ".f90",
        ".s", ".S", ".sx", ".swig", ".swigcxx", ".syso",
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
                    pointer_width: rust_pointer_width(&self.target_id)
                        .expect("profile validation closes the Rust target set"),
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
    pub(crate) struct OwnedFrontendStaging {
        pub(crate) captured_inputs: Vec<OwnedCapturedInput>,
        pub(crate) staged_directories: Vec<String>,
        pub(crate) staged_placeholders: Vec<String>,
    }

    #[derive(Debug)]
    pub(crate) struct PolicyScanV1RunOutput {
        pub(crate) invocation: PolicyScanV1Invocation,
        pub(crate) scan: ValidatedPolicyScanV1,
        pub(crate) frontend: AcceptedFrontendRun,
        pub(crate) captured_inputs: Vec<OwnedCapturedInput>,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PolicyScanV1Error {
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

        pub const fn code(&self) -> &'static str {
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

    /// Shared CLI prefix used by verify so its evidence-path scalar
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
                    || !rust_package_name(&invocation.package)
                    || !valid_rust_selection(&invocation.function) =>
            {
                return Err(cli_error(
                    "POLICY_CLI_SCALAR",
                    "Rust target, package, or function identity is not canonical",
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
        let compatible = match invocation.source_language.as_str() {
            "go" => {
                invocation.semantic_profile == "mpk.go.fixed.v0"
                    && invocation.target_id == "linux/amd64"
            }
            "rust" => {
                invocation.semantic_profile == "mpk.rust.checked.v0"
                    && rust_pointer_width(&invocation.target_id).is_some()
            }
            _ => unreachable!("source language was validated first"),
        };
        if !compatible {
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
        function
            .split("::")
            .next()
            .is_some_and(|crate_name| rust_function_id(function, crate_name))
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

    /// Runs the released policy-scan command with an immutable snapshot of all
    /// profile-relevant files below the selected source root.
    pub fn run_cli(
        argv: &[String],
        working_directory: &Path,
    ) -> Result<Option<String>, PolicyScanV1Error> {
        let Some(invocation) = parse_policy_scan_v1_argv(argv)? else {
            return Ok(None);
        };
        let prepared =
            prepare_installed_frontend(&invocation.release_request()).map_err(|error| {
                PolicyScanV1Error::new(
                    error.code().as_str(),
                    "generic frontend release preflight failed",
                )
            })?;
        let output_target = preflight_scan_output(working_directory, &invocation.json_out)?;
        let staging = capture_invocation_staging(&invocation, working_directory)?;
        let output = run_prepared_policy_scan_v1(
            invocation,
            output_target,
            staging,
            prepared,
            |prepared, request| {
                run_prepared_frontend(prepared, request).map_err(|error| {
                    PolicyScanV1Error::new(error.code().as_str(), "generic frontend runner failed")
                })
            },
        )?;
        Ok(Some(format!(
            "ok policy scan status={} json={}",
            output.scan.document().readiness,
            output.invocation.json_out
        )))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn capture_invocation_inputs(
        invocation: &PolicyScanV1Invocation,
        working_directory: &Path,
    ) -> Result<Vec<OwnedCapturedInput>, PolicyScanV1Error> {
        Ok(capture_invocation_staging(invocation, working_directory)?.captured_inputs)
    }

    pub(crate) fn capture_invocation_staging(
        invocation: &PolicyScanV1Invocation,
        working_directory: &Path,
    ) -> Result<OwnedFrontendStaging, PolicyScanV1Error> {
        let root = working_directory.join(&invocation.source_root);
        capture_successor_staging(&root, &invocation.source_language, &invocation.contracts)
    }

    /// Captures the immutable source closure shared by the active successor
    /// CLI without discovering a registry or executable path.
    pub(crate) fn capture_successor_staging(
        root: &Path,
        source_language: &str,
        contracts: &[String],
    ) -> Result<OwnedFrontendStaging, PolicyScanV1Error> {
        let contract_paths = contracts.iter().cloned().collect::<BTreeSet<_>>();
        #[cfg(target_os = "linux")]
        let mut staging = capture_linux_tree(root, source_language, &contract_paths)?;
        #[cfg(not(target_os = "linux"))]
        let mut staging = {
            let metadata = fs::symlink_metadata(root)
                .map_err(|error| input_error(format!("source root is unavailable: {error}")))?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(input_error("source root must be a regular directory"));
            }
            let mut inputs = Vec::new();
            let mut staged_directories = Vec::new();
            let mut staged_placeholders = Vec::new();
            capture_directory(
                root,
                root,
                source_language,
                &contract_paths,
                &mut inputs,
                &mut staged_directories,
                &mut staged_placeholders,
            )?;
            OwnedFrontendStaging {
                captured_inputs: inputs,
                staged_directories,
                staged_placeholders,
            }
        };
        staging.captured_inputs.sort_by(|left, right| {
            left.normalized_path
                .as_bytes()
                .cmp(right.normalized_path.as_bytes())
        });
        staging
            .staged_directories
            .sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        staging
            .staged_placeholders
            .sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        validate_owned_staging(&staging)?;
        Ok(staging)
    }

    #[cfg(target_os = "linux")]
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct CaptureStableMetadata {
        device: u64,
        inode: u64,
        mode: u32,
        links: u64,
        size: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
        changed_seconds: i64,
        changed_nanoseconds: i64,
    }

    #[cfg(target_os = "linux")]
    struct CapturePass {
        observed: BTreeMap<String, CaptureObservation>,
        inputs: Vec<OwnedCapturedInput>,
        staged_directories: Vec<String>,
        staged_placeholders: Vec<String>,
    }

    #[cfg(target_os = "linux")]
    #[derive(Clone, Debug, Eq, PartialEq)]
    enum CaptureObservation {
        Stable(CaptureStableMetadata),
        DirectoryKind,
        RegularFileKind,
    }

    #[cfg(target_os = "linux")]
    struct CaptureState<'a> {
        source_language: &'a str,
        contracts: &'a BTreeSet<String>,
        retain_bytes: bool,
        observed: BTreeMap<String, CaptureObservation>,
        inputs: Vec<OwnedCapturedInput>,
        staged_directories: Vec<String>,
        staged_placeholders: Vec<String>,
        staged_identities: BTreeSet<(u64, u64)>,
        directories_visited: usize,
        entries_examined: usize,
        staged_bytes: u64,
    }

    #[cfg(target_os = "linux")]
    struct CaptureDirectoryFrame {
        directory: fs::File,
        relative_directory: String,
        stable_before: Option<CaptureStableMetadata>,
        names: Vec<String>,
        next_name: usize,
    }

    #[cfg(target_os = "linux")]
    fn capture_linux_tree(
        path: &Path,
        source_language: &str,
        contracts: &BTreeSet<String>,
    ) -> Result<OwnedFrontendStaging, PolicyScanV1Error> {
        let descriptor = openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
            ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
        )
        .map_err(|error| input_error(format!("source root is unavailable: {error}")))?;
        let root = fs::File::from(descriptor);
        let metadata = root
            .metadata()
            .map_err(|error| input_error(format!("source root is unavailable: {error}")))?;
        if !metadata.is_dir() {
            return Err(input_error("source root must be a regular directory"));
        }
        let captured = capture_linux_pass(&root, source_language, contracts, true)?;
        let observed_after = capture_linux_pass(&root, source_language, contracts, false)?;
        if captured.observed != observed_after.observed {
            return Err(input_error(
                "source snapshot namespace changed during capture",
            ));
        }
        Ok(OwnedFrontendStaging {
            captured_inputs: captured.inputs,
            staged_directories: captured.staged_directories,
            staged_placeholders: captured.staged_placeholders,
        })
    }

    #[cfg(target_os = "linux")]
    fn capture_linux_pass(
        root: &fs::File,
        source_language: &str,
        contracts: &BTreeSet<String>,
        retain_bytes: bool,
    ) -> Result<CapturePass, PolicyScanV1Error> {
        let root = open_capture_entry(root, ".", true)?;
        let root_metadata = root
            .metadata()
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        let root_stable = capture_stable(&root_metadata);
        let mut state = CaptureState {
            source_language,
            contracts,
            retain_bytes,
            observed: BTreeMap::from([(
                String::new(),
                CaptureObservation::Stable(root_stable.clone()),
            )]),
            inputs: Vec::new(),
            staged_directories: Vec::new(),
            staged_placeholders: Vec::new(),
            staged_identities: BTreeSet::new(),
            directories_visited: 1,
            entries_examined: 0,
            staged_bytes: 0,
        };
        capture_linux_directory(root, root_stable, &mut state)?;
        Ok(CapturePass {
            observed: state.observed,
            inputs: state.inputs,
            staged_directories: state.staged_directories,
            staged_placeholders: state.staged_placeholders,
        })
    }

    #[cfg(target_os = "linux")]
    fn capture_linux_directory(
        root: fs::File,
        root_stable: CaptureStableMetadata,
        state: &mut CaptureState<'_>,
    ) -> Result<(), PolicyScanV1Error> {
        let mut frames = vec![capture_directory_frame(
            root,
            String::new(),
            Some(root_stable),
            state,
        )?];
        while !frames.is_empty() {
            let next = {
                let frame = frames.last_mut().expect("the loop requires one frame");
                if frame.next_name == frame.names.len() {
                    None
                } else {
                    let name = frame.names[frame.next_name].clone();
                    frame.next_name += 1;
                    Some((name, frame.relative_directory.clone()))
                }
            };
            let Some((name, relative_directory)) = next else {
                let frame = frames.pop().expect("the loop requires one frame");
                if let Some(stable_before) = frame.stable_before {
                    let after = frame
                        .directory
                        .metadata()
                        .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
                    if stable_before != capture_stable(&after) {
                        return Err(input_error("source directory changed during capture"));
                    }
                }
                continue;
            };
            let child = capture_linux_entry(
                &frames
                    .last()
                    .expect("the loop requires one frame")
                    .directory,
                &name,
                &relative_directory,
                state,
            )?;
            if let Some((directory, relative, stable_before)) = child {
                frames.push(capture_directory_frame(
                    directory,
                    relative,
                    Some(stable_before),
                    state,
                )?);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn capture_directory_frame(
        directory: fs::File,
        relative_directory: String,
        stable_before: Option<CaptureStableMetadata>,
        state: &mut CaptureState<'_>,
    ) -> Result<CaptureDirectoryFrame, PolicyScanV1Error> {
        let remaining_entries = STAGING_DIRECTORY_ENTRIES_MAX
            .checked_sub(state.entries_examined)
            .ok_or_else(|| input_error("source staging entry limit exceeded"))?;
        let names = capture_directory_names(&directory, remaining_entries)?;
        state.entries_examined = state
            .entries_examined
            .checked_add(names.len())
            .filter(|count| *count <= STAGING_DIRECTORY_ENTRIES_MAX)
            .ok_or_else(|| input_error("source staging entry limit exceeded"))?;
        Ok(CaptureDirectoryFrame {
            directory,
            relative_directory,
            stable_before,
            names,
            next_name: 0,
        })
    }

    #[cfg(target_os = "linux")]
    fn capture_linux_entry(
        directory: &fs::File,
        name: &str,
        relative_directory: &str,
        state: &mut CaptureState<'_>,
    ) -> Result<Option<(fs::File, String, CaptureStableMetadata)>, PolicyScanV1Error> {
        let relative = if relative_directory.is_empty() {
            name.to_owned()
        } else {
            format!("{relative_directory}/{name}")
        };
        if relative.len() > STAGING_PATH_BYTES_MAX {
            return Err(input_error("source staging path limit exceeded"));
        }
        let inspected = inspect_capture_entry(directory, name)?;
        let before = inspected
            .metadata()
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        let stable_before = capture_stable(&before);
        let observation = capture_entry_observation(
            state.source_language,
            state.contracts,
            &relative,
            name,
            &before,
        )?;
        if state
            .observed
            .insert(relative.clone(), observation)
            .is_some()
        {
            return Err(input_error("source snapshot contains duplicate paths"));
        }
        // `stable_before` owns the complete comparison metadata. Release the
        // O_PATH fd before opening a byte-bearing entry or descending.
        drop(inspected);
        if before.is_dir() {
            if state.retain_bytes {
                state.staged_directories.push(relative.clone());
            }
            if state.source_language == "go" && name == ".git" {
                return Ok(None);
            }
            let entry = open_capture_entry(directory, name, true)?;
            let opened = entry
                .metadata()
                .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
            if stable_before != capture_stable(&opened) {
                return Err(input_error("source directory changed during capture"));
            }
            state.directories_visited = state
                .directories_visited
                .checked_add(1)
                .filter(|count| *count <= STAGING_DIRECTORIES_MAX)
                .ok_or_else(|| input_error("source staging directory limit exceeded"))?;
            return Ok(Some((entry, relative, stable_before)));
        }
        debug_assert!(before.is_file());
        let kind = captured_input_kind(state.source_language, state.contracts, &relative);
        if !state.retain_bytes {
            return Ok(None);
        }
        let Some(kind) = kind else {
            state.staged_placeholders.push(relative);
            return Ok(None);
        };
        if state.inputs.len() >= STAGING_FILES_MAX {
            return Err(input_error("source staging file-count limit exceeded"));
        }
        if before.len() > STAGING_FILE_BYTES_MAX {
            return Err(input_error("source staging file-size limit exceeded"));
        }
        state.staged_bytes = state
            .staged_bytes
            .checked_add(before.len())
            .filter(|bytes| *bytes <= STAGING_TOTAL_BYTES_MAX)
            .ok_or_else(|| input_error("source staging total-byte limit exceeded"))?;
        if state.source_language == "rust"
            && !state.staged_identities.insert((before.dev(), before.ino()))
        {
            return Err(input_error("source snapshot contains a hard-link alias"));
        }
        let mut entry = open_capture_entry(directory, name, false)?;
        let opened = entry
            .metadata()
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        if stable_before != capture_stable(&opened) {
            return Err(input_error("source input changed during capture"));
        }
        let mut bytes = Vec::with_capacity(usize::try_from(before.len()).unwrap_or(0));
        Read::by_ref(&mut entry)
            .take(STAGING_FILE_BYTES_MAX + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        let after = entry
            .metadata()
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        if bytes.len() as u64 > STAGING_FILE_BYTES_MAX
            || bytes.len() as u64 != before.len()
            || stable_before != capture_stable(&after)
        {
            return Err(input_error("source input changed during capture"));
        }
        state.inputs.push(OwnedCapturedInput {
            kind,
            normalized_path: relative,
            bytes,
        });
        Ok(None)
    }

    #[cfg(target_os = "linux")]
    fn capture_entry_observation(
        source_language: &str,
        contracts: &BTreeSet<String>,
        relative: &str,
        name: &str,
        metadata: &fs::Metadata,
    ) -> Result<CaptureObservation, PolicyScanV1Error> {
        if metadata.is_dir() {
            return Ok(if source_language == "go" && name == ".git" {
                CaptureObservation::DirectoryKind
            } else {
                CaptureObservation::Stable(capture_stable(metadata))
            });
        }
        if !metadata.is_file() {
            return Err(input_error("source snapshot contains a non-regular file"));
        }
        Ok(
            if captured_input_kind(source_language, contracts, relative).is_some() {
                CaptureObservation::Stable(capture_stable(metadata))
            } else {
                CaptureObservation::RegularFileKind
            },
        )
    }

    #[cfg(target_os = "linux")]
    fn inspect_capture_entry(
        directory: &fs::File,
        name: &str,
    ) -> Result<fs::File, PolicyScanV1Error> {
        let descriptor = openat2(
            directory,
            name,
            OFlags::PATH | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
            capture_inspection_resolve_flags(),
        )
        .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        Ok(fs::File::from(descriptor))
    }

    #[cfg(target_os = "linux")]
    fn open_capture_entry(
        directory: &fs::File,
        name: &str,
        require_directory: bool,
    ) -> Result<fs::File, PolicyScanV1Error> {
        let mut flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
        if require_directory {
            flags |= OFlags::DIRECTORY;
        }
        let descriptor = openat2(
            directory,
            name,
            flags,
            Mode::empty(),
            capture_content_resolve_flags(),
        )
        .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        Ok(fs::File::from(descriptor))
    }

    #[cfg(target_os = "linux")]
    fn capture_inspection_resolve_flags() -> ResolveFlags {
        // Name/kind-only entries may be mountpoints. The O_PATH descriptor
        // cannot expose bytes, and candidates/traversed directories are
        // reopened below with the cross-device boundary enforced.
        ResolveFlags::BENEATH | ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS
    }

    #[cfg(target_os = "linux")]
    fn capture_content_resolve_flags() -> ResolveFlags {
        capture_inspection_resolve_flags() | ResolveFlags::NO_XDEV
    }

    #[cfg(target_os = "linux")]
    fn capture_directory_names(
        directory: &fs::File,
        maximum: usize,
    ) -> Result<Vec<String>, PolicyScanV1Error> {
        let mut storage = [MaybeUninit::uninit(); 64 * 1024];
        let mut reader = RawDir::new(directory, &mut storage);
        let mut names = Vec::new();
        while let Some(entry) = reader.next() {
            let entry =
                entry.map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
            let bytes = entry.file_name().to_bytes();
            if matches!(bytes, b"." | b"..") {
                continue;
            }
            let name = std::str::from_utf8(bytes)
                .map_err(|_| input_error("source snapshot path is not UTF-8"))?;
            if name.is_empty() || name.contains('/') {
                return Err(input_error("source snapshot path is invalid"));
            }
            if names.len() >= maximum {
                return Err(input_error("source staging entry limit exceeded"));
            }
            names.push(name.to_owned());
        }
        names.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if names.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(input_error("source snapshot contains duplicate paths"));
        }
        Ok(names)
    }

    #[cfg(target_os = "linux")]
    fn capture_stable(metadata: &fs::Metadata) -> CaptureStableMetadata {
        CaptureStableMetadata {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            links: metadata.nlink(),
            size: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }

    fn captured_input_kind(
        source_language: &str,
        contracts: &BTreeSet<String>,
        relative: &str,
    ) -> Option<InputKind> {
        let path = Path::new(relative);
        let file_name = path.file_name().and_then(|name| name.to_str());
        match source_language {
            "go" => match file_name {
                Some("go.mod" | "go.work") => Some(InputKind::BuildManifest),
                Some("go.sum" | "go.work.sum") => Some(InputKind::Lockfile),
                Some(name) if name.ends_with("_test.go") => None,
                Some(name) if name.ends_with(".go") => Some(InputKind::Source),
                Some(name)
                    if GO_AUXILIARY_SUFFIXES
                        .iter()
                        .any(|suffix| name.ends_with(suffix)) =>
                {
                    Some(InputKind::Source)
                }
                Some(name) if go_contract_candidate(name) => Some(InputKind::Contract),
                _ => None,
            },
            "rust" => {
                if contracts.contains(relative) {
                    Some(InputKind::Contract)
                } else {
                    match relative {
                        "Cargo.toml" => Some(InputKind::BuildManifest),
                        "Cargo.lock" => Some(InputKind::Lockfile),
                        "rust-toolchain"
                        | "rust-toolchain.toml"
                        | ".cargo/config"
                        | ".cargo/config.toml" => Some(InputKind::BuildManifest),
                        _ => file_name.map(|_| InputKind::Source),
                    }
                }
            }
            "csharp" | "java" => {
                if contracts.contains(relative) {
                    Some(InputKind::Contract)
                } else {
                    file_name
                        .filter(|name| {
                            name.ends_with(if source_language == "java" {
                                ".java"
                            } else {
                                ".cs"
                            })
                        })
                        .map(|_| InputKind::Source)
                }
            }
            _ => None,
        }
    }

    fn go_contract_candidate(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower == "contract.json"
            || lower.ends_with(".contract.json")
            || lower.ends_with("_contract.json")
    }

    #[cfg(not(target_os = "linux"))]
    fn capture_directory(
        root: &Path,
        directory: &Path,
        source_language: &str,
        contracts: &BTreeSet<String>,
        output: &mut Vec<OwnedCapturedInput>,
        staged_directories: &mut Vec<String>,
        staged_placeholders: &mut Vec<String>,
    ) -> Result<(), PolicyScanV1Error> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|_| input_error("source snapshot escaped its root"))?
                .to_str()
                .ok_or_else(|| input_error("source snapshot path is not UTF-8"))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            let file_type = entry
                .file_type()
                .map_err(|error| input_error(format!("source snapshot failed: {error}")))?;
            if file_type.is_symlink() {
                return Err(input_error("source snapshot contains a symbolic link"));
            }
            if file_type.is_dir() {
                staged_directories.push(relative);
                let name = entry.file_name();
                if source_language == "go" && name.to_str() == Some(".git") {
                    continue;
                }
                capture_directory(
                    root,
                    &path,
                    source_language,
                    contracts,
                    output,
                    staged_directories,
                    staged_placeholders,
                )?;
                continue;
            }
            if !file_type.is_file() {
                return Err(input_error("source snapshot contains a non-regular file"));
            }
            let kind = captured_input_kind(source_language, contracts, &relative);
            if let Some(kind) = kind {
                output.push(OwnedCapturedInput {
                    kind,
                    normalized_path: relative,
                    bytes: fs::read(&path)
                        .map_err(|error| input_error(format!("source snapshot failed: {error}")))?,
                });
            } else {
                staged_placeholders.push(relative);
            }
        }
        Ok(())
    }

    fn input_error(detail: impl Into<String>) -> PolicyScanV1Error {
        PolicyScanV1Error::new("POLICY_CLI_INPUT", detail)
    }

    #[cfg(test)]
    pub(crate) fn run_policy_scan_v1_with<P, F, R>(
        argv: &[String],
        working_directory: &Path,
        captured_inputs: Vec<OwnedCapturedInput>,
        mut prepare: F,
        runner: R,
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
        let staging = OwnedFrontendStaging {
            captured_inputs,
            staged_directories: Vec::new(),
            staged_placeholders: Vec::new(),
        };
        run_prepared_policy_scan_v1(invocation, output_target, staging, prepared, runner).map(Some)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn run_policy_scan_v1_with_staging<P, F, R>(
        argv: &[String],
        working_directory: &Path,
        staging: OwnedFrontendStaging,
        mut prepare: F,
        runner: R,
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
        run_prepared_policy_scan_v1(invocation, output_target, staging, prepared, runner).map(Some)
    }

    fn run_prepared_policy_scan_v1<P, R>(
        invocation: PolicyScanV1Invocation,
        output_target: ScanOutputTarget,
        staging: OwnedFrontendStaging,
        prepared: P,
        mut runner: R,
    ) -> Result<PolicyScanV1RunOutput, PolicyScanV1Error>
    where
        R: for<'a> FnMut(
            P,
            FrontendRunRequest<'a>,
        ) -> Result<AcceptedFrontendRun, PolicyScanV1Error>,
    {
        validate_owned_staging(&staging)?;
        let OwnedFrontendStaging {
            captured_inputs,
            staged_directories,
            staged_placeholders,
        } = staging;
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
                release: invocation.release_request(),
                semantic_parameters: &semantic_parameters,
                selection: &selection,
                captured_inputs: &captured_refs,
                staged_directories: &staged_directory_refs,
                staged_placeholders: &staged_placeholder_refs,
                contracts: &invocation.contracts,
            },
        )?;
        let output = build_policy_scan_v1_output(invocation, frontend, captured_inputs)?;
        safe_create_scan(&output_target, output.scan.canonical_bytes())?;
        Ok(output)
    }

    /// Builds the exact validated scan projection from a retained frontend run
    /// without writing a scan artifact. The v1 verify path uses this
    /// after its single frontend launch so it cannot drift into a hidden scan.
    pub(crate) fn build_policy_scan_v1_output(
        invocation: PolicyScanV1Invocation,
        frontend: AcceptedFrontendRun,
        captured_inputs: Vec<OwnedCapturedInput>,
    ) -> Result<PolicyScanV1RunOutput, PolicyScanV1Error> {
        validate_owned_captured_inputs(&captured_inputs)?;
        let captured_inputs = retain_validated_frontend_inputs(&frontend, captured_inputs)?;
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

    fn retain_validated_frontend_inputs(
        frontend: &AcceptedFrontendRun,
        available_inputs: Vec<OwnedCapturedInput>,
    ) -> Result<Vec<OwnedCapturedInput>, PolicyScanV1Error> {
        let Some(artifacts) = frontend.envelope.artifacts.as_ref() else {
            return Ok(available_inputs);
        };
        let mut available_by_path = available_inputs
            .into_iter()
            .map(|input| (input.normalized_path.clone(), input))
            .collect::<BTreeMap<_, _>>();
        let mut captured_inputs =
            Vec::with_capacity(artifacts.source_manifest.manifest().inputs.len());
        for manifest_input in &artifacts.source_manifest.manifest().inputs {
            let input = available_by_path
                .remove(manifest_input.normalized_path.as_str())
                .ok_or_else(|| internal_linkage("validated manifest input is not retained"))?;
            if input.kind != manifest_input.kind {
                return Err(internal_linkage(
                    "validated manifest input kind differs from retained input",
                ));
            }
            captured_inputs.push(input);
        }
        validate_owned_captured_inputs(&captured_inputs)?;
        Ok(captured_inputs)
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

    fn validate_owned_staging(staging: &OwnedFrontendStaging) -> Result<(), PolicyScanV1Error> {
        validate_owned_captured_inputs(&staging.captured_inputs)?;
        let input_paths = staging
            .captured_inputs
            .iter()
            .map(|input| input.normalized_path.as_str())
            .collect::<BTreeSet<_>>();
        let mut namespace_paths = BTreeSet::new();
        for path in staging
            .staged_directories
            .iter()
            .chain(&staging.staged_placeholders)
        {
            if path.is_empty()
                || path.len() > STAGING_PATH_BYTES_MAX
                || !Path::new(path)
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)))
                || input_paths.contains(path.as_str())
                || !namespace_paths.insert(path.as_str())
            {
                return Err(input_error("private staging namespace is not exact"));
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
            let (schema, raw_function) =
                contract_identity(&captured.bytes, manifest.source_language)?;
            let function =
                resolve_contract_function(&artifacts.vir, manifest.source_language, &raw_function)?;
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

    fn contract_identity(
        bytes: &[u8],
        source_language: SourceLanguage,
    ) -> Result<(String, String), PolicyScanV1Error> {
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
        let expected_schema = match source_language {
            SourceLanguage::Go => "mpk.go.contract.v0",
            SourceLanguage::Rust => "mpk.rust.contract.v0",
        };
        if schema != expected_schema {
            return Err(internal_linkage(
                "validated contract schema differs from the source language",
            ));
        }
        let raw_function = object
            .get("function")
            .and_then(Value::as_str)
            .ok_or_else(|| internal_linkage("validated contract function is absent"))?;
        let function = match source_language {
            SourceLanguage::Go => raw_function.trim(),
            SourceLanguage::Rust => raw_function,
        };
        if function.is_empty()
            || (source_language == SourceLanguage::Rust
                && !function
                    .split("::")
                    .next()
                    .is_some_and(|crate_name| rust_function_id(function, crate_name)))
        {
            return Err(internal_linkage(
                "validated contract function is not canonical",
            ));
        }
        Ok((schema.to_owned(), function.to_owned()))
    }

    fn resolve_contract_function<'a>(
        vir: &'a mpk_vc::VirModule,
        source_language: SourceLanguage,
        raw: &str,
    ) -> Result<&'a mpk_vc::VirFunction, PolicyScanV1Error> {
        let mut matches = vir
            .units
            .iter()
            .flat_map(|unit| &unit.functions)
            .filter(|function| {
                function.id == raw
                    || (source_language == SourceLanguage::Go
                        && (function
                            .id
                            .rsplit_once('/')
                            .is_some_and(|(_, suffix)| suffix == raw)
                            || function.id.ends_with(&format!(".{raw}"))))
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
        fn private_java_capture_preserves_source_contract_and_unlisted_inventory() {
            let temporary = tempfile::tempdir().unwrap();
            fs::create_dir_all(temporary.path().join("src/demo")).unwrap();
            fs::create_dir(temporary.path().join("contracts")).unwrap();
            let source = b"package demo; public interface Probe {}\n";
            fs::write(temporary.path().join("src/demo/Probe.java"), source).unwrap();
            fs::write(temporary.path().join("contracts/probe.json"), b"{}\n").unwrap();
            fs::write(
                temporary.path().join("ambient.class"),
                b"never a Java dependency",
            )
            .unwrap();
            let contracts = ["contracts/probe.json".to_owned()];
            let staging = capture_successor_staging(temporary.path(), "java", &contracts).unwrap();
            assert_eq!(staging.captured_inputs.len(), 2);
            assert_eq!(staging.captured_inputs[0].kind, InputKind::Contract);
            assert_eq!(staging.captured_inputs[1].kind, InputKind::Source);
            assert_eq!(staging.captured_inputs[1].bytes, source);
            assert_eq!(staging.staged_placeholders, ["ambient.class"]);
            fs::write(temporary.path().join("src/demo/Probe.java"), b"changed\n").unwrap();
            assert_eq!(staging.captured_inputs[1].bytes, source);
            // The native capture retains the inventory for the child's exact
            // selection gate; unlisted .java files cannot silently disappear.
            fs::write(temporary.path().join("src/demo/Unlisted.java"), source).unwrap();
            assert_eq!(
                capture_successor_staging(temporary.path(), "java", &contracts)
                    .unwrap()
                    .captured_inputs
                    .len(),
                3
            );
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(
                    "Probe.java",
                    temporary.path().join("src/demo/Alias.java"),
                )
                .unwrap();
                assert!(capture_successor_staging(temporary.path(), "java", &contracts).is_err());
                fs::remove_file(temporary.path().join("src/demo/Alias.java")).unwrap();
            }
            #[cfg(target_os = "linux")]
            {
                fs::hard_link(
                    temporary.path().join("src/demo/Probe.java"),
                    temporary.path().join("src/demo/Alias.java"),
                )
                .unwrap();
                assert!(capture_successor_staging(temporary.path(), "java", &contracts).is_err());
            }
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn noncandidate_inspection_uses_a_path_only_descriptor() {
            let temporary = tempfile::tempdir().unwrap();
            fs::write(temporary.path().join("ignored_test.go"), b"not opened\n").unwrap();
            let root = fs::File::open(temporary.path()).unwrap();
            let mut inspected = inspect_capture_entry(&root, "ignored_test.go").unwrap();
            let flags = rustix::fs::fcntl_getfl(&inspected).unwrap();
            assert!(flags.contains(OFlags::PATH));
            assert!(!capture_inspection_resolve_flags().contains(ResolveFlags::NO_XDEV));
            assert!(capture_content_resolve_flags().contains(ResolveFlags::NO_XDEV));
            let mut byte = [0_u8; 1];
            assert!(std::io::Read::read(&mut inspected, &mut byte).is_err());
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn go_name_only_observations_ignore_excluded_bytes_but_retain_name_and_kind() {
            let temporary = tempfile::tempdir().unwrap();
            let contracts = BTreeSet::new();
            let placeholder = temporary.path().join("ignored_test.go");
            fs::write(&placeholder, b"one").unwrap();
            let before = fs::symlink_metadata(&placeholder).unwrap();
            let observed_before = capture_entry_observation(
                "go",
                &contracts,
                "ignored_test.go",
                "ignored_test.go",
                &before,
            )
            .unwrap();
            fs::write(&placeholder, b"excluded bytes are not captured").unwrap();
            let after = fs::symlink_metadata(&placeholder).unwrap();
            let observed_after = capture_entry_observation(
                "go",
                &contracts,
                "ignored_test.go",
                "ignored_test.go",
                &after,
            )
            .unwrap();
            assert_ne!(capture_stable(&before), capture_stable(&after));
            assert_eq!(observed_before, observed_after);

            let original = BTreeMap::from([("ignored_test.go", observed_after.clone())]);
            let renamed = BTreeMap::from([("renamed_test.go", observed_after.clone())]);
            assert_ne!(original, renamed);
            fs::remove_file(&placeholder).unwrap();
            fs::create_dir(&placeholder).unwrap();
            let changed_kind = capture_entry_observation(
                "go",
                &contracts,
                "ignored_test.go",
                "ignored_test.go",
                &fs::symlink_metadata(&placeholder).unwrap(),
            )
            .unwrap();
            assert_ne!(observed_after, changed_kind);

            let candidate = temporary.path().join("main.go");
            fs::write(&candidate, b"a").unwrap();
            let candidate_before = capture_entry_observation(
                "go",
                &contracts,
                "main.go",
                "main.go",
                &fs::symlink_metadata(&candidate).unwrap(),
            )
            .unwrap();
            fs::write(&candidate, b"package main\n").unwrap();
            let candidate_after = capture_entry_observation(
                "go",
                &contracts,
                "main.go",
                "main.go",
                &fs::symlink_metadata(&candidate).unwrap(),
            )
            .unwrap();
            assert_ne!(candidate_before, candidate_after);

            let skipped = temporary.path().join(".git");
            fs::create_dir(&skipped).unwrap();
            let git_before = capture_entry_observation(
                "go",
                &contracts,
                ".git",
                ".git",
                &fs::symlink_metadata(&skipped).unwrap(),
            )
            .unwrap();
            fs::write(skipped.join("index"), b"unobserved internals").unwrap();
            let git_after = capture_entry_observation(
                "go",
                &contracts,
                ".git",
                ".git",
                &fs::symlink_metadata(&skipped).unwrap(),
            )
            .unwrap();
            assert_eq!(git_before, git_after);
        }

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
        fn policy_scan_v1_parser_accepts_both_registered_rust_targets() {
            for (target, pointer_width) in [
                ("i686-unknown-linux-gnu", 32),
                ("x86_64-unknown-linux-gnu", 64),
            ] {
                let parsed = parse_policy_scan_v1_argv(&rust_scan_argv(target))
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(parsed.semantic_parameters()).unwrap(),
                    json!({
                        "target_id":target,
                        "pointer_width":pointer_width,
                        "overflow_mode":"checked",
                        "panic_mode":"abort"
                    })
                );
                assert_eq!(
                    serde_json::to_value(parsed.selection()).unwrap(),
                    json!({
                        "package":"vector-core",
                        "crate":"vector_core",
                        "kind":"lib",
                        "function":"vector_core::identity"
                    })
                );
            }

            for package in ["2vector", "vector.core", "vector/core"] {
                let mut argv = rust_scan_argv("x86_64-unknown-linux-gnu");
                replace_option(&mut argv, "--package", package);
                assert_eq!(
                    parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
                    "POLICY_CLI_SCALAR",
                    "{package}"
                );
            }
            let mut overlong_package = rust_scan_argv("x86_64-unknown-linux-gnu");
            replace_option(&mut overlong_package, "--package", &"a".repeat(1_025));
            assert_eq!(
                parse_policy_scan_v1_argv(&overlong_package)
                    .unwrap_err()
                    .code(),
                "POLICY_CLI_SCALAR"
            );

            for function in [
                "::identity".to_owned(),
                format!("{}::identity", "a".repeat(256)),
                format!("vector::{}", vec!["item"; 205].join("::")),
            ] {
                let mut argv = rust_scan_argv("x86_64-unknown-linux-gnu");
                replace_option(&mut argv, "--function", &function);
                assert_eq!(
                    parse_policy_scan_v1_argv(&argv).unwrap_err().code(),
                    "POLICY_CLI_SCALAR",
                    "{function:?}"
                );
            }
        }

        #[test]
        fn rust_contract_helper_identity_never_applies_go_whitespace_normalization() {
            let rust = br#"{"function":" vector::identity ","schema":"mpk.rust.contract.v0"}"#;
            assert_eq!(
                contract_identity(rust, SourceLanguage::Rust)
                    .unwrap_err()
                    .code(),
                "POLICY_SOURCE_LINKAGE"
            );
            let go = br#"{"function":" Identity ","schema":"mpk.go.contract.v0"}"#;
            assert_eq!(
                contract_identity(go, SourceLanguage::Go).unwrap(),
                ("mpk.go.contract.v0".to_owned(), "Identity".to_owned())
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

        fn rust_scan_argv(target: &str) -> Vec<String> {
            let mut argv = go_scan_argv();
            replace_option(&mut argv, "--language", "rust");
            replace_option(&mut argv, "--semantic-profile", "mpk.rust.checked.v0");
            replace_option(&mut argv, "--frontend-bundle", "frontend.rust.synthetic.v0");
            replace_option(
                &mut argv,
                "--toolchain-bundle",
                "toolchain.rust.synthetic.v0",
            );
            replace_option(&mut argv, "--target", target);
            replace_option(&mut argv, "--package", "vector-core");
            replace_option(&mut argv, "--function", "vector_core::identity");
            argv
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
