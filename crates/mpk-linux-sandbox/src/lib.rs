#![cfg(target_os = "linux")]
#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::BTreeSet;
use std::ffi::{c_char, CString, OsString};
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::ExitStatus;
use std::ptr;

use rustix::io::{fcntl_dupfd_cloexec, fcntl_getfd, fcntl_setfd, FdFlags};
use rustix::pipe::{pipe_with, PipeFlags};
use rustix::process::{pidfd_send_signal, waitid, Signal, WaitId, WaitIdOptions, WaitIdStatus};

mod java_policy;

/// Irreversibly installs the Java privilege/syscall boundary in a dedicated,
/// single-threaded capability-probe process. No source may be exposed to it.
/// Its only UID/GID must already be mapped to 65534 in a private user namespace.
pub fn install_java_probe_policy() -> io::Result<()> {
    let filter = java_policy::filter();
    let program = java_policy::program(&filter);
    if java_policy::install(&program) {
        Ok(())
    } else {
        Err(io::Error::other("Java native policy unavailable"))
    }
}

const CLONE_PIDFD: u64 = 0x0000_1000;
const CLONE_CLEAR_SIGHAND: u64 = 1 << 32;
// libc 0.2 exposes this as an overflowing c_int on some targets. clone3's
// flags field is u64 and the kernel ABI value is bit 33.
const CLONE_INTO_CGROUP: u64 = 1 << 33;
const RESOURCE_CLONE_FLAGS: u64 = CLONE_PIDFD | CLONE_CLEAR_SIGHAND | CLONE_INTO_CGROUP;
const CLOSE_RANGE_CLOEXEC: usize = 1 << 2;
const AT_EMPTY_PATH: usize = 0x1000;
const KERNEL_SIGSET_BYTES: usize = mem::size_of::<u64>();
const RLIM_INFINITY: u64 = u64::MAX;
const CHILD_SETUP_MARKER: u8 = 1;
const CHILD_EXEC_MARKER: u8 = 2;
const CHILD_SETUP_EXIT: u8 = 125;
const CHILD_EXEC_EXIT: u8 = 126;

/// Failure phase for a launch whose child never became an accepted exec.
#[derive(Debug)]
pub enum LaunchError {
    /// Parent-side clone failure or closed child-trampoline setup failure.
    CloneOrSetup(io::Error),
    /// The native executable descriptor could not be executed.
    Exec(io::Error),
}

impl LaunchError {
    pub fn io_error(&self) -> &io::Error {
        match self {
            Self::CloneOrSetup(error) | Self::Exec(error) => error,
        }
    }

    pub fn into_io_error(self) -> io::Error {
        match self {
            Self::CloneOrSetup(error) | Self::Exec(error) => error,
        }
    }
}

impl fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CloneOrSetup(error) => write!(formatter, "clone or child setup failed: {error}"),
            Self::Exec(error) => write!(formatter, "child exec failed: {error}"),
        }
    }
}

impl std::error::Error for LaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.io_error())
    }
}

/// Exact process controls installed after atomic cgroup placement and before
/// the child's first `exec`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessControls {
    pub open_files: u64,
    pub address_space_bytes: u64,
}

/// Already-open identities consumed by one resource-accounted launch.
///
/// `cgroup` must name the destination cgroup-v2 directory itself, rather than
/// its `cgroup.procs` file. All three descriptors are closed in the parent
/// immediately after `clone3` returns there.
pub struct LaunchFiles {
    pub executable: File,
    pub cgroup: File,
    pub current_directory: File,
}

/// A fully allocated launch description.
///
/// Construction performs all fallible string conversion, metadata checks,
/// and descriptor mutation in the parent. [`PreparedLaunch::spawn`] creates
/// the standard-stream pipes before cloning and then uses a syscall-only child
/// path. The executable must be a native binary because `execveat` with
/// `AT_EMPTY_PATH` does not provide stable script-interpreter semantics.
pub struct PreparedLaunch {
    files: LaunchFiles,
    arguments: Vec<CString>,
    environment: Vec<CString>,
    controls: ProcessControls,
}

impl PreparedLaunch {
    pub fn new(
        files: LaunchFiles,
        arguments: Vec<OsString>,
        environment: Vec<(OsString, OsString)>,
        controls: ProcessControls,
    ) -> io::Result<Self> {
        validate_launch_files(&files)?;
        if controls.open_files < 3 || controls.address_space_bytes == 0 {
            return Err(invalid_input("process controls must preserve standard I/O"));
        }

        Ok(Self {
            files,
            arguments: encode_arguments(arguments)?,
            environment: encode_environment(environment)?,
            controls,
        })
    }

    /// Creates the child atomically in the supplied cgroup and returns only
    /// after its first `execveat` succeeds or its setup reports a closed
    /// failure.
    ///
    /// The caller must serialize launches with changes to the process-wide
    /// SIGCHLD disposition. `spawn` rejects SIGCHLD ignored or
    /// `SA_NOCLDWAIT`, either of which would violate owned-child reaping.
    pub fn spawn(self) -> Result<ResourceChild, LaunchError> {
        let cgroup = self.files.cgroup.as_raw_fd() as u64;
        self.spawn_with(RESOURCE_CLONE_FLAGS, cgroup, FixedLimits::REQUIRED)
    }

