use crate::cli::RustTarget;
use std::collections::BTreeMap;

pub const ENVIRONMENT_PROFILE_ID: &str = "mpk.rust.frontend_environment.v0";
pub const ARGUMENT_PROFILE_ID: &str = "mpk.rust.frontend_arguments.v0";
pub const TARGET_ALLOWLIST_ID: &str = "mpk.rust.targets.v0";

pub const INPUT_ROOT: &str = "/mpk/input";
pub const TOOLCHAIN_ROOT: &str = "/mpk/toolchain";
pub const FRONTEND_ROOT: &str = "/mpk/frontend";
pub const WORK_ROOT: &str = "/mpk/work";
pub const HOME_ROOT: &str = "/mpk/home";
pub const CARGO_HOME_ROOT: &str = "/mpk/cargo-home";
pub const TEMP_ROOT: &str = "/mpk/tmp";
pub const TARGET_ROOT: &str = "/mpk/target";
pub const DRIVER_OUTPUT_ROOT: &str = "/mpk/driver-output";
pub const NATIVE_RUNTIME_ROOT: &str = "/mpk/native-runtime";
pub const CARGO_PATH: &str = "/mpk/toolchain/bin/cargo";
pub const RUSTC_PATH: &str = "/mpk/toolchain/bin/rustc";
pub const DRIVER_PATH: &str = "/mpk/frontend/rust2vir-driver";
pub const INITIAL_LOADER_PATH: &str = "/mpk/toolchain/lib";

pub const ENCODED_RUSTFLAGS_ELEMENTS: [&str; 11] = [
    "-C",
    "overflow-checks=yes",
    "-C",
    "panic=abort",
    "-C",
    "debug-assertions=no",
    "-C",
    "opt-level=0",
    "-Z",
    "mir-opt-level=0",
    "--remap-path-prefix=/mpk/input=.",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceEnvironment {
    entries: BTreeMap<String, String>,
}

impl EvidenceEnvironment {
    pub fn frozen() -> Self {
        let encoded_rustflags = ENCODED_RUSTFLAGS_ELEMENTS.join("\u{1f}");
        let entries = BTreeMap::from([
            ("CARGO_ENCODED_RUSTFLAGS".to_owned(), encoded_rustflags),
            ("CARGO_HOME".to_owned(), CARGO_HOME_ROOT.to_owned()),
            ("CARGO_INCREMENTAL".to_owned(), "0".to_owned()),
            ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
            ("CARGO_TARGET_DIR".to_owned(), TARGET_ROOT.to_owned()),
            ("CARGO_TERM_COLOR".to_owned(), "never".to_owned()),
            ("HOME".to_owned(), HOME_ROOT.to_owned()),
            ("LANG".to_owned(), "C".to_owned()),
            ("LC_ALL".to_owned(), "C".to_owned()),
            ("LD_LIBRARY_PATH".to_owned(), INITIAL_LOADER_PATH.to_owned()),
            ("PATH".to_owned(), "/mpk/toolchain/bin".to_owned()),
            ("RUSTC".to_owned(), RUSTC_PATH.to_owned()),
            ("RUSTC_WORKSPACE_WRAPPER".to_owned(), DRIVER_PATH.to_owned()),
            ("RUST_BACKTRACE".to_owned(), "0".to_owned()),
            ("TERM".to_owned(), "dumb".to_owned()),
            ("TMPDIR".to_owned(), TEMP_ROOT.to_owned()),
            ("TZ".to_owned(), "UTC".to_owned()),
        ]);
        Self { entries }
    }

    pub fn entries(&self) -> &BTreeMap<String, String> {
        &self.entries
    }

    pub fn encoded_rustflags(&self) -> &str {
        self.entries
            .get("CARGO_ENCODED_RUSTFLAGS")
            .expect("the frozen environment always has encoded rustflags")
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        if self != &Self::frozen() {
            return Err(EnvironmentError::ProfileMismatch);
        }
        for (name, value) in &self.entries {
            if name.is_empty()
                || name.contains(['=', '\0'])
                || value.contains('\0')
                || (name != "CARGO_ENCODED_RUSTFLAGS" && value.contains('\u{1f}'))
            {
                return Err(EnvironmentError::InvalidEntry);
            }
        }
        Ok(())
    }
}

impl Default for EvidenceEnvironment {
    fn default() -> Self {
        Self::frozen()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvironmentError {
    ProfileMismatch,
    InvalidEntry,
    LoaderPath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustcChildKind {
    Probe,
    Primary,
}

pub fn expected_child_loader_path<F>(
    kind: RustcChildKind,
    target: RustTarget,
    mut directory_exists: F,
) -> String
where
    F: FnMut(&str) -> bool,
{
    if kind == RustcChildKind::Probe {
        return INITIAL_LOADER_PATH.to_owned();
    }
    let candidates = [
        format!("/mpk/target/{}/debug/deps", target.id()),
        "/mpk/target/debug/deps".to_owned(),
        "/mpk/toolchain/lib/rustlib/x86_64-unknown-linux-gnu/lib".to_owned(),
    ];
    candidates
        .iter()
        .filter(|path| directory_exists(path))
        .map(String::as_str)
        .chain(std::iter::once(INITIAL_LOADER_PATH))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn validate_child_loader_path<F>(
    actual: &str,
    kind: RustcChildKind,
    target: RustTarget,
    directory_exists: F,
) -> Result<(), EnvironmentError>
where
    F: FnMut(&str) -> bool,
{
    if actual.is_empty()
        || actual.split(':').any(str::is_empty)
        || actual != expected_child_loader_path(kind, target, directory_exists)
    {
        return Err(EnvironmentError::LoaderPath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_environment_is_closed_and_uses_real_unit_separators() {
        let environment = EvidenceEnvironment::frozen();
        environment.validate().unwrap();
        assert_eq!(environment.entries().len(), 17);
        assert!(!environment.entries().contains_key("RUSTFLAGS"));
        assert!(!environment.entries().contains_key("RUSTC_WRAPPER"));
        assert!(!environment.entries().contains_key("RUSTC_BOOTSTRAP"));
        assert_eq!(
            environment
                .encoded_rustflags()
                .split('\u{1f}')
                .collect::<Vec<_>>(),
            ENCODED_RUSTFLAGS_ELEMENTS
        );
    }

    #[test]
    fn cargo_child_loader_order_is_exact_and_omits_missing_candidates() {
        let existing = [
            "/mpk/target/x86_64-unknown-linux-gnu/debug/deps",
            "/mpk/toolchain/lib/rustlib/x86_64-unknown-linux-gnu/lib",
        ];
        let expected = concat!(
            "/mpk/target/x86_64-unknown-linux-gnu/debug/deps:",
            "/mpk/toolchain/lib/rustlib/x86_64-unknown-linux-gnu/lib:",
            "/mpk/toolchain/lib"
        );
        assert_eq!(
            expected_child_loader_path(
                RustcChildKind::Primary,
                RustTarget::X86_64UnknownLinuxGnu,
                |path| existing.contains(&path),
            ),
            expected
        );
        assert_eq!(
            validate_child_loader_path(
                &format!("{expected}:"),
                RustcChildKind::Primary,
                RustTarget::X86_64UnknownLinuxGnu,
                |path| existing.contains(&path),
            ),
            Err(EnvironmentError::LoaderPath)
        );
    }
}
