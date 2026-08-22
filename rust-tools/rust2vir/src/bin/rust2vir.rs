use rust2vir_internal::cli::{self, NonSuccessStatus};
use rust2vir_internal::preflight;
use std::process::ExitCode;

const USAGE: &str = "rust2vir lower SOURCE_ROOT --manifest-path Cargo.toml --package PACKAGE --semantic-profile mpk.rust.checked.v0 --target TARGET --function FUNCTION --frontend-bundle-id ID --frontend-sha256 SHA256 --release-registry-id ID --release-registry-sha256 SHA256 --toolchain-bundle-id ID --toolchain-root PATH --toolchain-distribution-sha256 SHA256 --driver PATH --driver-sha256 SHA256 --contract RELATIVE_PATH [--contract RELATIVE_PATH ...]";

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
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
    match preflight::run(&request) {
        Ok(_preflight) => {
            println!(
                "{}",
                cli::non_success_envelope(
                    &request,
                    NonSuccessStatus::FrontendError,
                    "RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS",
                    "lowering stage is unavailable",
                )
            );
            ExitCode::from(1)
        }
        Err(error) => {
            println!(
                "{}",
                cli::non_success_envelope(
                    &request,
                    NonSuccessStatus::Rejected,
                    error.code.as_str(),
                    error.code.message(),
                )
            );
            ExitCode::from(3)
        }
    }
}