    fn spawn_with(
        self,
        clone_flags: u64,
        cgroup: u64,
        fixed_limits: FixedLimits,
    ) -> Result<ResourceChild, LaunchError> {
        validate_parent_reaping_contract().map_err(LaunchError::CloneOrSetup)?;

        let argument_pointers =
            nul_terminated_pointers(&self.arguments).map_err(LaunchError::CloneOrSetup)?;
        let environment_pointers =
            nul_terminated_pointers(&self.environment).map_err(LaunchError::CloneOrSetup)?;
        let (child_stdin, parent_stdin) =
            pipe_above_standard().map_err(LaunchError::CloneOrSetup)?;
        let (parent_stdout, child_stdout) =
            pipe_above_standard().map_err(LaunchError::CloneOrSetup)?;
        let (parent_stderr, child_stderr) =
            pipe_above_standard().map_err(LaunchError::CloneOrSetup)?;
        let (error_reader, error_writer) =
            pipe_above_standard().map_err(LaunchError::CloneOrSetup)?;

        let mut pidfd_slot = -1_i32;
        let clone_arguments = CloneArgs {
            flags: clone_flags,
            pidfd: pointer_bits(ptr::from_mut(&mut pidfd_slot)),
            child_tid: 0,
            parent_tid: 0,
            exit_signal: libc::SIGCHLD as u64,
            stack: 0,
            stack_size: 0,
            tls: 0,
            set_tid: 0,
            set_tid_size: 0,
            cgroup,
        };
        let child_plan = ChildPlan {
            executable: self.files.executable.as_raw_fd(),
            current_directory: self.files.current_directory.as_raw_fd(),
            stdin: child_stdin.as_raw_fd(),
            stdout: child_stdout.as_raw_fd(),
            stderr: child_stderr.as_raw_fd(),
            error_writer: error_writer.as_raw_fd(),
            argument_pointers: argument_pointers.as_ptr(),
            environment_pointers: environment_pointers.as_ptr(),
            controls: self.controls,
            fixed_limits,
        };

        let clone_result = raw_syscall2(
            libc::SYS_clone3,
            ptr::from_ref(&clone_arguments) as usize,
            mem::size_of::<CloneArgs>(),
        );
        if clone_result < 0 {
            return Err(LaunchError::CloneOrSetup(raw_os_error(clone_result)));
        }
        if clone_result == 0 {
            child_trampoline(&child_plan);
        }

        // The child inherited both pipe ends and all three launch identities,
        // but the parent has no further use for these ends/identities. Close
        // them before validation, protocol I/O, or any frontend cgroup wait.
        drop(error_writer);
        drop(child_stdin);
        drop(child_stdout);
        drop(child_stderr);
        drop(self);
        drop(argument_pointers);
        drop(environment_pointers);

        let raw_pid = match u32::try_from(clone_result) {
            Ok(pid) if pid != 0 => pid,
            _ => {
                if pidfd_slot >= 0 {
                    // SAFETY: CLONE_PIDFD reported success and initialized the
                    // caller-owned slot to a new descriptor.
                    let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd_slot) };
                    cleanup_pidfd_or_abort(&pidfd);
                } else {
                    // This combination is outside the clone3 ABI. There is no
                    // identity-safe descriptor with which cleanup could be
                    // proven, so fail-stop instead of risking a live child.
                    std::process::abort();
                }
                return Err(LaunchError::CloneOrSetup(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "clone3 returned an invalid PID",
                )));
            }
        };
        if pidfd_slot < 0 {
            // A successful CLONE_PIDFD call must initialize this slot. Without
            // it, numeric-PID cleanup could race an external SIGCHLD reaper.
            std::process::abort();
        }
        // SAFETY: successful clone3 with CLONE_PIDFD initialized pidfd_slot to
        // one new descriptor owned by this process.
        let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd_slot) };
        let child = ResourceChild {
            stdin: Some(File::from(parent_stdin)),
            stdout: Some(File::from(parent_stdout)),
            stderr: Some(File::from(parent_stderr)),
            pid: raw_pid,
            pidfd: Some(pidfd),
            status: None,
        };

        match fcntl_getfd(
            child
                .pidfd
                .as_ref()
                .expect("new resource child owns its pidfd"),
        )
        .map_err(os_error)
        {
            Ok(flags) if flags.contains(FdFlags::CLOEXEC) => {}
            Ok(_) => {
                return failed_spawn(
                    child,
                    LaunchError::CloneOrSetup(io::Error::other(
                        "clone3 returned a pidfd without close-on-exec",
                    )),
                );
            }
            Err(error) => {
                return failed_spawn(child, LaunchError::CloneOrSetup(error));
            }
        }

        match read_exec_result(error_reader) {
            Ok(None) => Ok(child),
            Ok(Some(CHILD_SETUP_MARKER)) => finish_reported_failure(
                child,
                CHILD_SETUP_EXIT,
                LaunchError::CloneOrSetup(io::Error::other("child setup failed")),
            ),
            Ok(Some(CHILD_EXEC_MARKER)) => finish_reported_failure(
                child,
                CHILD_EXEC_EXIT,
                LaunchError::Exec(io::Error::other("child execveat failed")),
            ),
            Ok(Some(_)) => failed_spawn(
                child,
                LaunchError::CloneOrSetup(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "child returned an invalid setup result",
                )),
            ),
            Err(error) => failed_spawn(child, LaunchError::CloneOrSetup(error)),
        }
    }

    #[cfg(test)]
    fn spawn_without_cgroup_for_test(self) -> Result<ResourceChild, LaunchError> {
        let fixed_limits = FixedLimits {
            file_size: current_hard_limit(libc::RLIMIT_FSIZE).map_err(LaunchError::CloneOrSetup)?,
            processes: current_hard_limit(libc::RLIMIT_NPROC).map_err(LaunchError::CloneOrSetup)?,
        };
        self.spawn_with(CLONE_PIDFD | CLONE_CLEAR_SIGHAND, 0, fixed_limits)
    }
}

/// Forks the caller's pending PID namespace, mounts a private read-only procfs
/// from PID 1, and execs one native payload with inherited stdout/stderr and
/// an explicit null-input descriptor.
///
/// The caller must already own its user and mount namespaces and must have
/// called `unshare(CLONE_NEWPID)`. All strings and descriptor identities are
/// prepared before `clone3`; the child path is syscall-only until `execve`.
pub fn run_pending_pid_namespace_process_with_proc(
    executable: &Path,
    arguments: Vec<OsString>,
    stdin: File,
) -> Result<ExitStatus, LaunchError> {
    run_pid_namespace_process(executable, arguments, stdin, false)
}

/// Java's private runtime additionally drops all capabilities and installs
/// the closed x86-64 syscall/thread filter after mounting its private procfs.
/// The caller must already have mapped its only UID/GID to 65534 and cleared
/// supplementary groups. This entrypoint never selects a host executable.
pub fn run_java_pid_namespace_process_with_proc(
    executable: &Path,
    arguments: Vec<OsString>,
    stdin: File,
) -> Result<ExitStatus, LaunchError> {
    if executable != Path::new("/mpk/toolchain/jdk/bin/java") {
        return Err(LaunchError::CloneOrSetup(invalid_input(
            "unregistered Java executable",
        )));
    }
    run_pid_namespace_process(executable, arguments, stdin, true)
}

