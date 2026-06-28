#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use mpk_kernel::{
    verify_certificate_bytes, verify_certificate_bytes_json_output, VerificationJsonOutput,
};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(RunOutcome::Help) => ExitCode::SUCCESS,
        Ok(RunOutcome::Check(output)) => {
            println!("{}", output.json);
            if output.accepted {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Ok(RunOutcome::Verify(message)) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
        Err(CliError::Input(message)) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<RunOutcome, CliError> {
    match args.as_slice() {
        [command, path] if command == "check" => check_path(Path::new(path)),
        [command, path] if command == "verify" => verify_path(Path::new(path)),
        [command] if command == "--help" || command == "-h" || command == "help" => {
            print_usage();
            Ok(RunOutcome::Help)
        }
        _ => Err(CliError::Usage(
            "usage: mpk check <certificate.mpcert|fixture.hex>".to_owned(),
        )),
    }
}

fn check_path(path: &Path) -> Result<RunOutcome, CliError> {
    let bytes = read_certificate_input(path)?;
    Ok(RunOutcome::Check(verify_certificate_bytes_json_output(
        &bytes,
    )))
}

fn verify_path(path: &Path) -> Result<RunOutcome, CliError> {
    let bytes = read_certificate_input(path)?;
    let report = verify_certificate_bytes(&bytes).map_err(|error| {
        CliError::Input(format!(
            "verification failed: {:?}: {}",
            error.kind(),
            error.detail()
        ))
    })?;

    Ok(RunOutcome::Verify(format!(
        "ok module={} declarations={} axioms={}",
        report.module, report.declaration_count, report.axiom_count
    )))
}

fn read_certificate_input(path: &Path) -> Result<Vec<u8>, CliError> {
    let bytes = fs::read(path)
        .map_err(|error| CliError::Input(format!("failed to read {}: {error}", path.display())))?;
    if path.extension().is_some_and(|extension| extension == "hex") {
        decode_hex(&bytes)
    } else {
        Ok(bytes)
    }
}

fn decode_hex(bytes: &[u8]) -> Result<Vec<u8>, CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| CliError::Input(format!("hex fixture is not UTF-8: {error}")))?;
    let hex = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if hex.len() % 2 != 0 {
        return Err(CliError::Input(
            "hex fixture has an odd number of digits".to_owned(),
        ));
    }

    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk)
                .map_err(|error| CliError::Input(format!("hex chunk is not UTF-8: {error}")))?;
            u8::from_str_radix(byte, 16)
                .map_err(|error| CliError::Input(format!("invalid hex byte `{byte}`: {error}")))
        })
        .collect()
}

fn print_usage() {
    println!("usage: mpk check <certificate.mpcert|fixture.hex>");
}

enum RunOutcome {
    Help,
    Check(VerificationJsonOutput),
    Verify(String),
}

enum CliError {
    Usage(String),
    Input(String),
}
