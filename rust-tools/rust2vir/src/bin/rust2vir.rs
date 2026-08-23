use rust2vir_internal::cargo_check::{CargoCheckCode, RustSourceCode};
use rust2vir_internal::cargo_metadata::{MetadataPhase, MetadataStatus};
use rust2vir_internal::cli::{self, LowerRequest, NonSuccessStatus};
use rust2vir_internal::emit;
use rust2vir_internal::manifest::{self, ManifestStatus};
use rust2vir_internal::metadata_request::MetadataRequest;
use rust2vir_internal::module_closure::{self, ClosureStatus};
use rust2vir_internal::preflight;
use rust2vir_internal::sandbox::{
    run_bootstrap, run_outer_bootstrap, InjectedCandidate, LinuxNamespaceExecutor,
};
use rust2vir_internal::snapshot::Snapshot;
use rust2vir_internal::source_gate;
use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "rust2vir lower SOURCE_ROOT --manifest-path Cargo.toml --package PACKAGE --semantic-profile mpk.rust.checked.v0 --target TARGET --function FUNCTION --frontend-bundle-id ID --frontend-sha256 SHA256 --release-registry-id ID --release-registry-sha256 SHA256 --toolchain-bundle-id ID --toolchain-root PATH --toolchain-distribution-sha256 SHA256 --driver PATH --driver-sha256 SHA256 --contract RELATIVE_PATH [--contract RELATIVE_PATH ...]";
const CARGO_SANDBOX_BOOTSTRAP: &str = "__rust2vir_cargo_sandbox_v0";
const CARGO_OUTER_SANDBOX_BOOTSTRAP: &str = "__rust2vir_cargo_outer_sandbox_v0";

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.first().and_then(|argument| argument.to_str())
        == Some("__rust2vir_outer_sandbox_v0")
    {
        arguments.remove(0);
        if rust2vir_internal::sandbox::mount_outer_proc() != 0 {
            return ExitCode::from(125);
        }
    }
    let cargo_bootstrap = arguments.first().and_then(|argument| argument.to_str());
    if matches!(
        cargo_bootstrap,
        Some(CARGO_SANDBOX_BOOTSTRAP | CARGO_OUTER_SANDBOX_BOOTSTRAP)
    ) {
        let inherited_user_namespace = cargo_bootstrap == Some(CARGO_OUTER_SANDBOX_BOOTSTRAP);
        let arguments = arguments[1..]
            .iter()
            .map(|argument| argument.to_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>();
        let status = match (arguments.as_deref(), inherited_user_namespace) {
            (Some(arguments), true) => run_outer_bootstrap(arguments),
            (Some(arguments), false) => run_bootstrap(arguments),
            (None, _) => 125,
        };
        return ExitCode::from(status);
    }
    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("{}", rust2vir_internal::version_line("rust2vir"));
        return ExitCode::SUCCESS;
    }
    if arguments.len() == 1 && arguments[0] == "--help" {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let request = match cli::parse_lower_args(arguments) {
        Ok(request) => request,
        Err(_) => {
            eprintln!("RUST_TOOLCHAIN_ARGUMENT");
            return ExitCode::from(2);
        }
    };
    lower(&request)
}

