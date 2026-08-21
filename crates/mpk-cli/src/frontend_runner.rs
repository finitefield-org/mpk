use crate::frontend_protocol::{
    validate_frontend_process, AcceptedFrontendEnvelope, FrontendProcessFacts,
    FrontendProtocolCode, FrontendProtocolError, FrontendProtocolRequest,
};
use crate::frontend_registry::{
    FrontendReleaseCode, InstalledReleaseResolver, SelectedFrontendRelease,
};
use crate::frontend_sandbox::{launch_release_frontend, SandboxError};
use mpk_vc::{CapturedInput, ReleaseSelectionRequest};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub(crate) struct FrontendRunRequest<'a> {
    pub(crate) release: ReleaseSelectionRequest,
    pub(crate) semantic_parameters: &'a Value,
    pub(crate) selection: &'a Value,
    pub(crate) captured_inputs: &'a [CapturedInput<'a>],
    pub(crate) args: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrontendRunCode {
    Release(FrontendReleaseCode),
    ProcessSpawn,
    ProcessKilled,
    Protocol(FrontendProtocolCode),
}

#[derive(Debug)]
pub(crate) struct FrontendRunError {
    code: FrontendRunCode,
}

impl FrontendRunError {
    pub(crate) const fn code(&self) -> FrontendRunCode {
        self.code
    }
}

impl fmt::Display for FrontendRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            FrontendRunCode::Release(code) => formatter.write_str(code.as_str()),
            FrontendRunCode::ProcessSpawn => formatter.write_str("FRONTEND_PROCESS_SPAWN"),
            FrontendRunCode::ProcessKilled => formatter.write_str("FRONTEND_PROCESS_KILLED"),
            FrontendRunCode::Protocol(code) => formatter.write_str(code.as_str()),
        }
    }
}

impl Error for FrontendRunError {}

/// Staged generic entry point. It is intentionally not wired into `mpk` command
/// dispatch until GO-VIR-02-T12.
pub(crate) fn run_installed_frontend(
    request: FrontendRunRequest<'_>,
) -> Result<AcceptedFrontendEnvelope, FrontendRunError> {
    let resolver = InstalledReleaseResolver::open().map_err(|error| FrontendRunError {
        code: FrontendRunCode::Release(error.code()),
    })?;
    let selected = resolver
        .resolve(&request.release)
        .map_err(|error| FrontendRunError {
            code: FrontendRunCode::Release(error.code()),
        })?;
    run_selected(selected, request)
}

fn run_selected(
    selected: SelectedFrontendRelease,
    request: FrontendRunRequest<'_>,
) -> Result<AcceptedFrontendEnvelope, FrontendRunError> {
    let environment = registered_environment(&selected, &request.release)?;
    let output = launch_release_frontend(
        &selected.frontend_snapshot,
        &selected.toolchain_snapshot,
        &selected.frontend.main.path,
        &request.args,
        &environment,
        request.captured_inputs,
    )
    .map_err(|error| FrontendRunError {
        code: match error {
            SandboxError::Unavailable => {
                FrontendRunCode::Release(FrontendReleaseCode::SandboxUnavailable)
            }
            SandboxError::Spawn => FrontendRunCode::ProcessSpawn,
            SandboxError::Killed => FrontendRunCode::ProcessKilled,
        },
    })?;
    if output.stream_limit_exceeded {
        return Err(FrontendRunError {
            code: FrontendRunCode::Protocol(FrontendProtocolCode::ProtocolLimit),
        });
    }
    validate_frontend_process(
        FrontendProtocolRequest {
            source_language: &request.release.source_language,
            semantic_profile: &request.release.semantic_profile,
            semantic_parameters: request.semantic_parameters,
            selection: request.selection,
            release_registry: Some(&selected.registry),
            captured_inputs: request.captured_inputs,
        },
        FrontendProcessFacts {
            exit_code: output.exit_code,
            signaled: output.signaled,
            stdout: &output.stdout,
            stderr_observed_bytes: output.stderr_observed_bytes,
        },
    )
    .map_err(protocol_error)
}

fn registered_environment(
    selected: &SelectedFrontendRelease,
    request: &ReleaseSelectionRequest,
) -> Result<BTreeMap<String, String>, FrontendRunError> {
    if selected.frontend.source_language != "go"
        || selected.frontend.environment_profile_id != "mpk.go.frontend_environment.v0"
        || request.target_id != "linux/amd64"
    {
        return Err(FrontendRunError {
            code: FrontendRunCode::Release(FrontendReleaseCode::BundleIncompatible),
        });
    }
    Ok([
        ("CGO_ENABLED", "0"),
        ("GO111MODULE", "on"),
        ("GOAMD64", "v1"),
        ("GOARCH", "amd64"),
        ("GOCACHE", "/mpk/cache/go-build"),
        ("GODEBUG", ""),
        ("GOENV", "off"),
        ("GOEXPERIMENT", ""),
        ("GOFLAGS", ""),
        ("GOMAXPROCS", "1"),
        ("GOMODCACHE", "/mpk/cache/go-mod"),
        ("GONOPROXY", ""),
        ("GONOSUMDB", ""),
        ("GOOS", "linux"),
        ("GOPACKAGESDRIVER", "off"),
        ("GOPATH", "/mpk/gopath"),
        ("GOPRIVATE", ""),
        ("GOPROXY", "off"),
        ("GOROOT", "/mpk/toolchain/go"),
        ("GOSUMDB", "off"),
        ("GOTELEMETRY", "off"),
        ("GOTOOLCHAIN", "local"),
        ("GOVCS", "*:off"),
        ("GOWORK", "off"),
        ("HOME", "/mpk/empty/home"),
        ("LANG", "C"),
        ("LC_ALL", "C"),
        ("PATH", "/mpk/toolchain/go/bin"),
        ("TMPDIR", "/mpk/tmp"),
        ("TZ", "UTC"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect())
}

fn protocol_error(error: FrontendProtocolError) -> FrontendRunError {
    FrontendRunError {
        code: FrontendRunCode::Protocol(error.code()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend_registry::SnapshotFile;
    use crate::frontend_sandbox::launch_snapshot_for_test;
    use sha2::{Digest, Sha256};

    #[test]
    fn validated_executable_bytes_are_snapshotted_before_launch() {
        let original = b"#!/bin/sh\nprintf '%s' snapshot-original\n".to_vec();
        let digest = Sha256::digest(&original)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let snapshot = SnapshotFile {
            bytes: original,
            executable: true,
            sha256: digest,
        };
        let mut replaced_path_bytes = b"#!/bin/sh\nprintf '%s' replacement\n".to_vec();
        replaced_path_bytes.fill(b'x');
        let output =
            launch_snapshot_for_test(&snapshot, &[], &BTreeMap::new()).expect("snapshot launches");
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.stdout, b"snapshot-original");
    }
}
