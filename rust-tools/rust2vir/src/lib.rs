//! Shared, non-installable support for the pinned Rust frontend binaries.

use std::fmt;

pub mod call_closure;
pub mod cargo_check;
pub mod cargo_metadata;
pub mod cli;
pub mod driver_process;
pub mod driver_protocol;
pub mod environment;
pub mod file_loader;
pub mod json;
pub mod manifest;
pub mod metadata_request;
pub mod mir_access;
pub mod module_closure;
pub mod path;
pub mod preflight;
pub mod sandbox;
pub mod session;
pub mod sha256;
pub mod snapshot;
pub mod source_capture;
pub mod source_gate;

pub const PACKAGE_VERSION: &str = "0.1.0";
pub const EXPECTED_RUSTC_RELEASE: &str = "1.89.0-nightly";
pub const EXPECTED_RUSTC_COMMIT: &str = "4d08223c054cf5a56d9761ca925fd46ffebe7115";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilerIdentityError {
    OutputLimit,
    InvalidUtf8,
    MissingRelease,
    DuplicateRelease,
    ReleaseMismatch,
    MissingCommit,
    DuplicateCommit,
    InvalidCommit,
    CommitMismatch,
}

impl fmt::Display for CompilerIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OutputLimit => "RUST_TOOLCHAIN_COMPONENT",
            Self::InvalidUtf8 => "RUST_TOOLCHAIN_COMMIT",
            Self::MissingRelease | Self::DuplicateRelease | Self::ReleaseMismatch => {
                "RUST_TOOLCHAIN_COMPONENT"
            }
            Self::MissingCommit => "RUST_TOOLCHAIN_COMMIT",
            Self::DuplicateCommit => "RUST_TOOLCHAIN_COMMIT",
            Self::InvalidCommit => "RUST_TOOLCHAIN_COMMIT",
            Self::CommitMismatch => "RUST_TOOLCHAIN_COMMIT",
        })
    }
}

pub fn version_line(binary: &str) -> String {
    format!("{binary} {PACKAGE_VERSION} (rustc {EXPECTED_RUSTC_COMMIT})")
}

pub fn validate_rustc_verbose(bytes: &[u8]) -> Result<(), CompilerIdentityError> {
    const MAX_VERBOSE_BYTES: usize = 64 * 1024;
    if bytes.len() > MAX_VERBOSE_BYTES {
        return Err(CompilerIdentityError::OutputLimit);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| CompilerIdentityError::InvalidUtf8)?;
    let mut releases = text.lines().filter_map(|line| line.strip_prefix("rustc "));
    let release = releases
        .next()
        .ok_or(CompilerIdentityError::MissingRelease)?;
    if releases.next().is_some() {
        return Err(CompilerIdentityError::DuplicateRelease);
    }
    if release
        .split_ascii_whitespace()
        .next()
        .filter(|release| *release == EXPECTED_RUSTC_RELEASE)
        .is_none()
    {
        return Err(CompilerIdentityError::ReleaseMismatch);
    }
    let mut commits = text
        .lines()
        .filter_map(|line| line.strip_prefix("commit-hash: "));
    let commit = commits.next().ok_or(CompilerIdentityError::MissingCommit)?;
    if commits.next().is_some() {
        return Err(CompilerIdentityError::DuplicateCommit);
    }
    if commit.len() != 40
        || !commit
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(CompilerIdentityError::InvalidCommit);
    }
    if commit != EXPECTED_RUSTC_COMMIT {
        return Err(CompilerIdentityError::CommitMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_commit_accepts_once() {
        let output = format!(
            "rustc {EXPECTED_RUSTC_RELEASE} (4d08223c0 2025-05-31)\ncommit-hash: {EXPECTED_RUSTC_COMMIT}\n"
        );
        assert_eq!(validate_rustc_verbose(output.as_bytes()), Ok(()));
    }

    #[test]
    fn another_commit_is_refused_before_analysis() {
        let output =
            b"rustc 1.89.0-nightly\ncommit-hash: 0000000000000000000000000000000000000000\n";
        assert_eq!(
            validate_rustc_verbose(output),
            Err(CompilerIdentityError::CommitMismatch)
        );
    }

    #[test]
    fn malformed_or_ambiguous_identity_is_refused() {
        assert_eq!(
            validate_rustc_verbose(b"release: nightly\n"),
            Err(CompilerIdentityError::MissingRelease)
        );
        let duplicate = format!(
            "rustc {EXPECTED_RUSTC_RELEASE}\ncommit-hash: {EXPECTED_RUSTC_COMMIT}\ncommit-hash: {EXPECTED_RUSTC_COMMIT}\n"
        );
        assert_eq!(
            validate_rustc_verbose(duplicate.as_bytes()),
            Err(CompilerIdentityError::DuplicateCommit)
        );
    }
}
