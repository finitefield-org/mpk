mod common;

use common::{frozen_environment, metadata_json, Fixture};
use rust2vir_internal::cargo_metadata::{MetadataCode, MetadataPhase};
use rust2vir_internal::environment::{EvidenceEnvironment, ENCODED_RUSTFLAGS_ELEMENTS};
use rust2vir_internal::sandbox::{
    fixed_read_only_views, fixed_writable_views, CargoInvocation, ProcessOutput, SandboxContext,
    SandboxError, SandboxExecutor, OPEN_FILE_LIMIT, OUTPUT_FILES_LIMIT, PROCESS_LIMIT,
    RESIDENT_MEMORY_LIMIT, STDERR_BYTES_LIMIT, STDOUT_BYTES_LIMIT, TARGET_BYTES_LIMIT,
    TEMP_BYTES_LIMIT, VIRTUAL_MEMORY_LIMIT,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct SharedExecutor {
    calls: Arc<Mutex<usize>>,
    responses: Arc<Mutex<Vec<Result<ProcessOutput, SandboxError>>>>,
}

impl SandboxExecutor for SharedExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        _invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        assert_eq!(context.environment().entries(), &frozen_environment());
        *self.calls.lock().unwrap() += 1;
        self.responses.lock().unwrap().remove(0)
    }
}

#[test]
fn environment_views_and_limits_are_the_exact_closed_profiles() {
    let environment = EvidenceEnvironment::frozen();
    environment.validate().unwrap();
    assert_eq!(environment.entries(), &frozen_environment());
    assert_eq!(
        environment
            .encoded_rustflags()
            .split('\u{1f}')
            .collect::<Vec<_>>(),
        ENCODED_RUSTFLAGS_ELEMENTS
    );
    assert_eq!(
        fixed_read_only_views(),
        [
            "/mpk/input",
            "/mpk/toolchain",
            "/mpk/frontend",
            "/mpk/work",
            "/mpk/native-runtime",
            "/mpk/driver-request.json",
        ]
    );
    assert_eq!(
        fixed_writable_views(),
        [
            "/mpk/home",
            "/mpk/cargo-home",
            "/mpk/tmp",
            "/mpk/target",
            "/mpk/driver-output",
        ]
    );
    assert_eq!(PROCESS_LIMIT, 256);
    assert_eq!(OPEN_FILE_LIMIT, 1_024);
    assert_eq!(VIRTUAL_MEMORY_LIMIT, 17_179_869_184);
    assert_eq!(RESIDENT_MEMORY_LIMIT, 8_589_934_592);
    assert_eq!(TEMP_BYTES_LIMIT, 4_294_967_296);
    assert_eq!(TARGET_BYTES_LIMIT, 17_179_869_184);
    assert_eq!(OUTPUT_FILES_LIMIT, 262_144);
    assert_eq!(STDOUT_BYTES_LIMIT, 67_108_864);
    assert_eq!(STDERR_BYTES_LIMIT, 2_097_152);
}

#[test]
fn sandbox_unavailable_is_not_retried_with_a_second_response() {
    let calls = Arc::new(Mutex::new(0));
    let executor = SharedExecutor {
        calls: Arc::clone(&calls),
        responses: Arc::new(Mutex::new(vec![
            Err(SandboxError::SandboxUnavailable),
            Ok(ProcessOutput::success(metadata_json())),
        ])),
    };
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let error = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .err()
    .expect("sandbox failure must be terminal");
    assert_eq!(
        error.code,
        MetadataCode::Sandbox(SandboxError::SandboxUnavailable)
    );
    assert_eq!(*calls.lock().unwrap(), 1);
}

#[test]
fn every_process_rehashes_candidate_files_before_executor_entry() {
    let calls = Arc::new(Mutex::new(0));
    let executor = SharedExecutor {
        calls: Arc::clone(&calls),
        responses: Arc::new(Mutex::new(vec![Ok(
            ProcessOutput::success(metadata_json()),
        )])),
    };
    let mut fixture = Fixture::new();
    let rustc = fixture.toolchain_root().join("bin/rustc");
    let metadata_request = fixture.metadata_request();
    let phase = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap();

    fs::set_permissions(&rustc, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(&rustc, b"replaced compiler").unwrap();
    fs::set_permissions(&rustc, fs::Permissions::from_mode(0o555)).unwrap();
    let error = phase
        .run()
        .err()
        .expect("candidate replacement must reject");
    assert_eq!(
        error.code,
        MetadataCode::Sandbox(SandboxError::ToolchainComponent)
    );
    assert_eq!(*calls.lock().unwrap(), 0);
}

#[test]
fn every_process_rejects_noncanonical_candidate_modes() {
    let calls = Arc::new(Mutex::new(0));
    let executor = SharedExecutor {
        calls: Arc::clone(&calls),
        responses: Arc::new(Mutex::new(vec![Ok(
            ProcessOutput::success(metadata_json()),
        )])),
    };
    let mut fixture = Fixture::new();
    let rustc = fixture.toolchain_root().join("bin/rustc");
    let metadata_request = fixture.metadata_request();
    let phase = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap();

    fs::set_permissions(&rustc, fs::Permissions::from_mode(0o500)).unwrap();
    let error = phase
        .run()
        .err()
        .expect("noncanonical installed mode must reject");
    assert_eq!(
        error.code,
        MetadataCode::Sandbox(SandboxError::ToolchainComponent)
    );
    assert_eq!(*calls.lock().unwrap(), 0);
}

#[test]
fn launcher_assertion_mismatch_rejects_before_sandbox_or_process_creation() {
    let calls = Arc::new(Mutex::new(0));
    let executor = SharedExecutor {
        calls: Arc::clone(&calls),
        responses: Arc::new(Mutex::new(vec![Ok(
            ProcessOutput::success(metadata_json()),
        )])),
    };
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let mut request = fixture.request().clone();
    request.release.driver_sha256 =
        "0000000000000000000000000000000000000000000000000000000000000000".to_owned();
    let error = MetadataPhase::prepare(
        &request,
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .err()
    .expect("launcher assertion mismatch must reject");
    assert_eq!(
        error.code,
        MetadataCode::Sandbox(SandboxError::ToolchainComponent)
    );
    assert_eq!(*calls.lock().unwrap(), 0);
}

#[test]
fn private_bootstrap_rejects_incomplete_state_without_running_a_child() {
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .arg("__rust2vir_cargo_sandbox_v0")
        .arg("/untrusted")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(125));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
