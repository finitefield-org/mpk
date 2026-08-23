mod common;

use common::{
    failed_check_stream_with_codes, failed_process_output, metadata_json, successful_check_stream,
    Fixture, RecordingExecutor,
};
use rust2vir_internal::cargo_check::{CargoCheckCode, CompilerMessageLevel, RustSourceCode};
use rust2vir_internal::cargo_metadata::MetadataPhase;
use rust2vir_internal::driver_process::{publish_result, read_request};
use rust2vir_internal::driver_protocol::{
    encode_non_success, parse_request_transport, DriverProtocolCode, DriverStatus,
    PrivateDiagnostic,
};
use rust2vir_internal::environment::DRIVER_OUTPUT_ROOT;
use rust2vir_internal::sandbox::{
    CargoInvocation, CargoInvocationKind, ProcessOutput, SandboxContext, SandboxError,
    SandboxExecutor, STDERR_BYTES_LIMIT, STDOUT_BYTES_LIMIT,
};

#[derive(Default)]
struct HandshakeExecutor;

impl SandboxExecutor for HandshakeExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        if invocation.kind() == CargoInvocationKind::Metadata {
            return Ok(ProcessOutput::success(metadata_json()));
        }
        let request =
            parse_request_transport(&read_request(context.driver_request_host_path()).unwrap())
                .unwrap();
        let bytes = encode_non_success(
            &request,
            DriverStatus::SourceError,
            "typecheck",
            &[PrivateDiagnostic {
                code: "RUST_SOURCE_TYPE".to_owned(),
                message: "expression type does not match the declared result".to_owned(),
                function_id: Some("vector::identity".to_owned()),
            }],
        )
        .unwrap();
        publish_result(
            context.writable_host_path(DRIVER_OUTPUT_ROOT).unwrap(),
            &bytes,
        )
        .unwrap();
        Ok(common::failed_process_output())
    }
}

#[derive(Default)]
struct PrimaryArgumentFailureExecutor;

impl SandboxExecutor for PrimaryArgumentFailureExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        if invocation.kind() == CargoInvocationKind::Metadata {
            return Ok(ProcessOutput::success(metadata_json()));
        }
        let request =
            parse_request_transport(&read_request(context.driver_request_host_path()).unwrap())
                .unwrap();
        let bytes = encode_non_success(
            &request,
            DriverStatus::FrontendError,
            "typecheck",
            &[PrivateDiagnostic {
                code: "RUST_TOOLCHAIN_ARGUMENT".to_owned(),
                message: "selected rustc arguments differ from the pinned profile".to_owned(),
                function_id: Some("vector::identity".to_owned()),
            }],
        )
        .unwrap();
        publish_result(
            context.writable_host_path(DRIVER_OUTPUT_ROOT).unwrap(),
            &bytes,
        )
        .unwrap();
        Ok(ProcessOutput {
            exit_code: Some(101),
            signaled: false,
            stdout: b"{\"reason\":\"build-finished\",\"success\":false}\n".to_vec(),
            stderr_observed_bytes: 0,
            stdout_limit_exceeded: false,
            stderr_limit_exceeded: false,
        })
    }
}

#[test]
fn check_uses_the_exact_package_target_command_and_same_sandbox_state() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(ProcessOutput::success(successful_check_stream())),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    assert_eq!(
        check.arguments(),
        [
            "check",
            "--lib",
            "--package",
            "vector",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--manifest-path",
            "/mpk/input/Cargo.toml",
            "--locked",
            "--offline",
            "--no-default-features",
            "--jobs",
            "1",
            "--message-format",
            "json",
            "--color",
            "never",
        ]
    );
    let (result, executor) = check.run().unwrap();
    assert!(result.succeeded());
    assert_eq!(result.source_error_code(), None);
    assert_eq!(result.artifact_count(), 1);
    assert!(result.messages().is_empty());
    assert_eq!(executor.calls[1].kind, CargoInvocationKind::Check);
    assert_eq!(
        executor.calls[0].invocation_id,
        executor.calls[1].invocation_id
    );
    assert_eq!(executor.calls[0].environment, executor.calls[1].environment);
    assert_eq!(
        executor.calls[0].snapshot_root,
        executor.calls[1].snapshot_root
    );
    assert_eq!(executor.calls[0].writable, executor.calls[1].writable);
    assert_eq!(executor.calls[0].limits.stdout_bytes, STDOUT_BYTES_LIMIT);
    assert_eq!(executor.calls[0].limits.stderr_bytes, STDERR_BYTES_LIMIT);
    let mut remaining = executor.calls[0].limits;
    remaining.stdout_bytes -= metadata_json().len();
    assert_eq!(executor.calls[1].limits, remaining);
}

