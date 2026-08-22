mod common;

use common::{metadata_json, successful_check_stream, Fixture, RecordingExecutor};
use rust2vir_internal::cargo_metadata::{MetadataCode, MetadataPhase, MetadataStatus};
use rust2vir_internal::sandbox::{CargoInvocationKind, ProcessOutput};

#[test]
fn metadata_runs_first_with_the_exact_snapshot_command_and_closed_context() {
    let mut fixture = Fixture::new();
    let metadata_request = fixture.metadata_request();
    let executor = RecordingExecutor::with_responses([
        Ok(ProcessOutput::success(metadata_json())),
        Ok(ProcessOutput::success(successful_check_stream())),
    ]);
    let phase = MetadataPhase::prepare(
        fixture.request(),
        fixture.snapshot(),
        metadata_request,
        fixture.candidate(),
        fixture.private_parent(),
        executor,
    )
    .unwrap();
    let (metadata, check) = phase.run().unwrap();
    assert_eq!(metadata.package_name(), "vector");
    assert_eq!(metadata.crate_name(), "vector");
    let (_, executor) = check.run().unwrap();

    assert_eq!(executor.calls.len(), 2);
    let metadata_call = &executor.calls[0];
    assert_eq!(metadata_call.kind, CargoInvocationKind::Metadata);
    assert_eq!(metadata_call.executable, "/mpk/toolchain/bin/cargo");
    assert_eq!(
        metadata_call.arguments,
        [
            "metadata",
            "--manifest-path",
            "/mpk/input/Cargo.toml",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
            "--no-default-features",
            "--color",
            "never",
        ]
    );
    assert_eq!(metadata_call.snapshot_root, fixture.snapshot().path());
    assert!(metadata_call
        .arguments
        .iter()
        .all(|argument| !argument.contains(fixture.request().source_root.to_str().unwrap())));
}

#[test]
fn metadata_cross_checks_workspace_package_target_and_snapshot_paths() {
    for (needle, replacement) in [
        (
            "\"workspace_root\":\"/mpk/input\"",
            "\"workspace_root\":\"/host/source\"",
        ),
        ("\"name\":\"vector\"", "\"name\":\"other\""),
        (
            "\"src_path\":\"/mpk/input/src/lib.rs\"",
            "\"src_path\":\"/host/source/src/lib.rs\"",
        ),
        ("\"dependencies\":[]", "\"dependencies\":[{}]"),
    ] {
        let mut fixture = Fixture::new();
        let metadata_request = fixture.metadata_request();
        let changed = String::from_utf8(metadata_json())
            .unwrap()
            .replacen(needle, replacement, 1)
            .into_bytes();
        let executor = RecordingExecutor::with_responses([Ok(ProcessOutput::success(changed))]);
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
        .expect("changed metadata must reject");
        assert_eq!(error.code, MetadataCode::Mismatch, "{needle}");
        assert_eq!(error.code.status(), MetadataStatus::Rejected);
    }
}

#[test]
fn malformed_unknown_or_failed_metadata_is_artifact_free_frontend_error() {
    for (output, code) in [
        (
            ProcessOutput::success(b"{not json}\n".to_vec()),
            MetadataCode::Protocol,
        ),
        (
            ProcessOutput::success(
                String::from_utf8(metadata_json())
                    .unwrap()
                    .replacen(
                        "{\"metadata\":null",
                        "{\"ambient\":true,\"metadata\":null",
                        1,
                    )
                    .into_bytes(),
            ),
            MetadataCode::Protocol,
        ),
        (
            ProcessOutput {
                exit_code: Some(101),
                signaled: false,
                stdout: Vec::new(),
                stderr_observed_bytes: 20,
                stdout_limit_exceeded: false,
                stderr_limit_exceeded: false,
            },
            MetadataCode::Process,
        ),
    ] {
        let mut fixture = Fixture::new();
        let metadata_request = fixture.metadata_request();
        let executor = RecordingExecutor::with_responses([Ok(output)]);
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
        .expect("invalid metadata must fail");
        assert_eq!(error.code, code);
        assert_eq!(error.code.status(), MetadataStatus::FrontendError);
    }
}