fn lower(request: &LowerRequest) -> ExitCode {
    let preflight = match preflight::run(request) {
        Ok(preflight) => preflight,
        Err(error) => {
            return emit_local(
                request,
                NonSuccessStatus::Rejected,
                "capture",
                error.code.as_str(),
                error.code.message(),
                3,
            );
        }
    };
    let validated = match manifest::validate(request, preflight) {
        Ok(validated) => validated,
        Err(error) => {
            let (status, exit) = match error.code.status() {
                ManifestStatus::Rejected => (NonSuccessStatus::Rejected, 3),
                ManifestStatus::SourceError => (NonSuccessStatus::SourceError, 4),
            };
            return emit_local(
                request,
                status,
                "capture",
                error.code.as_str(),
                error.code.message(),
                exit,
            );
        }
    };
    let (closure, expected) = match module_closure::discover(validated) {
        Ok(value) => value,
        Err(error) => {
            let (status, exit) = match error.code.status() {
                ClosureStatus::Rejected => (NonSuccessStatus::Rejected, 3),
                ClosureStatus::SourceError => (NonSuccessStatus::SourceError, 4),
            };
            return emit_local(
                request,
                status,
                error.code.phase(),
                error.code.as_str(),
                error.code.message(),
                exit,
            );
        }
    };
    let core_prelude = closure
        .inputs
        .iter()
        .find(|input| input.normalized_path == closure.library_root)
        .and_then(|input| source_gate::crate_uses_core_prelude(&input.bytes).ok());
    let Some(core_prelude) = core_prelude else {
        return emit_local(
            request,
            NonSuccessStatus::FrontendError,
            "source",
            "RUST_FRONTEND_SOURCE_INVENTORY",
            "captured crate root could not be revalidated",
            1,
        );
    };
    let snapshot = match Snapshot::create(&std::env::temp_dir(), &closure) {
        Ok(snapshot) => snapshot,
        Err(_) => {
            return emit_local(
                request,
                NonSuccessStatus::FrontendError,
                "source",
                "RUST_FRONTEND_SOURCE_INVENTORY",
                "immutable source snapshot could not be created",
                1,
            );
        }
    };
    let metadata_request = match MetadataRequest::for_snapshot(&snapshot, expected) {
        Ok(metadata_request) => metadata_request,
        Err(_) => {
            return emit_local(
                request,
                NonSuccessStatus::FrontendError,
                "source",
                "RUST_FRONTEND_SOURCE_INVENTORY",
                "immutable source snapshot could not be validated",
                1,
            );
        }
    };
    let candidate = match InjectedCandidate::from_installed(request) {
        Ok(candidate) => candidate,
        Err(error) => {
            return emit_local(
                request,
                NonSuccessStatus::FrontendError,
                "metadata",
                error.code(),
                "installed Rust release identity could not be validated",
                1,
            );
        }
    };
    let metadata = match MetadataPhase::prepare(
        request,
        &snapshot,
        metadata_request,
        &candidate,
        &std::env::temp_dir(),
        LinuxNamespaceExecutor,
    ) {
        Ok(metadata) => metadata,
        Err(error) => {
            let (status, exit) = match error.code.status() {
                MetadataStatus::Rejected => (NonSuccessStatus::Rejected, 3),
                MetadataStatus::FrontendError => (NonSuccessStatus::FrontendError, 1),
            };
            return emit_local(
                request,
                status,
                error.code.phase(),
                error.code.as_str(),
                "Cargo metadata did not match the captured package",
                exit,
            );
        }
    };
    let (_metadata, check) = match metadata.run() {
        Ok(value) => value,
        Err(error) => {
            let (status, exit) = match error.code.status() {
                MetadataStatus::Rejected => (NonSuccessStatus::Rejected, 3),
                MetadataStatus::FrontendError => (NonSuccessStatus::FrontendError, 1),
            };
            return emit_local(
                request,
                status,
                error.code.phase(),
                error.code.as_str(),
                "Cargo metadata execution did not produce the pinned result",
                exit,
            );
        }
    };
    let (handshake, _executor) = match check.run_driver_handshake() {
        Ok(value) => value,
        Err(error) => {
            let message = match error.code {
                CargoCheckCode::DiagnosticBudget => "compiler diagnostic budget was exceeded",
                _ => "Cargo check did not produce the pinned result",
            };
            return emit_local(
                request,
                NonSuccessStatus::FrontendError,
                "typecheck",
                error.code.as_str(),
                message,
                1,
            );
        }
    };
    if let Some(code) = handshake.local_frontend_error() {
        let phase = if matches!(
            code,
            rust2vir_internal::driver_protocol::DriverProtocolCode::SourceMapExternal
                | rust2vir_internal::driver_protocol::DriverProtocolCode::SourceMapRange
                | rust2vir_internal::driver_protocol::DriverProtocolCode::SourceMapReference
        ) {
            "emission"
        } else {
            "lowering"
        };
        return emit_local(
            request,
            NonSuccessStatus::FrontendError,
            phase,
            code.as_str(),
            "private Rust driver output could not be validated",
            1,
        );
    }
    let Some(output) = handshake.driver() else {
        return emit_local(
            request,
            NonSuccessStatus::FrontendError,
            "lowering",
            "RUST_FRONTEND_DRIVER_PROTOCOL_COUNT",
            "private Rust driver output was not produced exactly once",
            1,
        );
    };
    if output.status() == rust2vir_internal::driver_protocol::DriverStatus::SourceError
        && output.phase() == "typecheck"
    {
        let code = handshake
            .cargo()
            .and_then(|cargo| cargo.source_error_code())
            .unwrap_or(RustSourceCode::Type);
        return emit_local(
            request,
            NonSuccessStatus::SourceError,
            "typecheck",
            code.as_str(),
            "rustc rejected the captured source before MIR access",
            4,
        );
    }
    let private_request = match rust2vir_internal::driver_protocol::construct_request(
        request,
        snapshot.inputs(),
        candidate.driver_release_identity(),
    ) {
        Ok(private_request) => private_request,
        Err(error) => {
            return emit_local(
                request,
                NonSuccessStatus::FrontendError,
                "lowering",
                error.code.as_str(),
                "validated private identities could not be reconstructed",
                1,
            );
        }
    };
    let (transport, exit) =
        if output.status() == rust2vir_internal::driver_protocol::DriverStatus::Lowered {
            (
                emit::success_envelope(request, &private_request, output, core_prelude),
                0,
            )
        } else {
            (
                emit::driver_non_success_envelope(request, output),
                output.status().exit_code(),
            )
        };
    match transport {
        Ok(transport) => write_transport(&transport, exit),
        Err(emit::EmissionError::IrLimit) => emit_local(
            request,
            NonSuccessStatus::Rejected,
            "emission",
            "RUST_LIMIT_IR",
            "canonical Rust artifact limit was exceeded",
            3,
        ),
        Err(emit::EmissionError::Integrity) => emit_local(
            request,
            NonSuccessStatus::FrontendError,
            "lowering",
            "RUST_FRONTEND_DRIVER_PROTOCOL_HASH",
            "public Rust artifacts could not be deterministically assembled",
            1,
        ),
    }
}

fn emit_local(
    request: &LowerRequest,
    status: NonSuccessStatus,
    phase: &str,
    code: &str,
    message: &str,
    exit: i32,
) -> ExitCode {
    match emit::local_non_success_envelope(request, status, phase, code, message) {
        Ok(transport) => write_transport(&transport, exit),
        Err(_) => ExitCode::from(1),
    }
}

fn write_transport(transport: &[u8], exit: i32) -> ExitCode {
    if std::io::stdout().write_all(transport).is_err() {
        return ExitCode::from(1);
    }
    ExitCode::from(u8::try_from(exit).unwrap_or(1))
}
