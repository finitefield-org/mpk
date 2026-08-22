use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("{}", rust2vir_internal::version_line("rust2vir"));
        ExitCode::SUCCESS
    } else {
        eprintln!("RUST_TOOLCHAIN_ARGUMENT");
        ExitCode::from(64)
    }
}