fn run_pid_namespace_process(
    executable: &Path,
    arguments: Vec<OsString>,
    stdin: File,
    java: bool,
) -> Result<ExitStatus, LaunchError> {
    validate_parent_reaping_contract().map_err(LaunchError::CloneOrSetup)?;
    if !executable.is_absolute() {
        return Err(LaunchError::CloneOrSetup(invalid_input(
            "PID-namespace executable must be absolute",
        )));
    }
    let executable = CString::new(executable.as_os_str().as_bytes())
        .map_err(|_| LaunchError::CloneOrSetup(invalid_input("executable contains NUL")))?;
    let arguments = encode_arguments(arguments).map_err(LaunchError::CloneOrSetup)?;
    let environment =
        encode_environment(std::env::vars_os().collect()).map_err(LaunchError::CloneOrSetup)?;
    let argument_pointers =
        nul_terminated_pointers(&arguments).map_err(LaunchError::CloneOrSetup)?;
    let environment_pointers =
        nul_terminated_pointers(&environment).map_err(LaunchError::CloneOrSetup)?;
    let (error_reader, error_writer) = pipe_above_standard().map_err(LaunchError::CloneOrSetup)?;
    let mut pidfd_slot = -1_i32;
    let clone_arguments = CloneArgs {
        flags: CLONE_PIDFD | CLONE_CLEAR_SIGHAND,
        pidfd: pointer_bits(ptr::from_mut(&mut pidfd_slot)),
        child_tid: 0,
        parent_tid: 0,
        exit_signal: libc::SIGCHLD as u64,
        stack: 0,
        stack_size: 0,
        tls: 0,
        set_tid: 0,
        set_tid_size: 0,
        cgroup: 0,
    };
    let java_filter = java_policy::filter();
    let java_program = java_policy::program(&java_filter);
    let child_plan = PendingPidNamespacePlan {
        java_program: java.then_some(&java_program),
        executable: executable.as_ptr(),
        stdin: stdin.as_raw_fd(),
        error_writer: error_writer.as_raw_fd(),
        argument_pointers: argument_pointers.as_ptr(),
        environment_pointers: environment_pointers.as_ptr(),
    };
    let clone_result = raw_syscall2(
        libc::SYS_clone3,
        ptr::from_ref(&clone_arguments) as usize,
        mem::size_of::<CloneArgs>(),
    );
    if clone_result < 0 {
        return Err(LaunchError::CloneOrSetup(raw_os_error(clone_result)));
    }
    if clone_result == 0 {
        pending_pid_namespace_trampoline(&child_plan);
    }
    drop(error_writer);
    drop(stdin);
    drop(argument_pointers);
    drop(environment_pointers);
    drop(arguments);
    drop(environment);
    drop(executable);
    if clone_result <= 0 || pidfd_slot < 0 {
        if pidfd_slot >= 0 {
            // SAFETY: successful clone3 with CLONE_PIDFD initialized the slot.
            let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd_slot) };
            cleanup_pidfd_or_abort(&pidfd);
        } else if clone_result > 0 {
            std::process::abort();
        }
        return Err(LaunchError::CloneOrSetup(io::Error::new(
            io::ErrorKind::InvalidData,
            "clone3 returned an invalid PID identity",
        )));
    }
    // SAFETY: successful clone3 with CLONE_PIDFD initialized one owned fd.
    let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd_slot) };
    let pidfd_flags = match fcntl_getfd(&pidfd).map_err(os_error) {
        Ok(flags) => flags,
        Err(error) => {
            cleanup_pidfd_or_abort(&pidfd);
            return Err(LaunchError::CloneOrSetup(error));
        }
    };
    if !pidfd_flags.contains(FdFlags::CLOEXEC) {
        cleanup_pidfd_or_abort(&pidfd);
        return Err(LaunchError::CloneOrSetup(io::Error::other(
            "clone3 returned a pidfd without close-on-exec",
        )));
    }
    let phase = match read_exec_result(error_reader) {
        Ok(phase) => phase,
        Err(error) => {
            cleanup_pidfd_or_abort(&pidfd);
            return Err(LaunchError::CloneOrSetup(error));
        }
    };
    let status = match wait_for_pidfd(&pidfd, false) {
        Ok(Some(status)) => status,
        Ok(None) => {
            cleanup_pidfd_or_abort(&pidfd);
            return Err(LaunchError::CloneOrSetup(io::Error::other(
                "child was not reaped",
            )));
        }
        Err(error) => {
            cleanup_pidfd_or_abort(&pidfd);
            return Err(LaunchError::CloneOrSetup(error));
        }
    };
    match phase {
        None => Ok(status),
        Some(CHILD_SETUP_MARKER) if status.code() == Some(i32::from(CHILD_SETUP_EXIT)) => Err(
            LaunchError::CloneOrSetup(io::Error::other("PID namespace child setup failed")),
        ),
        Some(CHILD_EXEC_MARKER) if status.code() == Some(i32::from(CHILD_EXEC_EXIT)) => Err(
            LaunchError::Exec(io::Error::other("PID namespace child exec failed")),
        ),
        _ => Err(LaunchError::CloneOrSetup(io::Error::new(
            io::ErrorKind::InvalidData,
            "PID namespace child report disagreed with its exit status",
        ))),
    }
}

/// Pidfd-identified child plus the parent ends of its standard streams.
///
/// Waiting closes an untaken stdin first. Dropping a live handle sends
/// SIGKILL through the pidfd and reaps through that same pidfd. Any uncertain
/// drop-time cleanup is fail-stop. Tree cleanup remains the caller's job via
/// `cgroup.kill`; this type never signals a reusable numeric PID or PGID.
pub struct ResourceChild {
    pub stdin: Option<File>,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
    pid: u32,
    pidfd: Option<OwnedFd>,
    status: Option<ExitStatus>,
}

impl ResourceChild {
    /// Returns the original numeric PID for observation only.
    pub fn id(&self) -> u32 {
        self.pid
    }

    /// Borrows the pidfd while the child remains unreaped.
    pub fn pidfd(&self) -> Option<BorrowedFd<'_>> {
        self.pidfd.as_ref().map(AsFd::as_fd)
    }

    pub fn take_stdin(&mut self) -> Option<File> {
        self.stdin.take()
    }

    pub fn take_stdout(&mut self) -> Option<File> {
        self.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<File> {
        self.stderr.take()
    }

    /// Sends SIGKILL to the pidfd identity, never a numeric PID or PGID.
    pub fn kill(&self) -> io::Result<()> {
        if self.status.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "child has already been reaped",
            ));
        }
        let pidfd = self
            .pidfd
            .as_ref()
            .ok_or_else(|| io::Error::other("live child is missing its pidfd"))?;
        pidfd_send_signal(pidfd, Signal::KILL).map_err(os_error)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        let pidfd = self
            .pidfd
            .as_ref()
            .ok_or_else(|| io::Error::other("live child is missing its pidfd"))?;
        let status = wait_for_pidfd(pidfd, true)?;
        if let Some(status) = status {
            self.status = Some(status);
            self.pidfd.take();
        }
        Ok(status)
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.stdin.take();
        if let Some(status) = self.status {
            return Ok(status);
        }
        loop {
            let pidfd = self
                .pidfd
                .as_ref()
                .ok_or_else(|| io::Error::other("live child is missing its pidfd"))?;
            if let Some(status) = wait_for_pidfd(pidfd, false)? {
                self.status = Some(status);
                self.pidfd.take();
                return Ok(status);
            }
        }
    }

    /// Sends SIGKILL if needed and reaps the exact pidfd identity.
    pub fn terminate(&mut self) -> io::Result<ExitStatus> {
        if self.status.is_none() {
            match self.kill() {
                Ok(()) => {}
                Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
                Err(error) => return Err(error),
            }
        }
        self.wait()
    }
}

