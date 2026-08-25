//! Fixed execution boundary for the embedded independent reference checker.

use std::fmt;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use rustix::fs::{
    fchmod, fcntl_add_seals, fcntl_get_seals, memfd_create, MemfdFlags, Mode, SealFlags,
};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::fs::File;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::io::{self, Read, Write};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::os::fd::AsRawFd;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::os::unix::process::CommandExt;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::process::{Child, Command, ExitStatus, Stdio};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::thread;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::time::{Duration, Instant};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const REFERENCE_CHECKER_BINARY: &[u8] =
    include_bytes!("../../../release/checkers/mpk-checker-ref-linux-amd64");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const REFERENCE_CHECKER_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const REFERENCE_CHECKER_STREAM_MAX: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceCheckerProcessOutput {
    status_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl ReferenceCheckerProcessOutput {
    pub fn status_code(&self) -> Option<i32> {
        self.status_code
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceCheckerExecutionError {
    detail: String,
}

impl ReferenceCheckerExecutionError {
    fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ReferenceCheckerExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for ReferenceCheckerExecutionError {}

/// Runs only the build-pinned checker bytes embedded in `mpk-cli`.
///
/// There is deliberately no executable path, environment, registry, callback,
/// or source-tree argument. The candidate is delivered on standard input.
pub fn execute_reference_checker(
    certificate: &[u8],
) -> Result<ReferenceCheckerProcessOutput, ReferenceCheckerExecutionError> {
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = certificate;
        Err(ReferenceCheckerExecutionError::new(
            "the embedded reference checker requires the v0 Linux x86_64 host",
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        execute_linux(certificate)
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn execute_linux(
    certificate: &[u8],
) -> Result<ReferenceCheckerProcessOutput, ReferenceCheckerExecutionError> {
    let executable = sealed_checker_file().map_err(execution_error("materialize checker"))?;
    let executable_path = format!("/proc/self/fd/{}", executable.as_raw_fd());
    let mut command = Command::new(executable_path);
    command
        .arg0("mpk-checker-ref")
        .args(["verify", "-"])
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .env("GOMAXPROCS", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(execution_error("launch embedded checker"))?;
    let stdin = child.stdin.take().ok_or_else(|| {
        terminate_and_reap(&mut child);
        ReferenceCheckerExecutionError::new("embedded checker stdin is unavailable")
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        terminate_and_reap(&mut child);
        ReferenceCheckerExecutionError::new("embedded checker stdout is unavailable")
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        terminate_and_reap(&mut child);
        ReferenceCheckerExecutionError::new("embedded checker stderr is unavailable")
    })?;

    let (status, input_result, stdout_result, stderr_result) = thread::scope(|scope| {
        let input = scope.spawn(move || write_candidate(stdin, certificate));
        let output = scope.spawn(move || capture_stream(stdout));
        let errors = scope.spawn(move || capture_stream(stderr));
        let status = wait_for_checker(&mut child);
        (status, input.join(), output.join(), errors.join())
    });
    let status = status?;
    input_result
        .map_err(|_| ReferenceCheckerExecutionError::new("checker input worker panicked"))?
        .map_err(execution_error("write checker input"))?;
    let stdout = stdout_result
        .map_err(|_| ReferenceCheckerExecutionError::new("checker stdout worker panicked"))?
        .map_err(execution_error("read checker stdout"))?;
    let stderr = stderr_result
        .map_err(|_| ReferenceCheckerExecutionError::new("checker stderr worker panicked"))?
        .map_err(execution_error("read checker stderr"))?;
    if stdout.exceeded || stderr.exceeded {
        return Err(ReferenceCheckerExecutionError::new(
            "embedded checker exceeded its output limit",
        ));
    }
    Ok(ReferenceCheckerProcessOutput {
        status_code: status.code(),
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn sealed_checker_file() -> io::Result<File> {
    let (descriptor, seal_executable_mode) = match memfd_create(
        "mpk-reference-checker",
        MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING | MemfdFlags::EXEC,
    ) {
        Ok(descriptor) => (descriptor, true),
        Err(rustix::io::Errno::INVAL) => (
            memfd_create(
                "mpk-reference-checker",
                MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING,
            )?,
            false,
        ),
        Err(error) => return Err(error.into()),
    };
    let mut file = File::from(descriptor);
    file.write_all(REFERENCE_CHECKER_BINARY)?;
    file.flush()?;
    fchmod(&file, Mode::from_raw_mode(0o555))?;
    let mut seals = SealFlags::SEAL
        | SealFlags::SHRINK
        | SealFlags::GROW
        | SealFlags::WRITE
        | SealFlags::FUTURE_WRITE;
    if seal_executable_mode {
        seals |= SealFlags::EXEC;
    }
    fcntl_add_seals(&file, seals)?;
    if !fcntl_get_seals(&file)?.contains(seals)
        || file.metadata()?.len() != REFERENCE_CHECKER_BINARY.len() as u64
    {
        return Err(io::Error::other(
            "embedded checker identity could not be sealed",
        ));
    }
    Ok(file)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn write_candidate(mut stdin: impl Write, certificate: &[u8]) -> io::Result<()> {
    stdin.write_all(certificate)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
struct BoundedCapture {
    bytes: Vec<u8>,
    exceeded: bool,
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn capture_stream(mut stream: impl Read) -> io::Result<BoundedCapture> {
    let mut bytes = Vec::new();
    let mut exceeded = false;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let available = REFERENCE_CHECKER_STREAM_MAX.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..count.min(available)]);
        exceeded |= count > available;
    }
    Ok(BoundedCapture { bytes, exceeded })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn wait_for_checker(child: &mut Child) -> Result<ExitStatus, ReferenceCheckerExecutionError> {
    let deadline = Instant::now() + REFERENCE_CHECKER_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(5));
            }
            Ok(None) => {
                terminate_and_reap(child);
                return Err(ReferenceCheckerExecutionError::new(
                    "embedded checker exceeded its wall-clock limit",
                ));
            }
            Err(error) => {
                terminate_and_reap(child);
                return Err(ReferenceCheckerExecutionError::new(format!(
                    "wait for embedded checker: {error}"
                )));
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn terminate_and_reap(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn execution_error(
    operation: &'static str,
) -> impl FnOnce(io::Error) -> ReferenceCheckerExecutionError {
    move |error| ReferenceCheckerExecutionError::new(format!("{operation}: {error}"))
}