#[test]
fn failed_compilation_retains_only_bounded_structured_classification() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(failed_process_output()),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (result, _) = check.run().unwrap();
    assert!(!result.succeeded());
    assert_eq!(result.artifact_count(), 0);
    assert_eq!(result.messages().len(), 1);
    assert_eq!(result.messages()[0].level(), CompilerMessageLevel::Error);
    assert_eq!(result.messages()[0].code(), Some("E0308"));
}

#[test]
fn compiler_diagnostics_map_normative_source_error_codes() {
    let cases: &[(&[&str], RustSourceCode, &str)] = &[
        (&["E0412"], RustSourceCode::Name, "RUST_SOURCE_NAME"),
        (&["E0308"], RustSourceCode::Type, "RUST_SOURCE_TYPE"),
        (&["E0277"], RustSourceCode::Type, "RUST_SOURCE_TYPE"),
        (&["E0499"], RustSourceCode::Borrow, "RUST_SOURCE_BORROW"),
        (&["E9999"], RustSourceCode::Type, "RUST_SOURCE_TYPE"),
        (
            &["overflowing_literals"],
            RustSourceCode::LiteralRange,
            "RUST_SOURCE_LITERAL_RANGE",
        ),
        (
            &["overflowing_literals", "E0499", "E0308", "E0412"],
            RustSourceCode::Name,
            "RUST_SOURCE_NAME",
        ),
        (
            &["overflowing_literals", "E0499", "E0308"],
            RustSourceCode::Type,
            "RUST_SOURCE_TYPE",
        ),
        (
            &["overflowing_literals", "E0499"],
            RustSourceCode::Borrow,
            "RUST_SOURCE_BORROW",
        ),
    ];
    for (codes, expected, expected_code) in cases {
        let mut fixture = Fixture::new();
        let metadata_request = fixture.metadata_request();
        let executor = RecordingExecutor::with_responses([
            Ok(ProcessOutput::success(metadata_json())),
            Ok(ProcessOutput {
                exit_code: Some(101),
                signaled: false,
                stdout: failed_check_stream_with_codes(codes),
                stderr_observed_bytes: 0,
                stdout_limit_exceeded: false,
                stderr_limit_exceeded: false,
            }),
        ]);
        let (_, check) = MetadataPhase::prepare(
            fixture.request(),
            fixture.snapshot(),
            metadata_request,
            fixture.candidate(),
            fixture.private_parent(),
            executor,
        )
        .unwrap()
        .run()
        .unwrap();
        let (result, _) = check.run().unwrap();
        assert_eq!(result.source_error_code(), Some(*expected), "{codes:?}");
        assert_eq!(result.source_error_code().unwrap().as_str(), *expected_code);
    }
}

#[test]
fn check_handshake_consumes_one_status_artifact_or_classifies_missing_locally() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        HandshakeExecutor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (handshake, _) = check.run_driver_handshake().unwrap();
    assert_eq!(
        handshake.driver().unwrap().status(),
        DriverStatus::SourceError
    );
    assert!(handshake.cargo().is_some_and(|cargo| !cargo.succeeded()));
    assert_eq!(handshake.local_frontend_error(), None);

    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(common::failed_process_output()),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (handshake, _) = check.run_driver_handshake().unwrap();
    assert!(handshake.driver().is_none());
    assert_eq!(
        handshake.local_frontend_error(),
        Some(DriverProtocolCode::Filesystem)
    );

    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(ProcessOutput::success(successful_check_stream())),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (handshake, _) = check.run_driver_handshake().unwrap();
    assert!(handshake.driver().is_none());
    assert_eq!(
        handshake.local_frontend_error(),
        Some(DriverProtocolCode::Count)
    );
}

