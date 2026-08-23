#![allow(dead_code)]

#[cfg(any(test, target_os = "linux"))]
use crate::frontend_protocol::{FRONTEND_STDERR_BYTES_MAX, FRONTEND_STDOUT_BYTES_MAX};
use crate::frontend_registry::BundleSnapshot;
#[cfg(test)]
use crate::frontend_registry::SnapshotFile;
use mpk_vc::CapturedInput;
use std::collections::BTreeMap;
#[cfg(target_os = "linux")]
use std::collections::BTreeSet;
#[cfg(any(test, target_os = "linux"))]
use std::fs;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(any(test, target_os = "linux"))]
use std::io::{self, Read};
#[cfg(any(test, target_os = "linux"))]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::{Component, PathBuf};
#[cfg(any(test, target_os = "linux"))]
use std::process::{Command, Stdio};
#[cfg(any(test, target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(test, target_os = "linux"))]
use std::sync::Arc;
#[cfg(any(test, target_os = "linux"))]
use std::thread;
#[cfg(any(test, target_os = "linux"))]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant;

#[cfg(any(test, target_os = "linux"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(any(test, target_os = "linux"))]
use std::os::unix::process::ExitStatusExt;

#[derive(Debug)]
pub(crate) struct SandboxOutput {
    pub(crate) exit_code: Option<i32>,
    pub(crate) signaled: bool,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr_observed_bytes: usize,
    pub(crate) stream_limit_exceeded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SandboxError {
    Unavailable,
    Spawn,
    Killed,
}

/// The release-facing sandbox stays fail-closed until entered through the
/// Linux namespace bootstrap. The v1 CLI does not call this boundary before
/// GO-VIR-02-T12.
pub(crate) fn launch_release_frontend(
    frontend: &BundleSnapshot,
    toolchain: &BundleSnapshot,
    executable: &str,
    args: &[String],
    environment: &BTreeMap<String, String>,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<SandboxOutput, SandboxError> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (
            frontend,
            toolchain,
            executable,
            args,
            environment,
            captured_inputs,
        );
        Err(SandboxError::Unavailable)
    }
    #[cfg(target_os = "linux")]
    {
        if !matches!(executable, "bin/go2vir" | "bin/rust2vir") {
            return Err(SandboxError::Unavailable);
        }
        run_sandbox_probe()?;
        let temporary = tempfile::Builder::new()
            .prefix("mpk-frontend-sandbox-")
            .tempdir_in("/tmp")
            .map_err(|_| SandboxError::Unavailable)?;
        let root = temporary.path();
        materialize_snapshot(frontend, &root.join("mpk/frontend"))?;
        materialize_snapshot(toolchain, &root.join("mpk/toolchain"))?;
        materialize_sources(captured_inputs, &root.join("mpk/source"))?;
        for relative in [
            "dev",
            "mpk/cache/go-build",
            "mpk/cache/go-mod",
            "mpk/gopath",
            "mpk/empty/home",
            "mpk/native-runtime",
            "mpk/tmp",
        ] {
            fs::create_dir_all(root.join(relative)).map_err(|_| SandboxError::Unavailable)?;
        }
        fs::write(root.join("dev/null"), b"").map_err(|_| SandboxError::Unavailable)?;
        if executable == "bin/rust2vir" {
            for relative in ["proc", "lib64", "lib/x86_64-linux-gnu"] {
                fs::create_dir_all(root.join(relative)).map_err(|_| SandboxError::Unavailable)?;
            }
            fs::write(root.join("lib64/ld-linux-x86-64.so.2"), b"")
                .map_err(|_| SandboxError::Unavailable)?;
        }
        seal_read_only_tree(&root.join("mpk/frontend"))?;
        seal_read_only_tree(&root.join("mpk/toolchain"))?;
        seal_read_only_tree(&root.join("mpk/source"))?;
        for relative in [
            "mpk/cache/go-mod",
            "mpk/gopath",
            "mpk/empty/home",
            "mpk/native-runtime",
        ] {
            seal_read_only_tree(&root.join(relative))?;
        }
        let current_executable = std::env::current_exe().map_err(|_| SandboxError::Unavailable)?;
        let mut bootstrap_args = Vec::with_capacity(args.len() + 2);
        bootstrap_args.push("__mpk_frontend_sandbox_v0".to_owned());
        bootstrap_args.push(executable.to_owned());
        bootstrap_args.extend_from_slice(args);
        let result =
            match run_closed_process(&current_executable, root, &bootstrap_args, environment) {
                Ok(output) if output.exit_code == Some(125) => Err(SandboxError::Unavailable),
                Ok(output) if output.exit_code == Some(126) => Err(SandboxError::Spawn),
                output => output,
            };
        unseal_private_tree(root)?;
        temporary.close().map_err(|_| SandboxError::Unavailable)?;
        result
    }
}

