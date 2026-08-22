#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../rustc_driver.rs"]
mod rustc_driver_adapter;

use rust2vir_internal::driver_process::{
    classify_invocation, publish_primary_result, read_request, validate_fixed_binary_identities,
    WrapperInvocation,
};
use rust2vir_internal::driver_protocol::{
    encode_non_success, parse_request_transport, DriverStatus, PrivateDiagnostic, OUTPUT_DIRECTORY,
    REQUEST_PATH,
};
use rust2vir_internal::environment::{EvidenceEnvironment, INPUT_ROOT};
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceLoaderError, SourceLoaderStatus};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use rustc_driver_adapter::RustcDriverError;

const PROBE_STREAM_MAX: usize = 65_536;

fn main() -> ExitCode {
    let arguments = match std::env::args_os()
        .skip(1)
        .map(|argument| argument.into_string())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(arguments) => arguments,
        Err(_) => return fail("RUST_TOOLCHAIN_ARGUMENT"),
    };
    if arguments == ["--version"] {
        println!("{}", rust2vir_internal::version_line("rust2vir-driver"));
        return ExitCode::SUCCESS;
    }
    let bytes = match read_request(Path::new(REQUEST_PATH)) {
        Ok(bytes) => bytes,
        Err(error) => return fail(error.code.as_str()),
    };
    let request = match parse_request_transport(&bytes) {
        Ok(request) => request,
        Err(error) => return fail(error.code.as_str()),
    };
    if let Err(error) = validate_fixed_binary_identities(&request) {
        return fail(error.code.as_str());
    }
    let invocation = match classify_invocation(&request, &arguments) {
        Ok(invocation) => invocation,
        Err(_) => return fail("RUST_TOOLCHAIN_ARGUMENT"),
    };
    match invocation {
        WrapperInvocation::VersionProbe => version_probe(&arguments),
        WrapperInvocation::SysrootProbe
        | WrapperInvocation::CrateInformationHost
        | WrapperInvocation::CrateInformationTarget => {
            if compiler_identity(&arguments[0]).is_err() {
                return fail("RUST_TOOLCHAIN_COMMIT");
            }
            delegate(&arguments)
        }
        WrapperInvocation::Primary => {
            if compiler_identity(&arguments[0]).is_err() {
                return fail("RUST_TOOLCHAIN_COMMIT");
            }
            let crate_root = arguments
                .get(4)
                .expect("classified primary invocation has a crate root");
            let source_loader =
                match SnapshotFileLoader::from_request(Path::new(INPUT_ROOT), crate_root, &request)
                {
                    Ok(loader) => Arc::new(loader),
                    Err(error) => return publish_source_failure(&request, error),
                };
            if let Err(error) =
                rustc_driver_adapter::run_primary(&arguments, &request, source_loader)
            {
                return publish_rustc_failure(&request, error);
            }
            publish_primary_diagnostic(
                &request,
                DriverStatus::FrontendError,
                "lowering",
                "RUST_TOOLCHAIN_MIR_ADAPTER",
                "pinned MIR adapter is not initialized",
            )
        }
    }
}

fn publish_rustc_failure(
    request: &rust2vir_internal::driver_protocol::DriverRequest,
    error: RustcDriverError,
) -> ExitCode {
    match error {
        RustcDriverError::Source(error) => publish_source_failure(request, error),
        RustcDriverError::Session => publish_primary_diagnostic(
            request,
            DriverStatus::FrontendError,
            "typecheck",
            "RUST_TOOLCHAIN_OPTIONS",
            "effective rustc session differs from the pinned profile",
        ),
        RustcDriverError::MirAdapter => publish_primary_diagnostic(
            request,
            DriverStatus::FrontendError,
            "lowering",
            "RUST_TOOLCHAIN_MIR_ADAPTER",
            "pinned MIR adapter identity or access point differs",
        ),
        RustcDriverError::Compiler => publish_primary_diagnostic(
            request,
            DriverStatus::SourceError,
            "typecheck",
            "RUST_SOURCE_TYPE",
            "rustc rejected the captured source before MIR access",
        ),
    }
}

fn publish_source_failure(
    request: &rust2vir_internal::driver_protocol::DriverRequest,
    error: SourceLoaderError,
) -> ExitCode {
    let status = match error.code.status() {
        SourceLoaderStatus::Rejected => DriverStatus::Rejected,
        SourceLoaderStatus::SourceError => DriverStatus::SourceError,
        SourceLoaderStatus::FrontendError => DriverStatus::FrontendError,
    };
    publish_primary_diagnostic(
        request,
        status,
        error.code.phase(),
        error.code.as_str(),
        error.code.message(),
    )
}

