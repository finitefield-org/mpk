use std::ffi::OsString;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("{}", rust2vir_internal::version_line("rust2vir-driver"));
        return ExitCode::SUCCESS;
    }
    let Some(rustc) = arguments.first() else {
        eprintln!("RUST_TOOLCHAIN_ARGUMENT");
        return ExitCode::from(1);
    };
    match compiler_identity(rustc) {
        Ok(()) => {
            eprintln!("RUST_FRONTEND_DRIVER_PROTOCOL_COUNT");
            ExitCode::from(1)
        }
        Err(code) => {
            eprintln!("{code}");
            ExitCode::from(1)
        }
    }
}

fn compiler_identity(rustc: &OsString) -> Result<(), &'static str> {
    let output = Command::new(rustc)
        .arg("-vV")
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("LD_LIBRARY_PATH", "/mpk/toolchain/lib")
        .env("PATH", "/mpk/toolchain/bin")
        .env("TZ", "UTC")
        .stdin(Stdio::null())
        .output()
        .map_err(|_| "RUST_TOOLCHAIN_COMPONENT")?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err("RUST_TOOLCHAIN_COMPONENT");
    }
    rust2vir_internal::validate_rustc_verbose(&output.stdout).map_err(|_| "RUST_TOOLCHAIN_COMMIT")
}