#[cfg(target_os = "linux")]
fn run_sandbox_probe() -> Result<(), SandboxError> {
    let temporary = tempfile::Builder::new()
        .prefix("mpk-frontend-probe-")
        .tempdir_in("/tmp")
        .map_err(|_| SandboxError::Unavailable)?;
    let executable = std::env::current_exe().map_err(|_| SandboxError::Unavailable)?;
    let mut child = Command::new(executable)
        .arg("__mpk_frontend_probe_v0")
        .env_clear()
        .current_dir(temporary.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| SandboxError::Unavailable)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(SandboxError::Unavailable);
            }
        }
    };
    let output = child
        .wait_with_output()
        .map_err(|_| SandboxError::Unavailable)?;
    let result = if !status.success()
        || output.stdout != b"mpk.release.probe.v0 ok\n"
        || !output.stderr.is_empty()
        || output.stdout.len() + output.stderr.len() > 512
    {
        Err(SandboxError::Unavailable)
    } else {
        Ok(())
    };
    temporary.close().map_err(|_| SandboxError::Unavailable)?;
    result
}

#[cfg(test)]
pub(crate) fn launch_snapshot_for_test(
    executable: &SnapshotFile,
    args: &[String],
    environment: &BTreeMap<String, String>,
) -> Result<SandboxOutput, SandboxError> {
    let temporary = tempfile::Builder::new()
        .prefix("mpk-frontend-snapshot-")
        .tempdir_in("/tmp")
        .map_err(|_| SandboxError::Unavailable)?;
    let executable_path = temporary.path().join("frontend");
    fs::write(&executable_path, &executable.bytes).map_err(|_| SandboxError::Unavailable)?;
    set_executable(&executable_path).map_err(|_| SandboxError::Unavailable)?;
    run_closed_process(&executable_path, temporary.path(), args, environment)
}

#[cfg(any(test, target_os = "linux"))]
fn run_closed_process(
    executable: &Path,
    current_directory: &Path,
    args: &[String],
    environment: &BTreeMap<String, String>,
) -> Result<SandboxOutput, SandboxError> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .env_clear()
        .envs(environment)
        .current_dir(current_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "linux")]
    command.process_group(0);
    let mut child = command.spawn().map_err(|_| SandboxError::Spawn)?;
    let child_id = child.id();
    let stdout = child.stdout.take().ok_or(SandboxError::Spawn)?;
    let stderr = child.stderr.take().ok_or(SandboxError::Spawn)?;
    let overflow = Arc::new(AtomicBool::new(false));
    let read_failed = Arc::new(AtomicBool::new(false));
    let stdout_reader = bounded_reader(
        stdout,
        FRONTEND_STDOUT_BYTES_MAX,
        true,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let stderr_reader = bounded_reader(
        stderr,
        FRONTEND_STDERR_BYTES_MAX,
        false,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            kill_child_tree(&mut child);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(2)),
            Err(_) => {
                kill_child_tree(&mut child);
                return Err(SandboxError::Killed);
            }
        }
    };
    kill_process_group(child_id);
    let (stdout, _) = stdout_reader.join().map_err(|_| SandboxError::Killed)?;
    let (_, stderr_observed_bytes) = stderr_reader.join().map_err(|_| SandboxError::Killed)?;
    if read_failed.load(Ordering::Acquire) {
        return Err(SandboxError::Killed);
    }
    Ok(SandboxOutput {
        exit_code: status.code(),
        signaled: exit_was_signaled(&status),
        stdout,
        stderr_observed_bytes,
        stream_limit_exceeded: overflow.load(Ordering::Acquire),
    })
}

#[cfg(target_os = "linux")]
fn kill_child_tree(child: &mut std::process::Child) {
    kill_process_group(child.id());
    let _ = child.kill();
}

#[cfg(all(test, not(target_os = "linux")))]
fn kill_child_tree(child: &mut std::process::Child) {
    let _ = child.kill();
}