fn publish_primary_diagnostic(
    request: &rust2vir_internal::driver_protocol::DriverRequest,
    status: DriverStatus,
    phase: &str,
    code: &str,
    message: &str,
) -> ExitCode {
    let result = encode_non_success(
        request,
        status,
        phase,
        &[PrivateDiagnostic {
            code: code.to_owned(),
            message: message.to_owned(),
            function_id: Some(request.selection().2.to_owned()),
        }],
    )
    .and_then(|bytes| publish_primary_result(Path::new(OUTPUT_DIRECTORY), &bytes));
    match result {
        Ok(()) => ExitCode::from(
            u8::try_from(status.exit_code()).expect("driver status exit codes fit in u8"),
        ),
        Err(error) => fail(error.code.as_str()),
    }
}

fn version_probe(arguments: &[String]) -> ExitCode {
    let output = match bounded_probe(arguments) {
        Ok(output) => output,
        Err(_) => return fail("RUST_TOOLCHAIN_COMPONENT"),
    };
    if !output.status.success()
        || !output.stderr.is_empty()
        || rust2vir_internal::validate_rustc_verbose(&output.stdout).is_err()
    {
        return fail("RUST_TOOLCHAIN_COMMIT");
    }
    if std::io::stdout().write_all(&output.stdout).is_err() {
        return fail("RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS");
    }
    ExitCode::SUCCESS
}

fn compiler_identity(rustc_path: &str) -> Result<(), ()> {
    let output = bounded_probe(&[rustc_path.to_owned(), "-vV".to_owned()])?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(());
    }
    rust2vir_internal::validate_rustc_verbose(&output.stdout).map_err(|_| ())
}

struct ProbeOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn bounded_probe(arguments: &[String]) -> Result<ProbeOutput, ()> {
    let mut child = rustc(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ())?;
    let stdout = child.stdout.take().ok_or(())?;
    let stderr = child.stderr.take().ok_or(())?;
    let overflow = Arc::new(AtomicBool::new(false));
    let read_failed = Arc::new(AtomicBool::new(false));
    let stdout_reader = bounded_reader(stdout, Arc::clone(&overflow), Arc::clone(&read_failed));
    let stderr_reader = bounded_reader(stderr, Arc::clone(&overflow), Arc::clone(&read_failed));
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::yield_now(),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(());
            }
        }
    };
    let stdout = stdout_reader.join().map_err(|_| ())??;
    let stderr = stderr_reader.join().map_err(|_| ())??;
    if overflow.load(Ordering::Acquire) || read_failed.load(Ordering::Acquire) {
        return Err(());
    }
    Ok(ProbeOutput {
        status,
        stdout,
        stderr,
    })
}

fn bounded_reader<R: Read + Send + 'static>(
    mut reader: R,
    overflow: Arc<AtomicBool>,
    read_failed: Arc<AtomicBool>,
) -> thread::JoinHandle<Result<Vec<u8>, ()>> {
    thread::spawn(move || {
        let mut retained = Vec::new();
        let mut buffer = [0_u8; 4_096];
        loop {
            let count = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => count,
                Err(_) => {
                    read_failed.store(true, Ordering::Release);
                    return Err(());
                }
            };
            if retained
                .len()
                .checked_add(count)
                .is_none_or(|length| length > PROBE_STREAM_MAX)
            {
                overflow.store(true, Ordering::Release);
                continue;
            }
            retained.extend_from_slice(&buffer[..count]);
        }
        Ok(retained)
    })
}

fn delegate(arguments: &[String]) -> ExitCode {
    let output = match bounded_probe(arguments) {
        Ok(output) => output,
        Err(_) => return fail("RUST_TOOLCHAIN_COMPONENT"),
    };
    if std::io::stdout().write_all(&output.stdout).is_err()
        || std::io::stderr().write_all(&output.stderr).is_err()
    {
        return fail("RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS");
    }
    output
        .status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or_else(
            || fail("RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS"),
            ExitCode::from,
        )
}

fn rustc(arguments: &[String]) -> Command {
    let mut command = Command::new(&arguments[0]);
    command
        .args(&arguments[1..])
        .env_clear()
        .envs(EvidenceEnvironment::frozen().entries());
    command
}

fn fail(code: &str) -> ExitCode {
    eprintln!("{code}");
    ExitCode::from(1)
}