#[test]
fn handshake_accepts_published_primary_argument_failure_without_compiler_prose() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        PrimaryArgumentFailureExecutor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (handshake, _) = check.run_driver_handshake().unwrap();
    let output = handshake.driver().expect("published private failure");
    assert_eq!(output.status(), DriverStatus::FrontendError);
    assert_eq!(output.phase(), "typecheck");
    assert!(handshake
        .cargo()
        .is_some_and(|cargo| !cargo.succeeded() && cargo.messages().is_empty()));
    assert_eq!(handshake.local_frontend_error(), None);
}

#[test]
fn malformed_stream_process_signal_and_output_limit_are_frontend_errors() {
    let cases = [
        (
            ProcessOutput::success(b"{\"reason\":\"build-finished\",\"success\":true}\n".to_vec()),
            CargoCheckCode::Protocol,
        ),
        (
            ProcessOutput {
                exit_code: None,
                signaled: true,
                stdout: Vec::new(),
                stderr_observed_bytes: 0,
                stdout_limit_exceeded: false,
                stderr_limit_exceeded: false,
            },
            CargoCheckCode::Process,
        ),
        (
            ProcessOutput {
                exit_code: Some(0),
                signaled: false,
                stdout: successful_check_stream(),
                stderr_observed_bytes: 0,
                stdout_limit_exceeded: true,
                stderr_limit_exceeded: false,
            },
            CargoCheckCode::Sandbox(SandboxError::ChildOutputLimit),
        ),
    ];
    for (check_output, expected) in cases {
        let mut fixture = Fixture::new();
        let metadata_request = fixture.metadata_request();
        let executor = RecordingExecutor::with_responses([
            Ok(ProcessOutput::success(metadata_json())),
            Ok(check_output),
        ]);
        let (_, check) = MetadataPhase::prepare(
            fixture.request(),
            fixture.snapshot(),
            metadata_request,
            fixture.candidate(),
            fixture.private_parent(),
            executor,
        )
        .unwrap()
        .run()
        .unwrap();
        assert_eq!(
            check
                .run()
                .err()
                .expect("invalid check output must fail")
                .code,
            expected
        );
    }
}

#[test]
fn sandbox_unavailable_maps_to_the_shared_typecheck_frontend_code() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Err(SandboxError::SandboxUnavailable),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    let error = check
        .run()
        .err()
        .expect("unavailable isolation must terminate typecheck");
    assert_eq!(
        error.code,
        CargoCheckCode::Sandbox(SandboxError::SandboxUnavailable)
    );
    assert_eq!(error.code.as_str(), "FRONTEND_SANDBOX_UNAVAILABLE");
}

#[test]
fn raw_compiler_message_count_does_not_reclassify_the_source_result() {
    let failed = String::from_utf8(common::failed_check_stream()).unwrap();
    let diagnostic = failed.lines().next().unwrap();
    let mut oversized = String::new();
    for _ in 0..=1_024 {
        oversized.push_str(diagnostic);
        oversized.push('\n');
    }
    oversized.push_str("{\"reason\":\"build-finished\",\"success\":false}\n");

    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(ProcessOutput {
            exit_code: Some(101),
            signaled: false,
            stdout: oversized.into_bytes(),
            stderr_observed_bytes: 0,
            stdout_limit_exceeded: false,
            stderr_limit_exceeded: false,
        }),
    ]);
    let (_, check) = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap()
    .run()
    .unwrap();
    let (result, _) = check.run().expect("raw messages remain a source result");
    assert!(!result.succeeded());
    assert_eq!(result.messages().len(), 1_024);
    assert_eq!(result.source_error_code(), Some(RustSourceCode::Type));
}