#[cfg(target_os = "linux")]
fn kill_process_group(raw_pid: u32) {
    if let Ok(raw_pid) = i32::try_from(raw_pid) {
        if let Some(pid) = rustix::process::Pid::from_raw(raw_pid) {
            let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
        }
    }
}

#[cfg(all(test, not(target_os = "linux")))]
fn kill_process_group(_raw_pid: u32) {}

#[cfg(any(test, target_os = "linux"))]
fn bounded_reader<R: Read + Send + 'static>(
    mut reader: R,
    maximum: usize,
    retain: bool,
    overflow: Arc<AtomicBool>,
    read_failed: Arc<AtomicBool>,
) -> thread::JoinHandle<(Vec<u8>, usize)> {
    thread::spawn(move || {
        let mut retained = Vec::new();
        let mut observed = 0usize;
        let mut block = [0u8; 16 * 1024];
        loop {
            match reader.read(&mut block) {
                Ok(0) => break,
                Err(_) => {
                    read_failed.store(true, Ordering::Release);
                    break;
                }
                Ok(count) => {
                    observed = observed.saturating_add(count);
                    if observed > maximum {
                        overflow.store(true, Ordering::Release);
                    }
                    if retain && retained.len() < maximum + 1 {
                        let remaining = maximum + 1 - retained.len();
                        retained.extend_from_slice(&block[..count.min(remaining)]);
                    }
                }
            }
        }
        (retained, observed)
    })
}

#[cfg(any(test, target_os = "linux"))]
fn set_executable(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o500))
}

#[cfg(all(test, not(unix)))]
fn set_executable(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "test executable snapshots require Unix",
    ))
}

#[cfg(any(test, target_os = "linux"))]
fn exit_was_signaled(status: &std::process::ExitStatus) -> bool {
    status.signal().is_some()
}

#[cfg(all(test, not(unix)))]
fn exit_was_signaled(_status: &std::process::ExitStatus) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn materialize_snapshot(snapshot: &BundleSnapshot, root: &Path) -> Result<(), SandboxError> {
    fs::create_dir_all(root).map_err(|_| SandboxError::Unavailable)?;
    for (relative, file) in snapshot.files() {
        let path = materialized_path(root, relative)?;
        let parent = path.parent().ok_or(SandboxError::Unavailable)?;
        fs::create_dir_all(parent).map_err(|_| SandboxError::Unavailable)?;
        fs::write(&path, &file.bytes).map_err(|_| SandboxError::Unavailable)?;
        fs::set_permissions(
            &path,
            fs::Permissions::from_mode(if file.executable { 0o555 } else { 0o444 }),
        )
        .map_err(|_| SandboxError::Unavailable)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn materialize_sources(inputs: &[CapturedInput<'_>], root: &Path) -> Result<(), SandboxError> {
    fs::create_dir_all(root).map_err(|_| SandboxError::Unavailable)?;
    let mut folded_paths = BTreeSet::new();
    for input in inputs {
        mpk_vc::validate_manifest_normalized_path(input.normalized_path)
            .map_err(|_| SandboxError::Unavailable)?;
        if !folded_paths.insert(input.normalized_path.to_ascii_lowercase()) {
            return Err(SandboxError::Unavailable);
        }
        let path = materialized_path(root, input.normalized_path)?;
        let parent = path.parent().ok_or(SandboxError::Unavailable)?;
        fs::create_dir_all(parent).map_err(|_| SandboxError::Unavailable)?;
        fs::write(&path, input.bytes).map_err(|_| SandboxError::Unavailable)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444))
            .map_err(|_| SandboxError::Unavailable)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn seal_read_only_tree(root: &Path) -> Result<(), SandboxError> {
    let mut directories = fs::read_dir(root)
        .map_err(|_| SandboxError::Unavailable)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SandboxError::Unavailable)?;
    directories.sort();
    for path in directories {
        if fs::symlink_metadata(&path)
            .map_err(|_| SandboxError::Unavailable)?
            .is_dir()
        {
            seal_read_only_tree(&path)?;
        }
    }
    fs::set_permissions(root, fs::Permissions::from_mode(0o555))
        .map_err(|_| SandboxError::Unavailable)
}

#[cfg(target_os = "linux")]
fn unseal_private_tree(root: &Path) -> Result<(), SandboxError> {
    let metadata = fs::symlink_metadata(root).map_err(|_| SandboxError::Unavailable)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(SandboxError::Unavailable);
    }
    let mut entries = fs::read_dir(root)
        .map_err(|_| SandboxError::Unavailable)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SandboxError::Unavailable)?;
    entries.sort();
    for path in entries {
        let metadata = fs::symlink_metadata(&path).map_err(|_| SandboxError::Unavailable)?;
        if metadata.file_type().is_symlink() {
            return Err(SandboxError::Unavailable);
        }
        if metadata.is_dir() {
            unseal_private_tree(&path)?;
        } else if metadata.is_file() {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|_| SandboxError::Unavailable)?;
        } else {
            return Err(SandboxError::Unavailable);
        }
    }
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))
        .map_err(|_| SandboxError::Unavailable)
}