impl Drop for ResourceChild {
    fn drop(&mut self) {
        if self.status.is_some() {
            return;
        }
        self.stdin.take();
        let Some(pidfd) = self.pidfd.as_ref() else {
            std::process::abort();
        };
        match pidfd_send_signal(pidfd, Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => {}
            Err(_) => std::process::abort(),
        }
        match wait_for_pidfd(pidfd, false) {
            Ok(Some(status)) => {
                self.status = Some(status);
                self.pidfd.take();
            }
            Err(error) if error.raw_os_error() == Some(libc::ECHILD) => {}
            Ok(None) | Err(_) => std::process::abort(),
        }
    }
}

#[repr(C)]
struct CloneArgs {
    flags: u64,
    pidfd: u64,
    child_tid: u64,
    parent_tid: u64,
    exit_signal: u64,
    stack: u64,
    stack_size: u64,
    tls: u64,
    set_tid: u64,
    set_tid_size: u64,
    cgroup: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KernelSigaction {
    handler: u64,
    flags: u64,
    restorer: u64,
    mask: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RLimit64 {
    current: u64,
    maximum: u64,
}

#[derive(Clone, Copy)]
struct FixedLimits {
    file_size: u64,
    processes: u64,
}

impl FixedLimits {
    const REQUIRED: Self = Self {
        file_size: RLIM_INFINITY,
        processes: RLIM_INFINITY,
    };
}

struct ChildPlan {
    executable: RawFd,
    current_directory: RawFd,
    stdin: RawFd,
    stdout: RawFd,
    stderr: RawFd,
    error_writer: RawFd,
    argument_pointers: *const *const c_char,
    environment_pointers: *const *const c_char,
    controls: ProcessControls,
    fixed_limits: FixedLimits,
}

struct PendingPidNamespacePlan {
    java_program: Option<*const libc::sock_fprog>,
    executable: *const c_char,
    stdin: RawFd,
    error_writer: RawFd,
    argument_pointers: *const *const c_char,
    environment_pointers: *const *const c_char,
}

fn pending_pid_namespace_trampoline(plan: &PendingPidNamespacePlan) -> ! {
    let empty_mask = 0_u64;
    let default_action = KernelSigaction::default();
    if raw_syscall4(
        libc::SYS_rt_sigprocmask,
        libc::SIG_SETMASK as usize,
        ptr::from_ref(&empty_mask) as usize,
        0,
        KERNEL_SIGSET_BYTES,
    ) != 0
        || raw_syscall4(
            libc::SYS_rt_sigaction,
            libc::SIGPIPE as usize,
            ptr::from_ref(&default_action) as usize,
            0,
            KERNEL_SIGSET_BYTES,
        ) != 0
        || raw_syscall0(libc::SYS_getpid) != 1
        || raw_syscall3(
            libc::SYS_dup3,
            plan.stdin as usize,
            libc::STDIN_FILENO as usize,
            0,
        ) != i64::from(libc::STDIN_FILENO)
    {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    let proc_source = b"proc\0";
    let proc_target = b"/proc\0";
    let proc_type = b"proc\0";
    let proc_options = b"hidepid=2\0";
    let mount_flags = libc::MS_RDONLY | libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC;
    if raw_syscall5(
        libc::SYS_mount,
        proc_source.as_ptr() as usize,
        proc_target.as_ptr() as usize,
        proc_type.as_ptr() as usize,
        mount_flags as usize,
        proc_options.as_ptr() as usize,
    ) != 0
        || raw_syscall3(
            libc::SYS_close_range,
            3,
            u32::MAX as usize,
            CLOSE_RANGE_CLOEXEC,
        ) != 0
    {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    if let Some(program) = plan.java_program {
        if !java_policy::install(program) {
            child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
        }
    }
    let result = raw_syscall3(
        libc::SYS_execve,
        plan.executable as usize,
        plan.argument_pointers as usize,
        plan.environment_pointers as usize,
    );
    let _ = result;
    child_fail(plan.error_writer, CHILD_EXEC_MARKER, CHILD_EXEC_EXIT)
}

fn child_trampoline(plan: &ChildPlan) -> ! {
    let empty_mask = 0_u64;
    if raw_syscall4(
        libc::SYS_rt_sigprocmask,
        libc::SIG_SETMASK as usize,
        ptr::from_ref(&empty_mask) as usize,
        0,
        KERNEL_SIGSET_BYTES,
    ) != 0
    {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    // CLONE_CLEAR_SIGHAND resets caught handlers but intentionally preserves
    // ignored dispositions. Restore SIGPIPE so the exec has Command-like
    // native process semantics even when the Rust parent ignores SIGPIPE.
    let default_action = KernelSigaction::default();
    if raw_syscall4(
        libc::SYS_rt_sigaction,
        libc::SIGPIPE as usize,
        ptr::from_ref(&default_action) as usize,
        0,
        KERNEL_SIGSET_BYTES,
    ) != 0
    {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    if raw_syscall1(libc::SYS_fchdir, plan.current_directory as usize) != 0 {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    for (source, target) in [
        (plan.stdin, libc::STDIN_FILENO),
        (plan.stdout, libc::STDOUT_FILENO),
        (plan.stderr, libc::STDERR_FILENO),
    ] {
        if raw_syscall3(libc::SYS_dup3, source as usize, target as usize, 0) != i64::from(target) {
            child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
        }
    }
    if raw_syscall2(libc::SYS_setpgid, 0, 0) != 0 {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    for (resource, value) in [
        (libc::RLIMIT_CORE, 0),
        (libc::RLIMIT_NOFILE, plan.controls.open_files),
        (libc::RLIMIT_AS, plan.controls.address_space_bytes),
        (libc::RLIMIT_FSIZE, plan.fixed_limits.file_size),
        (libc::RLIMIT_NPROC, plan.fixed_limits.processes),
    ] {
        if !set_and_verify_limit(resource, value) {
            child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
        }
    }
    if raw_syscall3(
        libc::SYS_close_range,
        3,
        u32::MAX as usize,
        CLOSE_RANGE_CLOEXEC,
    ) != 0
    {
        child_fail(plan.error_writer, CHILD_SETUP_MARKER, CHILD_SETUP_EXIT);
    }
    let empty_path = b"\0";
    let _ = raw_syscall5(
        libc::SYS_execveat,
        plan.executable as usize,
        empty_path.as_ptr() as usize,
        plan.argument_pointers as usize,
        plan.environment_pointers as usize,
        AT_EMPTY_PATH,
    );
    child_fail(plan.error_writer, CHILD_EXEC_MARKER, CHILD_EXEC_EXIT)
}

fn set_and_verify_limit(resource: u32, value: u64) -> bool {
    let requested = RLimit64 {
        current: value,
        maximum: value,
    };
    if raw_syscall4(
        libc::SYS_prlimit64,
        0,
        resource as usize,
        ptr::from_ref(&requested) as usize,
        0,
    ) != 0
    {
        return false;
    }
    let mut observed = RLimit64 {
        current: 0,
        maximum: 0,
    };
    if raw_syscall4(
        libc::SYS_prlimit64,
        0,
        resource as usize,
        0,
        ptr::from_mut(&mut observed) as usize,
    ) != 0
    {
        return false;
    }
    observed.current == value && observed.maximum == value
}

#[cfg(test)]
fn current_hard_limit(resource: u32) -> io::Result<u64> {
    let mut observed = RLimit64 {
        current: 0,
        maximum: 0,
    };
    let result = raw_syscall4(
        libc::SYS_prlimit64,
        0,
        resource as usize,
        0,
        ptr::from_mut(&mut observed) as usize,
    );
    if result < 0 {
        Err(raw_os_error(result))
    } else {
        Ok(observed.maximum)
    }
}

fn child_fail(error_writer: RawFd, marker: u8, exit_code: u8) -> ! {
    loop {
        let result = raw_syscall3(
            libc::SYS_write,
            error_writer as usize,
            ptr::from_ref(&marker) as usize,
            1,
        );
        if result == 1 {
            break;
        }
        if result != -i64::from(libc::EINTR) {
            break;
        }
    }
    loop {
        let _ = raw_syscall1(libc::SYS_exit_group, usize::from(exit_code));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn raw_syscall6(
    number: libc::c_long,
    first: usize,
    second: usize,
    third: usize,
    fourth: usize,
    fifth: usize,
    sixth: usize,
) -> i64 {
    let mut result = number;
    // SAFETY: this is the frozen Linux/x86_64 syscall ABI. The caller has
    // prepared every pointer and scalar before clone; the asm neither uses the
    // Rust stack nor calls libc/TLS/allocator code in the child.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") result,
            in("rdi") first,
            in("rsi") second,
            in("rdx") third,
            in("r10") fourth,
            in("r8") fifth,
            in("r9") sixth,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    result
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn raw_syscall6(
    _number: libc::c_long,
    _first: usize,
    _second: usize,
    _third: usize,
    _fourth: usize,
    _fifth: usize,
    _sixth: usize,
) -> i64 {
    // The registered release host is Linux/x86_64. Keep other Linux targets
    // buildable, but make the launcher unavailable before it can clone.
    -i64::from(libc::ENOSYS)
}

#[inline(always)]
fn raw_syscall0(number: libc::c_long) -> i64 {
    raw_syscall6(number, 0, 0, 0, 0, 0, 0)
}

#[inline(always)]
fn raw_syscall1(number: libc::c_long, first: usize) -> i64 {
    raw_syscall6(number, first, 0, 0, 0, 0, 0)
}

#[inline(always)]
fn raw_syscall2(number: libc::c_long, first: usize, second: usize) -> i64 {
    raw_syscall6(number, first, second, 0, 0, 0, 0)
}

#[inline(always)]
fn raw_syscall3(number: libc::c_long, first: usize, second: usize, third: usize) -> i64 {
    raw_syscall6(number, first, second, third, 0, 0, 0)
}

#[inline(always)]
fn raw_syscall4(
    number: libc::c_long,
    first: usize,
    second: usize,
    third: usize,
    fourth: usize,
) -> i64 {
    raw_syscall6(number, first, second, third, fourth, 0, 0)
}

#[inline(always)]
fn raw_syscall5(
    number: libc::c_long,
    first: usize,
    second: usize,
    third: usize,
    fourth: usize,
    fifth: usize,
) -> i64 {
    raw_syscall6(number, first, second, third, fourth, fifth, 0)
}

fn validate_parent_reaping_contract() -> io::Result<()> {
    let mut action = KernelSigaction::default();
    let result = raw_syscall4(
        libc::SYS_rt_sigaction,
        libc::SIGCHLD as usize,
        0,
        ptr::from_mut(&mut action) as usize,
        KERNEL_SIGSET_BYTES,
    );
    if result < 0 {
        return Err(raw_os_error(result));
    }
    if !sigchld_is_waitable(&action) {
        return Err(io::Error::other(
            "SIGCHLD must be waitable while launching a resource child",
        ));
    }
    Ok(())
}

fn sigchld_is_waitable(action: &KernelSigaction) -> bool {
    action.handler != libc::SIG_IGN as u64 && action.flags & libc::SA_NOCLDWAIT as u64 == 0
}

fn validate_launch_files(files: &LaunchFiles) -> io::Result<()> {
    if !files.executable.metadata()?.is_file()
        || !files.cgroup.metadata()?.is_dir()
        || !files.current_directory.metadata()?.is_dir()
    {
        return Err(invalid_input(
            "executable, cgroup, or current-directory descriptor has the wrong type",
        ));
    }
    let descriptors = [
        files.executable.as_raw_fd(),
        files.cgroup.as_raw_fd(),
        files.current_directory.as_raw_fd(),
    ];
    if descriptors.iter().any(|descriptor| *descriptor < 3)
        || descriptors.into_iter().collect::<BTreeSet<_>>().len() != descriptors.len()
    {
        return Err(invalid_input(
            "launch descriptors must be distinct and above standard I/O",
        ));
    }
    for file in [&files.executable, &files.cgroup, &files.current_directory] {
        fcntl_setfd(file, FdFlags::CLOEXEC).map_err(os_error)?;
        if !fcntl_getfd(file)
            .map_err(os_error)?
            .contains(FdFlags::CLOEXEC)
        {
            return Err(io::Error::other(
                "descriptor close-on-exec verification failed",
            ));
        }
    }
    Ok(())
}

fn pipe_above_standard() -> io::Result<(OwnedFd, OwnedFd)> {
    let (reader, writer) = pipe_with(PipeFlags::CLOEXEC).map_err(os_error)?;
    Ok((
        relocate_above_standard(reader)?,
        relocate_above_standard(writer)?,
    ))
}

fn relocate_above_standard(descriptor: OwnedFd) -> io::Result<OwnedFd> {
    if descriptor.as_raw_fd() > libc::STDERR_FILENO {
        return Ok(descriptor);
    }
    fcntl_dupfd_cloexec(&descriptor, libc::STDERR_FILENO + 1).map_err(os_error)
}

fn encode_arguments(arguments: Vec<OsString>) -> io::Result<Vec<CString>> {
    if arguments
        .first()
        .is_none_or(|argument| argument.as_os_str().as_bytes().is_empty())
    {
        return Err(invalid_input("argv[0] must be present and nonempty"));
    }
    arguments
        .into_iter()
        .map(|argument| {
            CString::new(argument.as_os_str().as_bytes())
                .map_err(|_| invalid_input("argument contains NUL"))
        })
        .collect()
}

fn encode_environment(environment: Vec<(OsString, OsString)>) -> io::Result<Vec<CString>> {
    let mut keys = BTreeSet::new();
    let mut encoded = Vec::with_capacity(environment.len());
    for (key, value) in environment {
        let key = key.as_os_str().as_bytes();
        let value = value.as_os_str().as_bytes();
        if key.is_empty() || key.contains(&b'=') || key.contains(&0) || value.contains(&0) {
            return Err(invalid_input("environment entry is not exec-safe"));
        }
        if !keys.insert(key.to_vec()) {
            return Err(invalid_input("environment contains a duplicate key"));
        }
        let capacity = key
            .len()
            .checked_add(1)
            .and_then(|length| length.checked_add(value.len()))
            .ok_or_else(|| invalid_input("environment entry is too large"))?;
        let mut entry = Vec::with_capacity(capacity);
        entry.extend_from_slice(key);
        entry.push(b'=');
        entry.extend_from_slice(value);
        encoded.push(
            CString::new(entry).map_err(|_| invalid_input("environment entry contains NUL"))?,
        );
    }
    Ok(encoded)
}

fn nul_terminated_pointers(values: &[CString]) -> io::Result<Vec<*const c_char>> {
    let capacity = values
        .len()
        .checked_add(1)
        .ok_or_else(|| invalid_input("argument vector is too large"))?;
    let mut pointers = Vec::with_capacity(capacity);
    pointers.extend(values.iter().map(|value| value.as_ptr()));
    pointers.push(ptr::null());
    Ok(pointers)
}

fn read_exec_result(error_reader: OwnedFd) -> io::Result<Option<u8>> {
    let mut reader = File::from(error_reader);
    let mut result = [0_u8; 2];
    let mut observed = 0;
    loop {
        match reader.read(&mut result[observed..]) {
            Ok(0) => break,
            Ok(count) => {
                observed += count;
                if observed == result.len() {
                    return Ok(Some(0));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(match observed {
        0 => None,
        1 => Some(result[0]),
        _ => Some(0),
    })
}

fn wait_for_pidfd(pidfd: &OwnedFd, nohang: bool) -> io::Result<Option<ExitStatus>> {
    let mut options = WaitIdOptions::EXITED;
    if nohang {
        options |= WaitIdOptions::NOHANG;
    }
    loop {
        match waitid(WaitId::PidFd(pidfd.as_fd()), options) {
            Ok(Some(status)) => return waitid_exit_status(&status).map(Some),
            Ok(None) => return Ok(None),
            Err(error) if error == rustix::io::Errno::INTR => {}
            Err(error) => return Err(os_error(error)),
        }
    }
}

fn waitid_exit_status(status: &WaitIdStatus) -> io::Result<ExitStatus> {
    let raw = if let Some(code) = status.exit_status() {
        code << 8
    } else if let Some(signal) = status.terminating_signal() {
        signal | if status.dumped() { 0x80 } else { 0 }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "waitid returned a nonterminal child status",
        ));
    };
    Ok(ExitStatus::from_raw(raw))
}

fn finish_reported_failure(
    mut child: ResourceChild,
    expected_exit: u8,
    error: LaunchError,
) -> Result<ResourceChild, LaunchError> {
    match child.wait() {
        Ok(status) if status.code() == Some(i32::from(expected_exit)) => Err(error),
        Ok(_) => failed_spawn(
            child,
            LaunchError::CloneOrSetup(io::Error::new(
                io::ErrorKind::InvalidData,
                "child failure phase disagreed with its exit status",
            )),
        ),
        Err(wait_error) if wait_error.raw_os_error() == Some(libc::ECHILD) => Err(error),
        Err(_) => std::process::abort(),
    }
}

fn failed_spawn(
    mut child: ResourceChild,
    error: LaunchError,
) -> Result<ResourceChild, LaunchError> {
    match child.terminate() {
        Ok(_) => Err(error),
        Err(cleanup_error) if cleanup_error.raw_os_error() == Some(libc::ECHILD) => Err(error),
        Err(_) => std::process::abort(),
    }
}

fn cleanup_pidfd_or_abort(pidfd: &OwnedFd) {
    match pidfd_send_signal(pidfd, Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => {}
        Err(_) => std::process::abort(),
    }
    match wait_for_pidfd(pidfd, false) {
        Ok(Some(_)) => {}
        Err(error) if error.raw_os_error() == Some(libc::ECHILD) => {}
        Ok(None) | Err(_) => std::process::abort(),
    }
}

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn os_error(error: rustix::io::Errno) -> io::Error {
    io::Error::from_raw_os_error(error.raw_os_error())
}

fn raw_os_error(result: i64) -> io::Error {
    debug_assert!(result < 0);
    io::Error::from_raw_os_error((-result) as i32)
}

fn pointer_bits<T>(pointer: *const T) -> u64 {
    pointer as usize as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustix::process::{getpgid, getpid, getrlimit, Resource, Rlimit};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::Duration;

    const CHILD_MARKER_KEY: &str = "MPK_LINUX_SANDBOX_TEST_CHILD";
    const CHILD_CWD_KEY: &str = "MPK_LINUX_SANDBOX_TEST_CWD";
    static LIVE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn clone_args_v2_layout_and_flags_are_exact() {
        assert_eq!(mem::size_of::<CloneArgs>(), 88);
        assert_eq!(mem::align_of::<CloneArgs>(), 8);
        assert_eq!(mem::offset_of!(CloneArgs, flags), 0);
        assert_eq!(mem::offset_of!(CloneArgs, pidfd), 8);
        assert_eq!(mem::offset_of!(CloneArgs, child_tid), 16);
        assert_eq!(mem::offset_of!(CloneArgs, parent_tid), 24);
        assert_eq!(mem::offset_of!(CloneArgs, exit_signal), 32);
        assert_eq!(mem::offset_of!(CloneArgs, stack), 40);
        assert_eq!(mem::offset_of!(CloneArgs, stack_size), 48);
        assert_eq!(mem::offset_of!(CloneArgs, tls), 56);
        assert_eq!(mem::offset_of!(CloneArgs, set_tid), 64);
        assert_eq!(mem::offset_of!(CloneArgs, set_tid_size), 72);
        assert_eq!(mem::offset_of!(CloneArgs, cgroup), 80);
        assert_eq!(CLONE_PIDFD, 0x1000);
        assert_eq!(CLONE_CLEAR_SIGHAND, 0x1_0000_0000);
        assert_eq!(CLONE_INTO_CGROUP, 0x2_0000_0000);
        assert_eq!(
            RESOURCE_CLONE_FLAGS,
            CLONE_PIDFD | CLONE_CLEAR_SIGHAND | CLONE_INTO_CGROUP
        );
        assert_eq!(FixedLimits::REQUIRED.file_size, RLIM_INFINITY);
        assert_eq!(FixedLimits::REQUIRED.processes, RLIM_INFINITY);
    }

    #[test]
    fn kernel_sigaction_layout_is_exact_for_registered_host() {
        assert_eq!(mem::size_of::<KernelSigaction>(), 32);
        assert_eq!(mem::align_of::<KernelSigaction>(), 8);
        assert_eq!(mem::offset_of!(KernelSigaction, handler), 0);
        assert_eq!(mem::offset_of!(KernelSigaction, flags), 8);
        assert_eq!(mem::offset_of!(KernelSigaction, restorer), 16);
        assert_eq!(mem::offset_of!(KernelSigaction, mask), 24);
        assert_eq!(KERNEL_SIGSET_BYTES, 8);
    }

    #[test]
    fn sigchld_auto_reaping_dispositions_are_rejected() {
        assert!(sigchld_is_waitable(&KernelSigaction::default()));
        assert!(!sigchld_is_waitable(&KernelSigaction {
            handler: libc::SIG_IGN as u64,
            ..KernelSigaction::default()
        }));
        assert!(!sigchld_is_waitable(&KernelSigaction {
            flags: libc::SA_NOCLDWAIT as u64,
            ..KernelSigaction::default()
        }));
    }

    #[test]
    fn preparation_rejects_noncanonical_exec_vectors() {
        let _guard = live_test_guard();
        let cases = [
            (vec![], vec![]),
            (vec![OsString::new()], vec![]),
            (
                vec![OsString::from("test")],
                vec![(OsString::new(), OsString::from("value"))],
            ),
            (
                vec![OsString::from("test")],
                vec![(OsString::from("A=B"), OsString::from("value"))],
            ),
            (
                vec![OsString::from("test")],
                vec![
                    (OsString::from("A"), OsString::from("one")),
                    (OsString::from("A"), OsString::from("two")),
                ],
            ),
        ];
        for (arguments, environment) in cases {
            let error = PreparedLaunch::new(
                test_files().expect("test files open"),
                arguments,
                environment,
                test_controls(),
            )
            .err()
            .expect("invalid launch vector rejects");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn preparation_rejects_regular_file_as_cgroup() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary directory");
        let invalid_cgroup = File::create(temporary.path().join("not-a-cgroup"))
            .expect("invalid cgroup file creates");
        let files = LaunchFiles {
            executable: File::open(std::env::current_exe().expect("test executable"))
                .expect("test executable opens"),
            cgroup: invalid_cgroup,
            current_directory: File::open(temporary.path()).expect("cwd opens"),
        };
        let error = PreparedLaunch::new(
            files,
            vec![OsString::from("probe")],
            vec![],
            test_controls(),
        )
        .err()
        .expect("regular cgroup file rejects");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn clone3_exec_probe_child() {
        let Some(mode) = std::env::var_os(CHILD_MARKER_KEY) else {
            return;
        };
        if mode == "sleep" {
            std::thread::sleep(Duration::from_secs(30));
            return;
        }
        let expected_cwd = PathBuf::from(std::env::var_os(CHILD_CWD_KEY).expect("cwd is present"));
        assert_eq!(
            std::env::current_dir().expect("cwd is readable"),
            expected_cwd
        );
        assert_eq!(getpgid(None).expect("process group is readable"), getpid());
        assert_eq!(getrlimit(Resource::Core), exact_limit(0));
        assert_eq!(getrlimit(Resource::Nofile), exact_limit(64));
        assert_eq!(getrlimit(Resource::As), exact_limit(4_294_967_296));
        assert_limit_is_exact_at_hard_maximum(getrlimit(Resource::Fsize));
        assert_limit_is_exact_at_hard_maximum(getrlimit(Resource::Nproc));
        verify_only_standard_descriptors();
        println!("mpk-linux-sandbox clone3 probe ok");
    }

    #[test]
    fn clone3_child_has_exact_controls_streams_and_descriptor_closure() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary cwd");
        let launch = test_launch(temporary.path(), "probe");
        let retained_fds = [
            launch.files.executable.as_raw_fd(),
            launch.files.cgroup.as_raw_fd(),
            launch.files.current_directory.as_raw_fd(),
        ];
        let Some(mut child) = live_test_child(launch) else {
            return;
        };
        for descriptor in retained_fds {
            assert!(
                std::fs::symlink_metadata(format!("/proc/self/fd/{descriptor}")).is_err(),
                "launch descriptor {descriptor} remained open in the parent"
            );
        }
        assert!(child.id() > 0);
        assert!(fcntl_getfd(child.pidfd().expect("live child has a pidfd"))
            .expect("pidfd flags")
            .contains(FdFlags::CLOEXEC));
        drop(child.take_stdin());
        let mut stdout = child.take_stdout().expect("stdout is exposed");
        let mut stderr = child.take_stderr().expect("stderr is exposed");
        let status = child.wait().expect("child is reaped");
        assert!(status.success(), "child status: {status:?}");
        let mut stdout_bytes = Vec::new();
        stdout.read_to_end(&mut stdout_bytes).expect("stdout reads");
        let mut stderr_bytes = Vec::new();
        stderr.read_to_end(&mut stderr_bytes).expect("stderr reads");
        assert!(
            String::from_utf8_lossy(&stdout_bytes).contains("mpk-linux-sandbox clone3 probe ok"),
            "stdout: {}",
            String::from_utf8_lossy(&stdout_bytes)
        );
        assert!(
            stderr_bytes.is_empty(),
            "stderr: {}",
            String::from_utf8_lossy(&stderr_bytes)
        );
    }

    #[test]
    fn pidfd_try_wait_kill_and_reap_are_identity_safe() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary cwd");
        let Some(mut child) = live_test_child(test_launch(temporary.path(), "sleep")) else {
            return;
        };
        assert_eq!(child.try_wait().expect("try_wait works"), None);
        child.kill().expect("pidfd kill works");
        let status = child.wait().expect("pidfd wait reaps");
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        assert!(child.pidfd().is_none(), "reaping closes the pidfd");
        assert_eq!(child.try_wait().expect("cached status works"), Some(status));
        assert_eq!(
            child
                .kill()
                .expect_err("reaped child cannot be killed")
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn dropping_a_live_handle_kills_and_reaps_by_pidfd() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary cwd");
        let Some(child) = live_test_child(test_launch(temporary.path(), "sleep")) else {
            return;
        };
        let observer = fcntl_dupfd_cloexec(child.pidfd().expect("live child has a pidfd"), 3)
            .expect("pidfd duplicates");
        drop(child);
        let error = waitid(
            WaitId::PidFd(observer.as_fd()),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
        )
        .expect_err("drop reaped the child");
        assert_eq!(error, rustix::io::Errno::CHILD);
    }

    #[test]
    fn execveat_failure_has_a_typed_exec_phase() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary cwd");
        let executable_path = temporary.path().join("not-an-executable");
        std::fs::write(&executable_path, b"not an executable\n").expect("fixture writes");
        let launch = PreparedLaunch::new(
            LaunchFiles {
                executable: File::open(&executable_path).expect("fixture opens"),
                cgroup: File::open(temporary.path()).expect("placeholder cgroup opens"),
                current_directory: File::open(temporary.path()).expect("cwd opens"),
            },
            vec![OsString::from("not-an-executable")],
            vec![],
            test_controls(),
        )
        .expect("launch prepares");
        match launch.spawn_without_cgroup_for_test() {
            Err(LaunchError::Exec(_)) => {}
            Err(error) if clone_unavailable(&error) => {}
            Err(error) => panic!("wrong launch phase: {error}"),
            Ok(mut child) => {
                let _ = child.terminate();
                panic!("invalid native image unexpectedly executed")
            }
        }
    }

    #[test]
    fn production_clone_rejects_a_non_cgroup_directory() {
        let _guard = live_test_guard();
        let temporary = tempfile::tempdir().expect("temporary cwd");
        let launch = test_launch(temporary.path(), "probe");
        match launch.spawn() {
            Err(error) if clone_unavailable(&error) => {}
            Err(LaunchError::CloneOrSetup(error))
                if [libc::EBADF, libc::EINVAL, libc::ENODEV, libc::EOPNOTSUPP]
                    .contains(&error.raw_os_error().unwrap_or_default()) => {}
            Err(error) => panic!("unexpected non-cgroup clone error: {error}"),
            Ok(mut child) => {
                let _ = child.terminate();
                panic!("clone3 accepted a non-cgroup directory")
            }
        }
    }

    fn live_test_child(launch: PreparedLaunch) -> Option<ResourceChild> {
        if !live_limits_supported() {
            return None;
        }
        match launch.spawn_without_cgroup_for_test() {
            Ok(child) => Some(child),
            Err(error) if clone_unavailable(&error) => {
                eprintln!("clone3 unavailable for live test: {error}");
                None
            }
            Err(error) => panic!("clone3 launch failed: {error}"),
        }
    }

    fn live_test_guard() -> std::sync::MutexGuard<'static, ()> {
        LIVE_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn clone_unavailable(error: &LaunchError) -> bool {
        match error {
            LaunchError::CloneOrSetup(error) => [libc::ENOSYS, libc::EPERM, libc::EACCES]
                .contains(&error.raw_os_error().unwrap_or_default()),
            LaunchError::Exec(_) => false,
        }
    }

    fn live_limits_supported() -> bool {
        let requirements = [
            (Resource::Core, 0),
            (Resource::Nofile, 64),
            (Resource::As, 4_294_967_296),
        ];
        requirements.into_iter().all(|(resource, requested)| {
            let current = getrlimit(resource);
            match (requested, current.maximum) {
                (RLIM_INFINITY, None) => true,
                (RLIM_INFINITY, Some(_)) => false,
                (value, Some(maximum)) => maximum >= value,
                (_, None) => true,
            }
        })
    }

    fn test_launch(directory: &Path, mode: &str) -> PreparedLaunch {
        let executable_path = std::env::current_exe().expect("test executable path");
        let arguments = vec![
            executable_path.clone().into_os_string(),
            OsString::from("--exact"),
            OsString::from("tests::clone3_exec_probe_child"),
            OsString::from("--nocapture"),
            OsString::from("--test-threads=1"),
        ];
        let environment = vec![
            (OsString::from(CHILD_MARKER_KEY), OsString::from(mode)),
            (
                OsString::from(CHILD_CWD_KEY),
                directory.as_os_str().to_owned(),
            ),
        ];
        PreparedLaunch::new(
            LaunchFiles {
                executable: File::open(executable_path).expect("test executable opens"),
                cgroup: File::open(directory).expect("placeholder cgroup directory opens"),
                current_directory: File::open(directory).expect("current directory opens"),
            },
            arguments,
            environment,
            test_controls(),
        )
        .expect("launch prepares")
    }

    fn test_files() -> io::Result<LaunchFiles> {
        let executable = File::open(std::env::current_exe()?)?;
        let directory = File::open(std::env::current_dir()?)?;
        Ok(LaunchFiles {
            executable,
            cgroup: directory.try_clone()?,
            current_directory: directory,
        })
    }

    fn test_controls() -> ProcessControls {
        ProcessControls {
            open_files: 64,
            address_space_bytes: 4_294_967_296,
        }
    }

    fn exact_limit(value: u64) -> Rlimit {
        Rlimit {
            current: Some(value),
            maximum: Some(value),
        }
    }

    fn assert_limit_is_exact_at_hard_maximum(limit: Rlimit) {
        assert_eq!(limit.current, limit.maximum);
    }

    fn verify_only_standard_descriptors() {
        let descriptors = std::fs::read_dir("/proc/self/fd")
            .expect("descriptor directory opens")
            .map(|entry| {
                entry
                    .expect("descriptor entry")
                    .file_name()
                    .to_str()
                    .and_then(|name| name.parse::<RawFd>().ok())
                    .expect("numeric descriptor")
            })
            .collect::<Vec<_>>();
        for descriptor in descriptors.into_iter().filter(|descriptor| *descriptor > 2) {
            assert!(
                std::fs::symlink_metadata(format!("/proc/self/fd/{descriptor}")).is_err(),
                "descriptor {descriptor} survived exec"
            );
        }
    }
}