#[cfg(target_os = "linux")]
fn materialized_path(root: &Path, relative: &str) -> Result<PathBuf, SandboxError> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.contains('\\')
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(root.join(path))
}

pub(crate) fn run_bootstrap(arguments: &[String]) -> u8 {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = arguments;
        125
    }
    #[cfg(target_os = "linux")]
    {
        linux_bootstrap(arguments).unwrap_or(125)
    }
}

pub(crate) fn run_probe() -> u8 {
    #[cfg(not(target_os = "linux"))]
    {
        125
    }
    #[cfg(target_os = "linux")]
    {
        linux_probe().unwrap_or(125)
    }
}

#[cfg(target_os = "linux")]
#[allow(deprecated)]
fn linux_probe() -> Result<u8, u8> {
    use rustix::fs::{
        openat2, renameat_with, statvfs, Mode, OFlags, RenameFlags, ResolveFlags,
        StatVfsMountFlags, CWD,
    };
    use rustix::mount::{
        mount_bind, mount_change, mount_remount, MountFlags, MountPropagationFlags,
    };
    use rustix::thread::{set_no_new_privs, unshare, UnshareFlags};
    use std::io::Read as _;
    use std::os::unix::fs::symlink;

    validate_minimum_kernel()?;
    let uid = rustix::process::getuid().as_raw();
    let gid = rustix::process::getgid().as_raw();
    unshare(UnshareFlags::NEWUSER).map_err(|_| 125)?;
    fs::write("/proc/self/setgroups", b"deny\n").map_err(|_| 125)?;
    fs::write("/proc/self/uid_map", format!("0 {uid} 1\n")).map_err(|_| 125)?;
    fs::write("/proc/self/gid_map", format!("0 {gid} 1\n")).map_err(|_| 125)?;
    unshare(
        UnshareFlags::NEWNS | UnshareFlags::NEWNET | UnshareFlags::NEWIPC | UnshareFlags::NEWUTS,
    )
    .map_err(|_| 125)?;
    set_no_new_privs(true).map_err(|_| 125)?;
    mount_change(
        "/",
        MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
    )
    .map_err(|_| 125)?;
    let root = std::env::current_dir().map_err(|_| 125)?;
    mount_bind(&root, &root).map_err(|_| 125)?;

    let interfaces = fs::read_to_string("/proc/net/dev")
        .map_err(|_| 125)?
        .lines()
        .skip(2)
        .map(|line| line.split_once(':').map(|(name, _)| name.trim().to_owned()))
        .collect::<Option<BTreeSet<_>>>()
        .ok_or(125)?;
    if interfaces != BTreeSet::from(["lo".to_owned()]) {
        return Err(125);
    }

    fs::create_dir("mount").map_err(|_| 125)?;
    fs::write("mount/read-only", b"sealed").map_err(|_| 125)?;
    fs::write("mount/no-exec", b"#!/bin/sh\nexit 0\n").map_err(|_| 125)?;
    fs::set_permissions("mount/no-exec", fs::Permissions::from_mode(0o555)).map_err(|_| 125)?;
    mount_bind("mount", "mount").map_err(|_| 125)?;
    mount_remount(
        "mount",
        MountFlags::BIND
            | MountFlags::RDONLY
            | MountFlags::NOSUID
            | MountFlags::NODEV
            | MountFlags::NOEXEC,
        "",
    )
    .map_err(|_| 125)?;
    match fs::write("mount/read-only", b"changed") {
        Err(error) if error.raw_os_error() == Some(rustix::io::Errno::ROFS.raw_os_error()) => {}
        _ => return Err(125),
    }
    let mount_state = statvfs("mount").map_err(|_| 125)?;
    if !mount_state
        .f_flag
        .contains(StatVfsMountFlags::RDONLY | StatVfsMountFlags::NOEXEC)
    {
        return Err(125);
    }
    let exec_error = Command::new("./mount/no-exec").env_clear().exec();
    if exec_error.raw_os_error() != Some(rustix::io::Errno::ACCESS.raw_os_error()) {
        return Err(125);
    }

    fs::write("nofollow-target", b"target").map_err(|_| 125)?;
    symlink("nofollow-target", "nofollow-link").map_err(|_| 125)?;
    if !matches!(
        openat2(
            CWD,
            "nofollow-link",
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
            ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
        ),
        Err(rustix::io::Errno::LOOP)
    ) {
        return Err(125);
    }

    fs::write("identity", b"retained").map_err(|_| 125)?;
    let mut retained = File::open("identity").map_err(|_| 125)?;
    fs::rename("identity", "identity-old").map_err(|_| 125)?;
    fs::write("identity", b"replacement").map_err(|_| 125)?;
    let mut retained_bytes = Vec::new();
    retained.read_to_end(&mut retained_bytes).map_err(|_| 125)?;
    if retained_bytes != b"retained" {
        return Err(125);
    }

    fs::write("publish-source", b"source").map_err(|_| 125)?;
    fs::write("publish-occupied", b"occupied").map_err(|_| 125)?;
    if !matches!(
        renameat_with(
            CWD,
            "publish-source",
            CWD,
            "publish-occupied",
            RenameFlags::NOREPLACE,
        ),
        Err(rustix::io::Errno::EXIST)
    ) || fs::read("publish-source").map_err(|_| 125)? != b"source"
        || fs::read("publish-occupied").map_err(|_| 125)? != b"occupied"
    {
        return Err(125);
    }

    println!("mpk.release.probe.v0 ok");
    Ok(0)
}

#[cfg(target_os = "linux")]
fn validate_minimum_kernel() -> Result<(), u8> {
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").map_err(|_| 125)?;
    let numeric = release
        .trim()
        .split(['.', '-'])
        .take(3)
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| 125)?;
    if numeric.len() != 3 || numeric.as_slice() < &[5, 10, 0] {
        return Err(125);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[allow(deprecated)]
fn linux_bootstrap(arguments: &[String]) -> Result<u8, u8> {
    use rustix::mount::{
        mount, mount_bind, mount_change, mount_remount, MountFlags, MountPropagationFlags,
    };
    use rustix::process::{setrlimit, Resource, Rlimit};
    use rustix::thread::{set_no_new_privs, unshare, UnshareFlags};

    let Some((executable, arguments)) = arguments.split_first() else {
        return Err(125);
    };
    if !matches!(executable.as_str(), "bin/go2vir" | "bin/rust2vir") {
        return Err(125);
    }
    let uid = rustix::process::getuid().as_raw();
    let gid = rustix::process::getgid().as_raw();
    unshare(UnshareFlags::NEWUSER).map_err(|_| 125)?;
    fs::write("/proc/self/setgroups", b"deny\n").map_err(|_| 125)?;
    fs::write("/proc/self/uid_map", format!("0 {uid} 1\n")).map_err(|_| 125)?;
    fs::write("/proc/self/gid_map", format!("0 {gid} 1\n")).map_err(|_| 125)?;
    unshare(
        UnshareFlags::NEWNS
            | UnshareFlags::NEWNET
            | UnshareFlags::NEWPID
            | UnshareFlags::NEWIPC
            | UnshareFlags::NEWUTS,
    )
    .map_err(|_| 125)?;
    mount_change(
        "/",
        MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
    )
    .map_err(|_| 125)?;
    let root = std::env::current_dir().map_err(|_| 125)?;
    mount_bind(&root, &root).map_err(|_| 125)?;
    let temporary_options = if executable == "bin/rust2vir" {
        c"size=21474836480,mode=700"
    } else {
        c"size=536870912,mode=700"
    };
    for (relative, options) in [
        ("mpk/cache/go-build", c"size=536870912,mode=700"),
        ("mpk/tmp", temporary_options),
    ] {
        let target = root.join(relative);
        mount(
            "tmpfs",
            &target,
            "tmpfs",
            MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC,
            Some(options),
        )
        .map_err(|_| 125)?;
    }
    let source = root.join("mpk/source");
    mount_bind(&source, &source).map_err(|_| 125)?;
    mount_remount(
        &source,
        MountFlags::BIND
            | MountFlags::RDONLY
            | MountFlags::NOSUID
            | MountFlags::NODEV
            | MountFlags::NOEXEC,
        "",
    )
    .map_err(|_| 125)?;
    let null = fs::metadata("/dev/null").map_err(|_| 125)?;
    if !null.file_type().is_char_device() || null.rdev() != 259 {
        return Err(125);
    }
    let null_target = root.join("dev/null");
    mount_bind("/dev/null", &null_target).map_err(|_| 125)?;
    let mounted_null = fs::metadata(&null_target).map_err(|_| 125)?;
    if mounted_null.dev() != null.dev()
        || mounted_null.ino() != null.ino()
        || mounted_null.rdev() != null.rdev()
    {
        return Err(125);
    }
    mount_remount(
        &null_target,
        MountFlags::BIND | MountFlags::NOSUID | MountFlags::NOEXEC,
        "",
    )
    .map_err(|_| 125)?;
    if executable == "bin/rust2vir" {
        let runtime = root.join("mpk/toolchain/native-runtime");
        let libraries = runtime.join("lib/x86_64-linux-gnu");
        let library_target = root.join("lib/x86_64-linux-gnu");
        mount_bind(&libraries, &library_target).map_err(|_| 125)?;
        mount_remount(
            &library_target,
            MountFlags::BIND | MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NODEV,
            "",
        )
        .map_err(|_| 125)?;
        let loader = runtime.join("lib64/ld-linux-x86-64.so.2");
        let loader_target = root.join("lib64/ld-linux-x86-64.so.2");
        mount_bind(&loader, &loader_target).map_err(|_| 125)?;
        mount_remount(
            &loader_target,
            MountFlags::BIND | MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NODEV,
            "",
        )
        .map_err(|_| 125)?;
    }
    mount_remount(
        &root,
        MountFlags::BIND | MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NODEV,
        "",
    )
    .map_err(|_| 125)?;
    rustix::process::chroot(&root).map_err(|_| 125)?;
    std::env::set_current_dir("/").map_err(|_| 125)?;
    set_no_new_privs(true).map_err(|_| 125)?;
    let (file_size_limit, open_file_limit, address_space_limit) = if executable == "bin/rust2vir" {
        (17_179_869_184, 1_024, 17_179_869_184)
    } else {
        (536_870_912, 256, 4_294_967_296)
    };
    for (resource, limit) in [
        (Resource::Core, 0),
        (Resource::Fsize, file_size_limit),
        (Resource::Nofile, open_file_limit),
        (Resource::Nproc, 256),
        (Resource::As, address_space_limit),
    ] {
        setrlimit(
            resource,
            Rlimit {
                current: Some(limit),
                maximum: Some(limit),
            },
        )
        .map_err(|_| 125)?;
    }
    let mut command = if executable == "bin/rust2vir" {
        let mut command = Command::new("/lib64/ld-linux-x86-64.so.2");
        command.args([
            "--library-path",
            "/lib/x86_64-linux-gnu",
            "/mpk/frontend/bin/rust2vir",
            "__rust2vir_outer_sandbox_v0",
        ]);
        command
    } else {
        Command::new("/mpk/frontend/bin/go2vir")
    };
    let status = command
        .args(arguments)
        .stdin(Stdio::null())
        .status()
        .map_err(|_| 126)?;
    if status.signal().is_some() {
        let _ =
            rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL);
        return Err(125);
    }
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .ok_or(125)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn frontend_runner_stream_capture_marks_the_first_excess_byte() {
        let overflow = Arc::new(AtomicBool::new(false));
        let read_failed = Arc::new(AtomicBool::new(false));
        let input = vec![b'x'; FRONTEND_STDERR_BYTES_MAX + 1];
        let reader = bounded_reader(
            Cursor::new(input),
            FRONTEND_STDERR_BYTES_MAX,
            true,
            Arc::clone(&overflow),
            Arc::clone(&read_failed),
        );
        let (retained, observed) = reader.join().expect("reader joins");
        assert_eq!(observed, FRONTEND_STDERR_BYTES_MAX + 1);
        assert_eq!(retained.len(), FRONTEND_STDERR_BYTES_MAX + 1);
        assert!(overflow.load(Ordering::Acquire));
        assert!(!read_failed.load(Ordering::Acquire));
    }
}
