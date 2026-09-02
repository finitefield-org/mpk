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
#[cfg(target_os = "linux")]
use std::ffi::OsString;
#[cfg(any(test, target_os = "linux"))]
use std::fs;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::Write;
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
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU64, AtomicU8};
#[cfg(any(test, target_os = "linux"))]
use std::sync::Arc;
#[cfg(any(test, target_os = "linux"))]
use std::thread;
#[cfg(any(test, target_os = "linux"))]
use std::time::Duration;
#[cfg(any(test, target_os = "linux"))]
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

#[cfg(any(test, target_os = "linux"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExecutionKillCause {
    StreamLimit,
    Timeout,
}

#[cfg(any(test, target_os = "linux"))]
fn first_execution_kill_cause(
    current: Option<ExecutionKillCause>,
    stream_limit_exceeded: bool,
    deadline_reached: bool,
) -> Option<ExecutionKillCause> {
    current.or({
        if stream_limit_exceeded {
            Some(ExecutionKillCause::StreamLimit)
        } else if deadline_reached {
            Some(ExecutionKillCause::Timeout)
        } else {
            None
        }
    })
}

pub(crate) const LEGACY_EXECUTION_HOST_PROFILE_ID: &str = "mpk.host.linux-x86_64-gnu.v0";
pub(crate) const RUST_EXECUTION_HOST_PROFILE_ID: &str =
    "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0";
const CSHARP_BOOTSTRAP_EXECUTABLE: &str = "dotnet/dotnet";
const JAVA_BOOTSTRAP_EXECUTABLE: &str = "jdk/bin/java";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SandboxProfile {
    LegacyNamespaces,
    Cgroup2Tmpfs,
    Java25,
}

impl SandboxProfile {
    fn resource(self) -> bool {
        self != Self::LegacyNamespaces
    }

    #[cfg(any(test, target_os = "linux"))]
    fn memory(self) -> u64 {
        if self == Self::Java25 {
            mpk_vc::java_release::MEMORY_BYTES
        } else {
            CGROUP_MEMORY_LIMIT
        }
    }

    #[cfg(any(test, target_os = "linux"))]
    fn pids(self) -> u64 {
        if self == Self::Java25 {
            mpk_vc::java_release::PIDS
        } else {
            CGROUP_TASK_LIMIT
        }
    }

    #[cfg(any(test, target_os = "linux"))]
    fn tmpfs(self) -> u64 {
        if self == Self::Java25 {
            mpk_vc::java_release::TMPFS_BYTES
        } else {
            WRITABLE_ALLOCATED_BYTES_LIMIT
        }
    }

    #[cfg(target_os = "linux")]
    fn probe_id(self) -> &'static str {
        if self == Self::Java25 {
            "mpk.release.probe.java25.v0"
        } else {
            RESOURCE_PROBE_PROFILE_ID
        }
    }
}

pub(crate) struct PreparedSandbox {
    profile: SandboxProfile,
    #[cfg(target_os = "linux")]
    resource_session: Option<CgroupLeaf>,
}

#[cfg(any(test, target_os = "linux"))]
const FRONTEND_WALL_CLOCK_TIMEOUT: Duration = Duration::from_secs(300);
#[cfg(target_os = "linux")]
const RESOURCE_PROBE_MARKER: &str = ".mpk-release-probe-cgroup2-tmpfs-v0";
#[cfg(target_os = "linux")]
const RESOURCE_PROBE_PROFILE_ID: &str = "mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0";
#[cfg(target_os = "linux")]
const RESOURCE_PROBE_OUTPUT: &[u8] = b"mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0 ok\n";
#[cfg(target_os = "linux")]
const RESOURCE_BOOTSTRAP_MARKER: &str = "__mpk_cgroup2_tmpfs_v0";
#[cfg(target_os = "linux")]
const CGROUP2_MOUNT: &str = "/sys/fs/cgroup";
#[cfg(target_os = "linux")]
const CGROUP_LEAF_PREFIX: &str = "mpk-rust-frontend-";
#[cfg(target_os = "linux")]
const CGROUP_MANAGER_PREFIX: &str = "mpk-rust-launcher-";
#[cfg(any(test, target_os = "linux"))]
const CGROUP_TASK_LIMIT: u64 = 256;
#[cfg(any(test, target_os = "linux"))]
const CGROUP_MEMORY_LIMIT: u64 = 34_359_738_368;
#[cfg(any(test, target_os = "linux"))]
const CGROUP_SWAP_LIMIT: u64 = 0;
#[cfg(any(test, target_os = "linux"))]
const RESOURCE_OPEN_FILE_LIMIT: u64 = 1_024;
#[cfg(any(test, target_os = "linux"))]
const RESOURCE_ADDRESS_SPACE_LIMIT: u64 = 17_179_869_184;
#[cfg(any(test, target_os = "linux"))]
const CSHARP_RESOURCE_ADDRESS_SPACE_LIMIT: u64 = 1_099_511_627_776;
#[cfg(any(test, target_os = "linux"))]
const WRITABLE_ALLOCATED_BYTES_LIMIT: u64 = 21_474_836_480;
#[cfg(any(test, target_os = "linux"))]
const WRITABLE_INODE_LIMIT: u64 = 262_144;
#[cfg(target_os = "linux")]
const CGROUP2_SUPER_MAGIC: u64 = 0x6367_7270;
#[cfg(any(test, target_os = "linux"))]
const INITIAL_CGROUP_NAMESPACE_INODE: u64 = 0xEFFF_FFFB;
#[cfg(target_os = "linux")]
static CGROUP_LEAF_SEQUENCE: AtomicU64 = AtomicU64::new(0);
#[cfg(target_os = "linux")]
static RESOURCE_SESSION_STATE: AtomicU8 = AtomicU8::new(0);

#[cfg(target_os = "linux")]
struct PrivateTempDir {
    temporary: Option<tempfile::TempDir>,
}

#[cfg(target_os = "linux")]
impl PrivateTempDir {
    fn create(prefix: &str) -> Result<Self, SandboxError> {
        let temporary = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in("/tmp")
            .map_err(|_| SandboxError::Unavailable)?;
        Ok(Self {
            temporary: Some(temporary),
        })
    }

    fn path(&self) -> &Path {
        self.temporary
            .as_ref()
            .expect("private temporary directory remains owned")
            .path()
    }

    fn remove(mut self) {
        let temporary = self
            .temporary
            .take()
            .expect("private temporary directory remains owned");
        remove_private_backing_or_abort(temporary);
    }
}

#[cfg(target_os = "linux")]
impl Drop for PrivateTempDir {
    fn drop(&mut self) {
        if let Some(temporary) = self.temporary.take() {
            remove_private_backing_or_abort(temporary);
        }
    }
}

#[cfg(target_os = "linux")]
fn remove_private_backing_or_abort(temporary: tempfile::TempDir) {
    // `TempDir::close` relinquishes ownership even when deletion fails. Keep
    // the path instead so every retry retains an exact deletion target.
    let path = temporary.keep();
    for _ in 0..3 {
        if matches!(
            fs::symlink_metadata(&path),
            Err(error) if error.kind() == io::ErrorKind::NotFound
        ) {
            return;
        }
        let _ = unseal_private_tree(&path);
        let _ = fs::remove_dir_all(&path);
        if matches!(
            fs::symlink_metadata(&path),
            Err(error) if error.kind() == io::ErrorKind::NotFound
        ) {
            return;
        }
    }
    // Continuing could leave captured source or toolchain bytes on the host,
    // and would make the subsequent cgroup discharge claim unsound.
    std::process::abort();
}

/// Runs the fixed capability probe before user source is exposed to the
/// release frontend. The private token makes the launch path consume exactly
/// one successful preparation.
pub(crate) fn prepare_release_sandbox(
    execution_host_profile_id: &str,
) -> Result<PreparedSandbox, SandboxError> {
    let profile = sandbox_profile(execution_host_profile_id)?;
    #[cfg(not(target_os = "linux"))]
    {
        let _ = profile;
        Err(SandboxError::Unavailable)
    }
    #[cfg(target_os = "linux")]
    {
        if profile == SandboxProfile::Java25 {
            require_native_java_host()?;
        }
        let resource_session = match profile {
            SandboxProfile::LegacyNamespaces => {
                run_sandbox_probe(profile, None)?;
                None
            }
            SandboxProfile::Cgroup2Tmpfs | SandboxProfile::Java25 => {
                if RESOURCE_SESSION_STATE
                    .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    return Err(SandboxError::Unavailable);
                }
                let mut session = match CgroupLeaf::create(profile) {
                    Ok(session) => session,
                    Err(error) => {
                        RESOURCE_SESSION_STATE.store(2, Ordering::Release);
                        return Err(error);
                    }
                };
                run_sandbox_probe(profile, Some(&mut session))?;
                Some(session)
            }
        };
        Ok(PreparedSandbox {
            profile,
            resource_session,
        })
    }
}

#[cfg(target_os = "linux")]
fn require_native_java_host() -> Result<(), SandboxError> {
    // Build-target or emulated uname alone is not native-host evidence. The
    // initial proc view must describe x86-64 CPUs before any namespace setup.
    let cpuinfo = read_bounded_file(Path::new("/proc/cpuinfo"), 16 * 1024 * 1024)?;
    let cpuinfo = std::str::from_utf8(&cpuinfo).map_err(|_| SandboxError::Unavailable)?;
    if !cfg!(target_arch = "x86_64")
        || !cpuinfo.lines().any(|line| {
            line.split_once(':').is_some_and(|(key, value)| {
                key.trim() == "flags"
                    && value.split_ascii_whitespace().any(|flag| flag == "lm")
                    && value.split_ascii_whitespace().any(|flag| flag == "sse2")
            })
        })
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(())
}

fn sandbox_profile(execution_host_profile_id: &str) -> Result<SandboxProfile, SandboxError> {
    match execution_host_profile_id {
        LEGACY_EXECUTION_HOST_PROFILE_ID => Ok(SandboxProfile::LegacyNamespaces),
        RUST_EXECUTION_HOST_PROFILE_ID => Ok(SandboxProfile::Cgroup2Tmpfs),
        mpk_vc::java_release::HOST_ID => Ok(SandboxProfile::Java25),
        _ => Err(SandboxError::Unavailable),
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
struct AncestorEventSnapshot {
    directory: PathBuf,
    pids_max: u64,
    memory_high: u64,
    memory_max: u64,
    memory_oom: u64,
    memory_oom_kill: u64,
}

#[cfg(target_os = "linux")]
struct CgroupLeaf {
    profile: SandboxProfile,
    domain: PathBuf,
    manager: PathBuf,
    path: PathBuf,
    ancestor_events: Vec<AncestorEventSnapshot>,
    manager_exists: bool,
    process_in_manager: bool,
    controllers_enabled: bool,
    resource_exists: bool,
    finished: bool,
}

#[cfg(all(test, not(target_os = "linux")))]
type CgroupLeaf = ();

#[cfg(target_os = "linux")]
impl CgroupLeaf {
    fn create(profile: SandboxProfile) -> Result<Self, SandboxError> {
        validate_minimum_kernel_version([6, 4, 0]).map_err(|_| SandboxError::Unavailable)?;
        let (domain, ancestor_events) = delegated_cgroup_domain()?;
        let raw_pid = rustix::process::getpid().as_raw_pid();
        let mut paths = None;
        for _ in 0..128 {
            let sequence = CGROUP_LEAF_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let manager = domain.join(format!("{CGROUP_MANAGER_PREFIX}{raw_pid}-{sequence}"));
            let resource = domain.join(format!("{CGROUP_LEAF_PREFIX}{raw_pid}-{sequence}"));
            match fs::create_dir(&manager) {
                Ok(()) => {
                    paths = Some((manager, resource));
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(SandboxError::Unavailable),
            }
        }
        let (manager, path) = paths.ok_or(SandboxError::Unavailable)?;
        let mut leaf = Self {
            profile,
            domain,
            manager,
            path,
            ancestor_events,
            manager_exists: true,
            process_in_manager: false,
            controllers_enabled: false,
            resource_exists: false,
            finished: false,
        };
        if leaf.configure_domain_and_leaf().is_err() {
            if leaf.rollback_domain().is_err() {
                // Continuing in a partially modified delegated domain could make a later
                // build appear to have the frozen resource boundary when it does not.
                std::process::abort();
            }
            return Err(SandboxError::Unavailable);
        }
        Ok(leaf)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn configure_domain_and_leaf(&mut self) -> Result<(), SandboxError> {
        validate_empty_cgroup(&self.manager)?;
        fs::write(self.manager.join("cgroup.procs"), b"0\n")
            .map_err(|_| SandboxError::Unavailable)?;
        self.process_in_manager = true;
        if current_cgroup_directory()? != self.manager
            || !control_numbers(&self.domain, "cgroup.procs")?.is_empty()
            || control_numbers(&self.manager, "cgroup.procs")?
                != BTreeSet::from([rustix::process::getpid().as_raw_pid() as u32])
        {
            return Err(SandboxError::Unavailable);
        }
        fs::write(
            self.domain.join("cgroup.subtree_control"),
            b"+memory +pids\n",
        )
        .map_err(|_| SandboxError::Unavailable)?;
        self.controllers_enabled = true;
        if control_words(&self.domain, "cgroup.subtree_control")?
            != BTreeSet::from(["memory".to_owned(), "pids".to_owned()])
            || read_control(&self.manager, "pids.max")? != "max"
            || read_control(&self.manager, "memory.max")? != "max"
            || read_control(&self.manager, "memory.high")? != "max"
            || read_control(&self.manager, "memory.swap.max")? != "max"
        {
            return Err(SandboxError::Unavailable);
        }
        self.create_and_configure_resource_leaf()?;
        Ok(())
    }

    fn create_and_configure_resource_leaf(&mut self) -> Result<(), SandboxError> {
        fs::create_dir(&self.path).map_err(|_| SandboxError::Unavailable)?;
        self.resource_exists = true;
        let metadata = fs::symlink_metadata(&self.path).map_err(|_| SandboxError::Unavailable)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(SandboxError::Unavailable);
        }
        if read_control(&self.path, "cgroup.type")? != "domain"
            || !control_words(&self.path, "cgroup.controllers")?.contains("memory")
            || !control_words(&self.path, "cgroup.controllers")?.contains("pids")
        {
            return Err(SandboxError::Unavailable);
        }
        write_control(&self.path, "pids.max", self.profile.pids())?;
        write_control(&self.path, "memory.max", self.profile.memory())?;
        write_control(&self.path, "memory.swap.max", CGROUP_SWAP_LIMIT)?;
        if read_control(&self.path, "pids.max")? != self.profile.pids().to_string()
            || read_control(&self.path, "memory.max")? != self.profile.memory().to_string()
            || read_control(&self.path, "memory.high")? != "max"
            || read_control(&self.path, "memory.swap.max")? != CGROUP_SWAP_LIMIT.to_string()
            || read_control(&self.path, "pids.current")? != "0"
            || read_control(&self.path, "memory.current")? != "0"
            || !control_words(&self.path, "cgroup.subtree_control")?.is_empty()
            || has_child_cgroups(&self.path)?
            || !resource_counters_are_clean(&self.path)?
        {
            return Err(SandboxError::Unavailable);
        }
        Ok(())
    }

    fn prepare_next_resource_leaf(&mut self) -> Result<(), SandboxError> {
        if self.finished
            || self.resource_exists
            || current_process_threads()?.len() != 1
            || !self.session_topology_is_exact()?
            || cgroup_descendant_counts(&self.domain)? != (1, 0)
        {
            return Err(SandboxError::Unavailable);
        }
        let raw_pid = rustix::process::getpid().as_raw_pid();
        let mut path = None;
        for _ in 0..128 {
            let sequence = CGROUP_LEAF_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let candidate = self
                .domain
                .join(format!("{CGROUP_LEAF_PREFIX}{raw_pid}-{sequence}"));
            if !candidate.exists() {
                path = Some(candidate);
                break;
            }
        }
        self.path = path.ok_or(SandboxError::Unavailable)?;
        self.create_and_configure_resource_leaf()
    }

    fn terminate_and_validate(&self) -> Result<(), SandboxError> {
        let kill =
            fs::write(self.path.join("cgroup.kill"), b"1\n").map_err(|_| SandboxError::Unavailable);
        wait_for_cgroup_value(&self.path, "cgroup.events", "populated", 0)?;
        let validation = if read_control(&self.path, "pids.current")? == "0"
            && resource_counters_are_clean(&self.path)?
            && self.topology_controls_are_exact()?
            && self.ancestors_unchanged()?
        {
            Ok(())
        } else {
            Err(SandboxError::Killed)
        };
        kill.and(validation)
    }

    fn finish_resource_after_backing_release(&mut self) -> Result<(), SandboxError> {
        let validation = match resource_counters_are_clean(&self.path) {
            Ok(true) => match (
                self.topology_controls_are_exact(),
                self.ancestors_unchanged(),
            ) {
                (Ok(true), Ok(true)) => Ok(()),
                (Err(error), _) | (_, Err(error)) => Err(error),
                _ => Err(SandboxError::Killed),
            },
            Ok(false) => Err(SandboxError::Killed),
            Err(error) => Err(error),
        };
        let cleanup = self.remove_resource_leaf();
        cleanup.and(validation)
    }

    fn finish_session(&mut self) -> Result<(), SandboxError> {
        let validation = if !self.resource_exists
            && cgroup_descendant_counts(&self.domain)? == (1, 0)
            && self.session_topology_is_exact()?
        {
            Ok(())
        } else {
            Err(SandboxError::Killed)
        };
        let cleanup = self.teardown_session();
        cleanup.and(validation)
    }

    fn kill_best_effort(&self) {
        let _ = fs::write(self.path.join("cgroup.kill"), b"1\n");
    }

    fn require_unpopulated(&self) -> Result<(), SandboxError> {
        let events = read_flat_counters(&self.path.join("cgroup.events"))?;
        if events.get("populated") == Some(&0) {
            Ok(())
        } else {
            Err(SandboxError::Killed)
        }
    }

    fn ancestors_unchanged(&self) -> Result<bool, SandboxError> {
        for expected in &self.ancestor_events {
            if snapshot_ancestor_events(&expected.directory)? != *expected {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn topology_controls_are_exact(&self) -> Result<bool, SandboxError> {
        Ok(self.session_topology_is_exact()?
            && self.resource_exists
            && cgroup_descendant_counts(&self.domain)? == (2, 0)
            && read_control(&self.path, "pids.max")? == self.profile.pids().to_string()
            && read_control(&self.path, "memory.max")? == self.profile.memory().to_string()
            && read_control(&self.path, "memory.high")? == "max"
            && read_control(&self.path, "memory.swap.max")? == CGROUP_SWAP_LIMIT.to_string()
            && control_words(&self.path, "cgroup.subtree_control")?.is_empty()
            && !has_child_cgroups(&self.path)?)
    }

    fn resource_leaf_is_empty_and_clean(&self) -> Result<bool, SandboxError> {
        let events = read_flat_counters(&self.path.join("cgroup.events"))?;
        Ok(self.resource_exists
            && control_numbers(&self.path, "cgroup.procs")?.is_empty()
            && control_numbers(&self.path, "cgroup.threads")?.is_empty()
            && read_control(&self.path, "pids.current")? == "0"
            && resource_memory_is_discharged(&self.path)?
            && events.get("populated") == Some(&0)
            && events.get("frozen") == Some(&0)
            && !has_child_cgroups(&self.path)?
            && resource_counters_are_clean(&self.path)?)
    }

    fn session_topology_is_exact(&self) -> Result<bool, SandboxError> {
        validate_global_cgroup2_mount()?;
        Ok(self.manager_exists
            && self.process_in_manager
            && self.controllers_enabled
            && current_cgroup_directory()? == self.manager
            && control_numbers(&self.domain, "cgroup.procs")?.is_empty()
            && control_words(&self.domain, "cgroup.subtree_control")?
                == BTreeSet::from(["memory".to_owned(), "pids".to_owned()])
            && control_numbers(&self.manager, "cgroup.procs")?
                == BTreeSet::from([rustix::process::getpid().as_raw_pid() as u32])
            && control_numbers(&self.manager, "cgroup.threads")? == current_process_threads()?
            && read_control(&self.manager, "pids.max")? == "max"
            && read_control(&self.manager, "memory.max")? == "max"
            && read_control(&self.manager, "memory.high")? == "max"
            && read_control(&self.manager, "memory.swap.max")? == "max"
            && !has_child_cgroups(&self.manager)?
            && self.ancestors_unchanged()?)
    }

    fn rollback_domain(&mut self) -> Result<(), SandboxError> {
        if self.finished {
            return Ok(());
        }
        self.remove_resource_leaf()?;
        self.teardown_session()
    }

    fn remove_resource_leaf(&mut self) -> Result<(), SandboxError> {
        if self.resource_exists {
            fs::write(self.path.join("cgroup.kill"), b"1\n")
                .map_err(|_| SandboxError::Unavailable)?;
            wait_for_cgroup_value(&self.path, "cgroup.events", "populated", 0)?;
            wait_for_resource_memory_discharge(&self.path)?;
            if !control_numbers(&self.path, "cgroup.procs")?.is_empty()
                || has_child_cgroups(&self.path)?
            {
                return Err(SandboxError::Killed);
            }
            if cgroup_descendant_counts(&self.domain)? != (2, 0) {
                return Err(SandboxError::Killed);
            }
            fs::remove_dir(&self.path).map_err(|_| SandboxError::Unavailable)?;
            self.resource_exists = false;
            wait_for_cgroup_descendants(&self.domain, 1, 0)?;
        }
        Ok(())
    }

    fn teardown_session(&mut self) -> Result<(), SandboxError> {
        if self.finished {
            return Ok(());
        }
        if self.resource_exists || cgroup_descendant_counts(&self.domain)? != (1, 0) {
            return Err(SandboxError::Killed);
        }
        if self.controllers_enabled {
            fs::write(
                self.domain.join("cgroup.subtree_control"),
                b"-memory -pids\n",
            )
            .map_err(|_| SandboxError::Unavailable)?;
            if !control_words(&self.domain, "cgroup.subtree_control")?.is_empty() {
                return Err(SandboxError::Unavailable);
            }
            self.controllers_enabled = false;
        }
        if self.process_in_manager {
            fs::write(self.domain.join("cgroup.procs"), b"0\n")
                .map_err(|_| SandboxError::Unavailable)?;
            if current_cgroup_directory()? != self.domain
                || control_numbers(&self.domain, "cgroup.procs")?
                    != BTreeSet::from([rustix::process::getpid().as_raw_pid() as u32])
                || control_numbers(&self.domain, "cgroup.threads")? != current_process_threads()?
            {
                return Err(SandboxError::Unavailable);
            }
            self.process_in_manager = false;
        }
        if self.manager_exists {
            if !control_numbers(&self.manager, "cgroup.procs")?.is_empty()
                || has_child_cgroups(&self.manager)?
            {
                return Err(SandboxError::Unavailable);
            }
            fs::remove_dir(&self.manager).map_err(|_| SandboxError::Unavailable)?;
            self.manager_exists = false;
        }
        let (_, dying) = wait_for_final_manager_removal(&self.domain)?;
        if control_numbers(&self.domain, "cgroup.procs")?
            != BTreeSet::from([rustix::process::getpid().as_raw_pid() as u32])
            || control_numbers(&self.domain, "cgroup.threads")? != current_process_threads()?
            || !control_words(&self.domain, "cgroup.subtree_control")?.is_empty()
            || has_child_cgroups(&self.domain)?
            || !matches!(dying, 0 | 1)
            || !self.ancestors_unchanged()?
        {
            return Err(SandboxError::Unavailable);
        }
        validate_global_cgroup2_mount()?;
        self.finished = true;
        RESOURCE_SESSION_STATE.store(2, Ordering::Release);
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for CgroupLeaf {
    fn drop(&mut self) {
        let cleanup = if self.finished {
            Ok(())
        } else {
            self.rollback_domain()
        };
        RESOURCE_SESSION_STATE.store(2, Ordering::Release);
        if cleanup.is_err() {
            // An uncertain one-shot cgroup topology cannot be allowed to
            // survive while this process continues through a trusted path.
            std::process::abort();
        }
    }
}

#[cfg(target_os = "linux")]
fn delegated_cgroup_domain() -> Result<(PathBuf, Vec<AncestorEventSnapshot>), SandboxError> {
    validate_global_cgroup2_mount()?;
    let mount = Path::new(CGROUP2_MOUNT);
    let domain = current_cgroup_directory()?;
    if !domain.starts_with(mount)
        || fs::canonicalize(&domain).map_err(|_| SandboxError::Unavailable)? != domain
    {
        return Err(SandboxError::Unavailable);
    }
    let controllers = control_words(&domain, "cgroup.controllers")?;
    let current_threads = current_process_threads()?;
    if !controllers.contains("memory")
        || !controllers.contains("pids")
        || !control_words(&domain, "cgroup.subtree_control")?.is_empty()
        || has_child_cgroups(&domain)?
        || cgroup_descendant_counts(&domain)? != (0, 0)
        || read_control(&domain, "cgroup.type")? != "domain"
        || control_numbers(&domain, "cgroup.procs")?
            != BTreeSet::from([rustix::process::getpid().as_raw_pid() as u32])
        || current_threads.len() != 1
        || control_numbers(&domain, "cgroup.threads")? != current_threads
    {
        return Err(SandboxError::Unavailable);
    }

    // The resource profile cannot be made exact beneath a competing finite
    // ancestor. A sibling could otherwise exhaust an ancestor before this
    // leaf reaches its own registered maximum without incrementing the leaf's
    // local event counter.
    let mut ancestor = domain.as_path();
    let mut ancestor_events = Vec::new();
    loop {
        if ancestor == mount {
            break;
        }
        if read_control(ancestor, "pids.max")? != "max"
            || read_control(ancestor, "memory.max")? != "max"
            || read_control(ancestor, "memory.high")? != "max"
            || read_control(ancestor, "memory.swap.max")? != "max"
        {
            return Err(SandboxError::Unavailable);
        }
        ancestor_events.push(snapshot_ancestor_events(ancestor)?);
        ancestor = ancestor.parent().ok_or(SandboxError::Unavailable)?;
        if !ancestor.starts_with(mount) {
            return Err(SandboxError::Unavailable);
        }
    }
    Ok((domain, ancestor_events))
}

#[cfg(target_os = "linux")]
fn validate_global_cgroup2_mount() -> Result<(), SandboxError> {
    let mount = Path::new(CGROUP2_MOUNT);
    let metadata = fs::symlink_metadata(mount).map_err(|_| SandboxError::Unavailable)?;
    let cgroup_namespace =
        fs::metadata("/proc/self/ns/cgroup").map_err(|_| SandboxError::Unavailable)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || fs::canonicalize(mount).map_err(|_| SandboxError::Unavailable)? != mount
        // Linux reserves this UAPI inode for the initial cgroup namespace.
        // A namespace-relative cgroup2 remount can otherwise claim mount root
        // `/` while hiding a finite host ancestor.
        || !initial_cgroup_namespace_is_exact(cgroup_namespace.ino())
        || rustix::fs::statfs(mount)
            .map_err(|_| SandboxError::Unavailable)?
            .f_type as u64
            != CGROUP2_SUPER_MAGIC
    {
        return Err(SandboxError::Unavailable);
    }
    let mountinfo = read_bounded_file(Path::new("/proc/self/mountinfo"), 16 * 1024 * 1024)?;
    let mountinfo = std::str::from_utf8(&mountinfo).map_err(|_| SandboxError::Unavailable)?;
    if !global_cgroup2_mountinfo_is_exact(mountinfo)
        || control_path_exists(mount, "pids.max")?
        || control_path_exists(mount, "memory.max")?
        || control_path_exists(mount, "memory.high")?
        || control_path_exists(mount, "memory.swap.max")?
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(())
}

#[cfg(any(test, target_os = "linux"))]
fn initial_cgroup_namespace_is_exact(inode: u64) -> bool {
    inode == INITIAL_CGROUP_NAMESPACE_INODE
}

#[cfg(target_os = "linux")]
fn global_cgroup2_mountinfo_is_exact(mountinfo: &str) -> bool {
    let mut cgroup2_mounts = 0usize;
    let mut exact = false;
    for line in mountinfo.lines() {
        let Some((mount_fields, filesystem_fields)) = line.split_once(" - ") else {
            return false;
        };
        let mount_fields = mount_fields.split_ascii_whitespace().collect::<Vec<_>>();
        let filesystem_fields = filesystem_fields
            .split_ascii_whitespace()
            .collect::<Vec<_>>();
        if filesystem_fields.first() != Some(&"cgroup2") {
            continue;
        }
        let Some(next) = cgroup2_mounts.checked_add(1) else {
            return false;
        };
        cgroup2_mounts = next;
        if mount_fields.get(3) == Some(&"/")
            && mount_fields.get(4) == Some(&CGROUP2_MOUNT)
            && mount_fields
                .get(5)
                .is_some_and(|options| options.split(',').any(|option| option == "rw"))
            && filesystem_fields.len() == 3
        {
            exact = true;
        }
    }
    cgroup2_mounts == 1 && exact
}

#[cfg(target_os = "linux")]
fn current_cgroup_directory() -> Result<PathBuf, SandboxError> {
    let memberships = read_bounded_file(Path::new("/proc/self/cgroup"), 4_096)?;
    let memberships = std::str::from_utf8(&memberships).map_err(|_| SandboxError::Unavailable)?;
    let mut unified = memberships
        .lines()
        .filter_map(|line| line.strip_prefix("0::"));
    let relative = unified.next().ok_or(SandboxError::Unavailable)?;
    if unified.next().is_some() {
        return Err(SandboxError::Unavailable);
    }
    let relative = Path::new(relative);
    if !relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(Path::new(CGROUP2_MOUNT).join(
        relative
            .strip_prefix("/")
            .map_err(|_| SandboxError::Unavailable)?,
    ))
}

#[cfg(target_os = "linux")]
fn current_process_threads() -> Result<BTreeSet<u32>, SandboxError> {
    let mut threads = BTreeSet::new();
    for entry in fs::read_dir("/proc/self/task").map_err(|_| SandboxError::Unavailable)? {
        let entry = entry.map_err(|_| SandboxError::Unavailable)?;
        let name = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
            .ok_or(SandboxError::Unavailable)?;
        if !threads.insert(name) {
            return Err(SandboxError::Unavailable);
        }
    }
    if threads.is_empty() {
        return Err(SandboxError::Unavailable);
    }
    Ok(threads)
}

#[cfg(target_os = "linux")]
fn control_numbers(directory: &Path, name: &str) -> Result<BTreeSet<u32>, SandboxError> {
    let value = read_control(directory, name)?;
    let mut numbers = BTreeSet::new();
    for word in value.split_ascii_whitespace() {
        let number = word.parse::<u32>().map_err(|_| SandboxError::Unavailable)?;
        if number == 0 || !numbers.insert(number) {
            return Err(SandboxError::Unavailable);
        }
    }
    Ok(numbers)
}

#[cfg(target_os = "linux")]
fn has_child_cgroups(directory: &Path) -> Result<bool, SandboxError> {
    for entry in fs::read_dir(directory).map_err(|_| SandboxError::Unavailable)? {
        let entry = entry.map_err(|_| SandboxError::Unavailable)?;
        let metadata = entry.file_type().map_err(|_| SandboxError::Unavailable)?;
        if metadata.is_dir() || metadata.is_symlink() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(target_os = "linux")]
fn validate_empty_cgroup(directory: &Path) -> Result<(), SandboxError> {
    let metadata = fs::symlink_metadata(directory).map_err(|_| SandboxError::Unavailable)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || read_control(directory, "cgroup.type")? != "domain"
        || !control_words(directory, "cgroup.subtree_control")?.is_empty()
        || !control_numbers(directory, "cgroup.procs")?.is_empty()
        || !control_numbers(directory, "cgroup.threads")?.is_empty()
        || has_child_cgroups(directory)?
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn snapshot_ancestor_events(directory: &Path) -> Result<AncestorEventSnapshot, SandboxError> {
    if read_control(directory, "pids.max")? != "max"
        || read_control(directory, "memory.max")? != "max"
        || read_control(directory, "memory.high")? != "max"
        || read_control(directory, "memory.swap.max")? != "max"
    {
        return Err(SandboxError::Unavailable);
    }
    let pids = read_flat_counters(&directory.join("pids.events"))?;
    let memory = read_flat_counters(&directory.join("memory.events.local"))?;
    Ok(AncestorEventSnapshot {
        directory: directory.to_owned(),
        pids_max: *pids.get("max").ok_or(SandboxError::Unavailable)?,
        memory_high: *memory.get("high").ok_or(SandboxError::Unavailable)?,
        memory_max: *memory.get("max").ok_or(SandboxError::Unavailable)?,
        memory_oom: *memory.get("oom").ok_or(SandboxError::Unavailable)?,
        memory_oom_kill: *memory.get("oom_kill").ok_or(SandboxError::Unavailable)?,
    })
}

#[cfg(target_os = "linux")]
fn control_path_exists(directory: &Path, name: &str) -> Result<bool, SandboxError> {
    match fs::symlink_metadata(directory.join(name)) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(SandboxError::Unavailable),
    }
}

#[cfg(target_os = "linux")]
fn read_bounded_file(path: &Path, maximum: u64) -> Result<Vec<u8>, SandboxError> {
    let file = File::open(path).map_err(|_| SandboxError::Unavailable)?;
    let mut bytes = Vec::new();
    file.take(maximum.checked_add(1).ok_or(SandboxError::Unavailable)?)
        .read_to_end(&mut bytes)
        .map_err(|_| SandboxError::Unavailable)?;
    if u64::try_from(bytes.len()).map_err(|_| SandboxError::Unavailable)? > maximum {
        return Err(SandboxError::Unavailable);
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn wait_for_cgroup_value(
    directory: &Path,
    control: &str,
    key: &str,
    expected: u64,
) -> Result<(), SandboxError> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let counters = read_flat_counters(&directory.join(control))?;
        if counters.get(key) == Some(&expected) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(SandboxError::Killed);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(target_os = "linux")]
fn wait_for_resource_memory_discharge(directory: &Path) -> Result<(), SandboxError> {
    // Large tmpfs page sets may be released asynchronously after the final
    // mount-namespace descriptor closes. Reclaim file cache charged while the
    // resource task read its executable and immutable inputs. Newer kernels
    // can retain kernel-object charges and a nonzero per-CPU stock in
    // memory.current after every task-owned gauge has reached zero, so those
    // gauges and swap.current are the discharge authority before removal.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        request_resource_memory_reclaim(directory)?;
        if resource_memory_is_discharged(directory)? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(SandboxError::Killed);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(target_os = "linux")]
fn verify_self_in_resource_cgroup(path: &Path, profile: SandboxProfile) -> Result<(), u8> {
    validate_global_cgroup2_mount().map_err(|_| 125)?;
    let mount = Path::new(CGROUP2_MOUNT);
    let name = path.file_name().and_then(|name| name.to_str()).ok_or(125)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| 125)?;
    let parent = path.parent().ok_or(125)?;
    let process = rustix::process::getpid().as_raw_pid() as u32;
    let events = read_flat_counters(&path.join("cgroup.events")).map_err(|_| 125)?;
    if !parent.starts_with(mount)
        || !valid_resource_cgroup_name(name)
        || !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || read_control(path, "pids.max").map_err(|_| 125)? != profile.pids().to_string()
        || read_control(path, "memory.max").map_err(|_| 125)? != profile.memory().to_string()
        || read_control(path, "memory.high").map_err(|_| 125)? != "max"
        || read_control(path, "memory.swap.max").map_err(|_| 125)? != CGROUP_SWAP_LIMIT.to_string()
        || control_words(parent, "cgroup.subtree_control").map_err(|_| 125)?
            != BTreeSet::from(["memory".to_owned(), "pids".to_owned()])
        || !control_numbers(parent, "cgroup.procs")
            .map_err(|_| 125)?
            .is_empty()
        || cgroup_descendant_counts(parent).map_err(|_| 125)? != (2, 0)
        || current_cgroup_directory().map_err(|_| 125)? != path
        || control_numbers(path, "cgroup.procs").map_err(|_| 125)? != BTreeSet::from([process])
        || control_numbers(path, "cgroup.threads").map_err(|_| 125)? != BTreeSet::from([process])
        || read_control(path, "pids.current").map_err(|_| 125)? != "1"
        || events.get("populated") != Some(&1)
        || events.get("frozen") != Some(&0)
        || !control_words(path, "cgroup.subtree_control")
            .map_err(|_| 125)?
            .is_empty()
        || has_child_cgroups(path).map_err(|_| 125)?
        || !resource_counters_are_clean(path).map_err(|_| 125)?
    {
        return Err(125);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn valid_resource_cgroup_name(name: &str) -> bool {
    let Some(suffix) = name.strip_prefix(CGROUP_LEAF_PREFIX) else {
        return false;
    };
    let Some((pid, sequence)) = suffix.split_once('-') else {
        return false;
    };
    let canonical_number = |value: &str, allow_zero: bool| {
        !value.is_empty()
            && value.bytes().all(|byte| byte.is_ascii_digit())
            && (allow_zero || value != "0")
            && (value == "0" || !value.starts_with('0'))
    };
    canonical_number(pid, false) && canonical_number(sequence, true) && !sequence.contains('-')
}

#[cfg(target_os = "linux")]
fn verify_resource_process_controls(address_space_limit: u64) -> Result<(), u8> {
    use rustix::process::{getrlimit, Resource, Rlimit};

    for (resource, expected) in [
        (
            Resource::Core,
            Rlimit {
                current: Some(0),
                maximum: Some(0),
            },
        ),
        (
            Resource::Nofile,
            Rlimit {
                current: Some(RESOURCE_OPEN_FILE_LIMIT),
                maximum: Some(RESOURCE_OPEN_FILE_LIMIT),
            },
        ),
        (
            Resource::As,
            Rlimit {
                current: Some(address_space_limit),
                maximum: Some(address_space_limit),
            },
        ),
        (
            Resource::Fsize,
            Rlimit {
                current: None,
                maximum: None,
            },
        ),
        (
            Resource::Nproc,
            Rlimit {
                current: None,
                maximum: None,
            },
        ),
    ] {
        if getrlimit(resource) != expected {
            return Err(125);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn verify_only_standard_descriptors() -> Result<(), u8> {
    let descriptors = fs::read_dir("/proc/self/fd")
        .map_err(|_| 125)?
        .map(|entry| {
            entry
                .map_err(|_| 125)?
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
                .ok_or(125)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for descriptor in descriptors.into_iter().filter(|descriptor| *descriptor > 2) {
        match fs::symlink_metadata(format!("/proc/self/fd/{descriptor}")) {
            Ok(_) => return Err(125),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(125),
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn read_control(directory: &Path, name: &str) -> Result<String, SandboxError> {
    let value = fs::read_to_string(directory.join(name)).map_err(|_| SandboxError::Unavailable)?;
    if value.len() > 4_096 || value.contains('\0') {
        return Err(SandboxError::Unavailable);
    }
    Ok(value.trim_end_matches('\n').to_owned())
}

#[cfg(target_os = "linux")]
fn control_words(directory: &Path, name: &str) -> Result<BTreeSet<String>, SandboxError> {
    let value = read_control(directory, name)?;
    let words = value.split_ascii_whitespace().map(str::to_owned).collect();
    Ok(words)
}

#[cfg(target_os = "linux")]
fn write_control(directory: &Path, name: &str, value: u64) -> Result<(), SandboxError> {
    fs::write(directory.join(name), format!("{value}\n")).map_err(|_| SandboxError::Unavailable)
}

#[cfg(target_os = "linux")]
fn read_flat_counters(path: &Path) -> Result<BTreeMap<String, u64>, SandboxError> {
    let contents = fs::read_to_string(path).map_err(|_| SandboxError::Unavailable)?;
    parse_flat_counters(&contents).ok_or(SandboxError::Unavailable)
}

#[cfg(target_os = "linux")]
fn cgroup_descendant_counts(directory: &Path) -> Result<(u64, u64), SandboxError> {
    let counters = read_flat_counters(&directory.join("cgroup.stat"))?;
    Ok((
        *counters
            .get("nr_descendants")
            .ok_or(SandboxError::Unavailable)?,
        *counters
            .get("nr_dying_descendants")
            .ok_or(SandboxError::Unavailable)?,
    ))
}

#[cfg(target_os = "linux")]
fn wait_for_cgroup_descendants(
    directory: &Path,
    descendants: u64,
    dying_descendants: u64,
) -> Result<(), SandboxError> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if cgroup_descendant_counts(directory)? == (descendants, dying_descendants) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(SandboxError::Killed);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(target_os = "linux")]
fn wait_for_final_manager_removal(directory: &Path) -> Result<(u64, u64), SandboxError> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let counts = cgroup_descendant_counts(directory)?;
        if counts.0 == 0 && matches!(counts.1, 0 | 1) {
            return Ok(counts);
        }
        if Instant::now() >= deadline {
            return Err(SandboxError::Killed);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(any(test, target_os = "linux"))]
fn parse_flat_counters(contents: &str) -> Option<BTreeMap<String, u64>> {
    if contents.len() > 4_096 || contents.is_empty() || !contents.ends_with('\n') {
        return None;
    }
    let mut counters = BTreeMap::new();
    for line in contents.lines() {
        let mut fields = line.split_ascii_whitespace();
        let key = fields.next()?;
        let value = fields.next()?.parse::<u64>().ok()?;
        if fields.next().is_some()
            || key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            || counters.insert(key.to_owned(), value).is_some()
        {
            return None;
        }
    }
    Some(counters)
}

#[cfg(target_os = "linux")]
fn resource_counters_are_clean(directory: &Path) -> Result<bool, SandboxError> {
    let pids = read_flat_counters(&directory.join("pids.events"))?;
    let memory = read_flat_counters(&directory.join("memory.events.local"))?;
    let peak = read_control(directory, "memory.peak")?
        .parse::<u64>()
        .map_err(|_| SandboxError::Unavailable)?;
    let limit = read_control(directory, "memory.max")?
        .parse::<u64>()
        .map_err(|_| SandboxError::Unavailable)?;
    Ok(resource_counter_values_are_clean(&pids, &memory, peak) && peak <= limit)
}

#[cfg(target_os = "linux")]
fn resource_memory_is_discharged(directory: &Path) -> Result<bool, SandboxError> {
    let _current = read_control(directory, "memory.current")?
        .parse::<u64>()
        .map_err(|_| SandboxError::Unavailable)?;
    let swap_current = read_control(directory, "memory.swap.current")?
        .parse::<u64>()
        .map_err(|_| SandboxError::Unavailable)?;
    let memory = read_flat_counters(&directory.join("memory.stat"))?;
    Ok(swap_current == 0 && resource_memory_values_are_discharged(&memory))
}

#[cfg(target_os = "linux")]
fn request_resource_memory_reclaim(directory: &Path) -> Result<(), SandboxError> {
    let current = read_control(directory, "memory.current")?
        .parse::<u64>()
        .map_err(|_| SandboxError::Unavailable)?;
    if current == 0 {
        return Ok(());
    }
    match fs::write(directory.join("memory.reclaim"), format!("{current}\n")) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(()),
        Err(_) => Err(SandboxError::Unavailable),
    }
}

#[cfg(any(test, target_os = "linux"))]
fn resource_memory_values_are_discharged(memory: &BTreeMap<String, u64>) -> bool {
    ["anon", "file", "sock", "shmem"]
        .into_iter()
        .all(|name| memory.get(name) == Some(&0))
        && ["zswap", "zswapped"]
            .into_iter()
            .all(|name| memory.get(name).is_none_or(|value| *value == 0))
}

#[cfg(any(test, target_os = "linux"))]
fn resource_counter_values_are_clean(
    pids: &BTreeMap<String, u64>,
    memory: &BTreeMap<String, u64>,
    peak: u64,
) -> bool {
    pids.get("max") == Some(&0)
        && memory.get("high") == Some(&0)
        && memory.get("max") == Some(&0)
        && memory.get("oom") == Some(&0)
        && memory.get("oom_kill") == Some(&0)
        && peak <= CGROUP_MEMORY_LIMIT
}

/// The release-facing sandbox stays fail-closed until entered through the
/// Linux namespace bootstrap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_release_frontend(
    prepared: PreparedSandbox,
    frontend: &BundleSnapshot,
    toolchain: &BundleSnapshot,
    executable: &str,
    args: &[String],
    environment: &BTreeMap<String, String>,
    captured_inputs: &[CapturedInput<'_>],
    staged_directories: &[&str],
    staged_placeholders: &[&str],
) -> Result<SandboxOutput, SandboxError> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (
            prepared,
            frontend,
            toolchain,
            executable,
            args,
            environment,
            captured_inputs,
            staged_directories,
            staged_placeholders,
        );
        Err(SandboxError::Unavailable)
    }
    #[cfg(target_os = "linux")]
    {
        let PreparedSandbox {
            profile,
            mut resource_session,
        } = prepared;
        if !matches!(
            executable,
            "bin/go2vir" | "bin/rust2vir" | CSHARP_BOOTSTRAP_EXECUTABLE | JAVA_BOOTSTRAP_EXECUTABLE
        ) {
            return Err(SandboxError::Unavailable);
        }
        let resource_frontend = matches!(
            executable,
            "bin/rust2vir" | CSHARP_BOOTSTRAP_EXECUTABLE | JAVA_BOOTSTRAP_EXECUTABLE
        );
        let csharp_frontend = executable == CSHARP_BOOTSTRAP_EXECUTABLE;
        let java_frontend = executable == JAVA_BOOTSTRAP_EXECUTABLE;
        if profile.resource() != resource_frontend
            || profile.resource() != resource_session.is_some()
            || (profile == SandboxProfile::Java25) != java_frontend
        {
            return Err(SandboxError::Unavailable);
        }
        // Create and configure the execution leaf before materializing any
        // captured source bytes. The bootstrap joins this leaf before it
        // performs namespace setup or launches any untrusted executable.
        if let Some(session) = resource_session.as_mut() {
            session.prepare_next_resource_leaf()?;
        }
        let temporary = PrivateTempDir::create("mpk-frontend-sandbox-")?;
        let root = temporary.path();
        materialize_snapshot(frontend, &root.join("mpk/frontend"))?;
        materialize_snapshot(toolchain, &root.join("mpk/toolchain"))?;
        materialize_sources(
            captured_inputs,
            staged_directories,
            staged_placeholders,
            &root.join("mpk/source"),
        )?;
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
        if csharp_frontend {
            for relative in [
                "mpk/empty-home",
                "mpk/empty-nuget",
                "mpk/empty-nuget-http",
                "mpk/empty-nuget-plugins",
            ] {
                fs::create_dir_all(root.join(relative)).map_err(|_| SandboxError::Unavailable)?;
            }
        }
        if java_frontend {
            fs::create_dir_all(root.join("mpk/empty-home"))
                .map_err(|_| SandboxError::Unavailable)?;
        }
        fs::write(root.join("dev/null"), b"").map_err(|_| SandboxError::Unavailable)?;
        if csharp_frontend || java_frontend {
            fs::write(root.join("dev/urandom"), b"").map_err(|_| SandboxError::Unavailable)?;
        }
        if resource_frontend {
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
        if csharp_frontend {
            for relative in [
                "mpk/empty-home",
                "mpk/empty-nuget",
                "mpk/empty-nuget-http",
                "mpk/empty-nuget-plugins",
            ] {
                seal_read_only_tree(&root.join(relative))?;
            }
        }
        if java_frontend {
            seal_read_only_tree(&root.join("mpk/empty-home"))?;
        }
        let current_executable = match profile {
            SandboxProfile::LegacyNamespaces => {
                std::env::current_exe().map_err(|_| SandboxError::Unavailable)?
            }
            SandboxProfile::Cgroup2Tmpfs | SandboxProfile::Java25 => {
                PathBuf::from("/proc/self/exe")
            }
        };
        let mut bootstrap_args = Vec::with_capacity(args.len() + 4);
        bootstrap_args.push("__mpk_frontend_sandbox_v0".to_owned());
        if let Some(cgroup) = resource_session.as_ref() {
            bootstrap_args.push(RESOURCE_BOOTSTRAP_MARKER.to_owned());
            bootstrap_args.push(
                cgroup
                    .path()
                    .to_str()
                    .ok_or(SandboxError::Unavailable)?
                    .to_owned(),
            );
        }
        bootstrap_args.push(executable.to_owned());
        bootstrap_args.extend_from_slice(args);
        let result = match run_closed_process(
            &current_executable,
            root,
            &bootstrap_args,
            environment,
            resource_session.as_ref(),
        ) {
            Ok(output) if output.exit_code == Some(125) => Err(SandboxError::Unavailable),
            Ok(output) if output.exit_code == Some(126) => Err(SandboxError::Spawn),
            output => output,
        };
        temporary.remove();
        let resource_cleanup = match resource_session.as_mut() {
            Some(session) => session.finish_resource_after_backing_release(),
            None => Ok(()),
        };
        let session_cleanup = match resource_session.as_mut() {
            Some(session) => session.finish_session(),
            None => Ok(()),
        };
        session_cleanup.and(resource_cleanup).and(result)
    }
}

pub(crate) fn launch_java_frontend(
    prepared: PreparedSandbox,
    frontend: &BundleSnapshot,
    toolchain: &BundleSnapshot,
    plan: &mpk_vc::java_release::JavaLauncherPlan,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<SandboxOutput, SandboxError> {
    if prepared.profile != SandboxProfile::Java25 {
        return Err(SandboxError::Unavailable);
    }
    launch_release_frontend(
        prepared,
        frontend,
        toolchain,
        JAVA_BOOTSTRAP_EXECUTABLE,
        &plan.argv()[1..],
        plan.environment(),
        captured_inputs,
        &[],
        &[],
    )
}

/// Runs the registered JVM and JAR through Java's exact sandbox solely to
/// measure native JVM thread/syscall behavior. Complete source processing is
/// exercised separately by the native cases; keeping trace transport out of
/// their frozen 120-second request budget avoids measuring ptrace overhead as
/// frontend work. This helper has no production command route.
pub(crate) fn launch_java_trace_probe(
    prepared: PreparedSandbox,
    frontend: &BundleSnapshot,
    toolchain: &BundleSnapshot,
) -> Result<SandboxOutput, SandboxError> {
    if prepared.profile != SandboxProfile::Java25 {
        return Err(SandboxError::Unavailable);
    }
    let mut arguments = mpk_vc::java_release::ARGV_PREFIX[1..]
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    arguments.push("--version".to_owned());
    launch_release_frontend(
        prepared,
        frontend,
        toolchain,
        JAVA_BOOTSTRAP_EXECUTABLE,
        &arguments,
        &mpk_vc::java_release::environment(),
        &[],
        &[],
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_csharp_frontend(
    prepared: PreparedSandbox,
    frontend: &BundleSnapshot,
    toolchain: &BundleSnapshot,
    args: &[String],
    environment: &BTreeMap<String, String>,
    captured_inputs: &[CapturedInput<'_>],
    staged_directories: &[&str],
    staged_placeholders: &[&str],
) -> Result<SandboxOutput, SandboxError> {
    launch_release_frontend(
        prepared,
        frontend,
        toolchain,
        CSHARP_BOOTSTRAP_EXECUTABLE,
        args,
        environment,
        captured_inputs,
        staged_directories,
        staged_placeholders,
    )
}

#[cfg(target_os = "linux")]
fn run_sandbox_probe(
    profile: SandboxProfile,
    mut resource_session: Option<&mut CgroupLeaf>,
) -> Result<(), SandboxError> {
    let temporary = PrivateTempDir::create(match profile {
        SandboxProfile::LegacyNamespaces => "mpk-frontend-probe-",
        SandboxProfile::Cgroup2Tmpfs | SandboxProfile::Java25 => "mpk-frontend-resource-probe-",
    })?;
    if profile.resource() {
        let cgroup_path = resource_session
            .as_ref()
            .ok_or(SandboxError::Unavailable)?
            .path()
            .to_str()
            .ok_or(SandboxError::Unavailable)?;
        fs::write(
            temporary.path().join(RESOURCE_PROBE_MARKER),
            format!("{}\n{cgroup_path}\n", profile.probe_id()),
        )
        .map_err(|_| SandboxError::Unavailable)?;
    }
    if let Some(cgroup) = resource_session.as_deref_mut() {
        let arguments = vec!["__mpk_frontend_probe_v0".to_owned()];
        let environment = BTreeMap::new();
        let output = run_resource_process_with_limits(
            temporary.path(),
            &arguments,
            &environment,
            Duration::from_secs(2),
            512,
            512,
            cgroup,
            false,
        );
        let result = match output {
            Ok(output)
                if output.exit_code == Some(0)
                    && !output.signaled
                    && output.stdout == format!("{} ok\n", profile.probe_id()).as_bytes()
                    && output.stderr_observed_bytes == 0
                    && !output.stream_limit_exceeded
                    && output.stdout.len() <= 512 =>
            {
                Ok(())
            }
            _ => Err(SandboxError::Unavailable),
        };
        temporary.remove();
        let cleanup = cgroup.finish_resource_after_backing_release();
        return cleanup.and(result);
    }
    let executable = std::env::current_exe().map_err(|_| SandboxError::Unavailable)?;
    let mut command = Command::new(executable);
    command
        .arg("__mpk_frontend_probe_v0")
        .env_clear()
        .current_dir(temporary.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        if error.raw_os_error() == Some(125) {
            SandboxError::Unavailable
        } else {
            SandboxError::Spawn
        }
    })?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut wait_failed = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            Ok(None) | Err(_) => {
                wait_failed = true;
                if let Some(cgroup) = resource_session.as_deref() {
                    cgroup.kill_best_effort();
                }
                let _ = child.kill();
                break child.wait().ok();
            }
        }
    };
    let reap_failed = status.is_none();
    let output = if reap_failed {
        None
    } else {
        child.wait_with_output().ok()
    };
    let expected_stdout = match profile {
        SandboxProfile::LegacyNamespaces => b"mpk.release.probe.v0 ok\n".as_slice(),
        SandboxProfile::Cgroup2Tmpfs => RESOURCE_PROBE_OUTPUT,
        SandboxProfile::Java25 => return Err(SandboxError::Unavailable),
    };
    let result = match (status, output) {
        (Some(status), Some(output))
            if !wait_failed
                && status.success()
                && output.stdout == expected_stdout
                && output.stderr.is_empty()
                && output.stdout.len() + output.stderr.len() <= 512 =>
        {
            Ok(())
        }
        _ => Err(SandboxError::Unavailable),
    };
    drop(command);
    let cgroup_validation = match resource_session.as_deref() {
        Some(cgroup) => cgroup.terminate_and_validate(),
        None => Ok(()),
    };
    temporary.remove();
    let cgroup_cleanup = match resource_session {
        Some(cgroup) => cgroup.finish_resource_after_backing_release(),
        None => Ok(()),
    };
    let final_result = cgroup_cleanup.and(cgroup_validation).and(result);
    if reap_failed {
        std::process::abort();
    }
    final_result
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
    run_closed_process(&executable_path, temporary.path(), args, environment, None)
}

#[cfg(any(test, target_os = "linux"))]
fn run_closed_process(
    executable: &Path,
    current_directory: &Path,
    args: &[String],
    environment: &BTreeMap<String, String>,
    resource_cgroup: Option<&CgroupLeaf>,
) -> Result<SandboxOutput, SandboxError> {
    run_closed_process_with_timeout(
        executable,
        current_directory,
        args,
        environment,
        FRONTEND_WALL_CLOCK_TIMEOUT,
        resource_cgroup,
    )
}

#[cfg(any(test, target_os = "linux"))]
fn run_closed_process_with_timeout(
    executable: &Path,
    current_directory: &Path,
    args: &[String],
    environment: &BTreeMap<String, String>,
    timeout: Duration,
    resource_cgroup: Option<&CgroupLeaf>,
) -> Result<SandboxOutput, SandboxError> {
    #[cfg(target_os = "linux")]
    if let Some(cgroup) = resource_cgroup {
        let java = cgroup.profile == SandboxProfile::Java25;
        return run_resource_process_with_limits(
            current_directory,
            args,
            environment,
            if java {
                Duration::from_secs(mpk_vc::java_release::TIMEOUT_SECONDS).min(timeout)
            } else {
                timeout
            },
            if java {
                268_435_456
            } else {
                FRONTEND_STDOUT_BYTES_MAX
            },
            if java {
                2_097_152
            } else {
                FRONTEND_STDERR_BYTES_MAX
            },
            cgroup,
            true,
        );
    }
    #[cfg(not(target_os = "linux"))]
    let _ = &resource_cgroup;
    #[cfg(target_os = "linux")]
    let requires_output_copy_ack = resource_cgroup.is_some();
    #[cfg(not(target_os = "linux"))]
    let requires_output_copy_ack = false;
    let mut command = Command::new(executable);
    command
        .args(args)
        .env_clear()
        .envs(environment)
        .current_dir(current_directory)
        .stdin(if requires_output_copy_ack {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "linux")]
    {
        command.process_group(0);
    }
    let mut child = command.spawn().map_err(|error| {
        if resource_cgroup.is_some() && error.raw_os_error() == Some(125) {
            SandboxError::Unavailable
        } else {
            SandboxError::Spawn
        }
    })?;
    #[cfg(target_os = "linux")]
    let mut namespace_ack = if requires_output_copy_ack {
        match child.stdin.take() {
            Some(stdin) => Some(stdin),
            None => {
                #[cfg(target_os = "linux")]
                let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, None);
                #[cfg(not(target_os = "linux"))]
                let cleanup = child
                    .kill()
                    .and_then(|()| child.wait().map(|_| ()))
                    .map_err(|_| SandboxError::Killed);
                return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
            }
        }
    } else {
        None
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            #[cfg(target_os = "linux")]
            let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, None);
            #[cfg(not(target_os = "linux"))]
            let cleanup = child
                .kill()
                .and_then(|()| child.wait().map(|_| ()))
                .map_err(|_| SandboxError::Killed);
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            drop(stdout);
            #[cfg(target_os = "linux")]
            let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, None);
            #[cfg(not(target_os = "linux"))]
            let cleanup = child
                .kill()
                .and_then(|()| child.wait().map(|_| ()))
                .map_err(|_| SandboxError::Killed);
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        }
    };
    #[cfg(target_os = "linux")]
    let resource_mount_namespace = if requires_output_copy_ack {
        let namespace = match capture_child_mount_namespace(&mut child) {
            Ok(namespace) => namespace,
            Err(error) => {
                drop(stdout);
                drop(stderr);
                let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, None);
                return Err(cleanup_dominates(error, cleanup));
            }
        };
        let Some(mut ack) = namespace_ack.take() else {
            drop(stdout);
            drop(stderr);
            let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, Some(namespace));
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        };
        if ack.write_all(b"\0").is_err() {
            drop(stdout);
            drop(stderr);
            let cleanup = cleanup_spawned_process(&mut child, resource_cgroup, Some(namespace));
            return Err(cleanup_dominates(SandboxError::Unavailable, cleanup));
        }
        Some(namespace)
    } else {
        None
    };
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
    let deadline = Instant::now().checked_add(timeout);
    let mut kill_cause = None;
    let mut wait_failed = false;
    let status = loop {
        let observed_cause = first_execution_kill_cause(
            kill_cause,
            overflow.load(Ordering::Acquire),
            deadline.is_none_or(|deadline| Instant::now() >= deadline),
        );
        if observed_cause != kill_cause {
            kill_cause = observed_cause;
            kill_execution(&mut child, resource_cgroup);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(2)),
            Err(_) => {
                wait_failed = true;
                kill_execution(&mut child, resource_cgroup);
                match child.wait() {
                    Ok(status) => break status,
                    Err(_) => {
                        #[cfg(target_os = "linux")]
                        if let Some(cgroup) = resource_cgroup.as_ref() {
                            cgroup.kill_best_effort();
                        }
                        // Without a successful reap there is no bounded proof that
                        // every inherited output descriptor has closed.
                        std::process::abort();
                    }
                }
            }
        }
    };
    #[cfg(target_os = "linux")]
    if let Some(cgroup) = resource_cgroup.as_ref() {
        cgroup.kill_best_effort();
    }
    let cgroup_validation = {
        #[cfg(target_os = "linux")]
        {
            match resource_cgroup.as_ref() {
                Some(cgroup) => cgroup.terminate_and_validate(),
                None => Ok(()),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    };
    #[cfg(target_os = "linux")]
    if resource_cgroup
        .as_ref()
        .is_some_and(|cgroup| cgroup.require_unpopulated().is_err())
    {
        // Pipe readers cannot be joined safely while an unaccounted writer may
        // still exist. Reusing this process would also leave the cgroup state
        // uncertain, so fail-stop instead of publishing partial output.
        std::process::abort();
    }
    let stdout_result = stdout_reader.join().map_err(|_| SandboxError::Killed);
    let stderr_result = stderr_reader.join().map_err(|_| SandboxError::Killed);
    // The namespace descriptor pins the aggregate tmpfs until every accepted
    // output byte has been copied out of the child pipes.
    #[cfg(target_os = "linux")]
    drop(resource_mount_namespace);
    cgroup_validation?;
    let (stdout, _) = stdout_result?;
    let (_, stderr_observed_bytes) = stderr_result?;
    if wait_failed || read_failed.load(Ordering::Acquire) {
        return Err(SandboxError::Killed);
    }
    if kill_cause == Some(ExecutionKillCause::Timeout) {
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
#[allow(clippy::too_many_arguments)]
fn run_resource_process_with_limits(
    current_directory: &Path,
    args: &[String],
    environment: &BTreeMap<String, String>,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
    cgroup: &CgroupLeaf,
    requires_output_copy_ack: bool,
) -> Result<SandboxOutput, SandboxError> {
    let launch = prepare_resource_launch(current_directory, args, environment, cgroup)?;
    let mut child = match launch.spawn() {
        Ok(child) => child,
        Err(mpk_linux_sandbox::LaunchError::CloneOrSetup(_)) => {
            return Err(SandboxError::Unavailable);
        }
        Err(mpk_linux_sandbox::LaunchError::Exec(_)) => return Err(SandboxError::Spawn),
    };
    let mut stdin = match child.take_stdin() {
        Some(stdin) => stdin,
        None => {
            let cleanup = cleanup_resource_process(&mut child, cgroup, None);
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        }
    };
    let stdout = match child.take_stdout() {
        Some(stdout) => stdout,
        None => {
            drop(stdin);
            let cleanup = cleanup_resource_process(&mut child, cgroup, None);
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        }
    };
    let stderr = match child.take_stderr() {
        Some(stderr) => stderr,
        None => {
            drop(stdin);
            drop(stdout);
            let cleanup = cleanup_resource_process(&mut child, cgroup, None);
            return Err(cleanup_dominates(SandboxError::Spawn, cleanup));
        }
    };

    let resource_mount_namespace = if requires_output_copy_ack {
        let namespace = match capture_resource_child_mount_namespace(&mut child) {
            Ok(namespace) => namespace,
            Err(error) => {
                drop(stdin);
                drop(stdout);
                drop(stderr);
                let cleanup = cleanup_resource_process(&mut child, cgroup, None);
                return Err(cleanup_dominates(error, cleanup));
            }
        };
        if stdin.write_all(b"\0").is_err() {
            drop(stdin);
            drop(stdout);
            drop(stderr);
            let cleanup = cleanup_resource_process(&mut child, cgroup, Some(namespace));
            return Err(cleanup_dominates(SandboxError::Unavailable, cleanup));
        }
        Some(namespace)
    } else {
        None
    };
    drop(stdin);

    let overflow = Arc::new(AtomicBool::new(false));
    let read_failed = Arc::new(AtomicBool::new(false));
    let stdout_reader = bounded_reader(
        stdout,
        stdout_limit,
        true,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let stderr_reader = bounded_reader(
        stderr,
        stderr_limit,
        false,
        Arc::clone(&overflow),
        Arc::clone(&read_failed),
    );
    let deadline = Instant::now().checked_add(timeout);
    let mut kill_cause = None;
    let mut cleanup_signal_failed = false;
    let mut kill_deadline = None;
    let status = loop {
        let observed_cause = first_execution_kill_cause(
            kill_cause,
            overflow.load(Ordering::Acquire),
            deadline.is_none_or(|deadline| Instant::now() >= deadline),
        );
        if observed_cause != kill_cause {
            kill_cause = observed_cause;
            cleanup_signal_failed |= request_resource_kill(&child, cgroup);
            kill_deadline = Some(Instant::now() + Duration::from_secs(2));
        }
        if kill_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            std::process::abort();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(2)),
            Err(_) => {
                let _ = request_resource_kill(&child, cgroup);
                // A pidfd wait failure leaves no safe bounded proof that the
                // child or inherited stream writers have been reaped.
                std::process::abort();
            }
        }
    };

    // The leader may have exited while descendants retained output fds. Kill
    // the complete accounting tree before joining either reader.
    cleanup_signal_failed |= fs::write(cgroup.path.join("cgroup.kill"), b"1\n").is_err();
    let cgroup_validation = cgroup.terminate_and_validate();
    if cgroup.require_unpopulated().is_err() {
        std::process::abort();
    }
    let stdout_result = stdout_reader.join().map_err(|_| SandboxError::Killed);
    let stderr_result = stderr_reader.join().map_err(|_| SandboxError::Killed);
    // This descriptor pins the aggregate tmpfs until every accepted output
    // byte has been copied from the now-closed child pipes.
    drop(resource_mount_namespace);
    cgroup_validation?;
    let (stdout, _) = stdout_result?;
    let (_, stderr_observed_bytes) = stderr_result?;
    if cleanup_signal_failed || read_failed.load(Ordering::Acquire) {
        return Err(SandboxError::Killed);
    }
    if kill_cause == Some(ExecutionKillCause::Timeout) {
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
fn prepare_resource_launch(
    current_directory: &Path,
    args: &[String],
    environment: &BTreeMap<String, String>,
    cgroup: &CgroupLeaf,
) -> Result<mpk_linux_sandbox::PreparedLaunch, SandboxError> {
    if current_cgroup_directory()? != cgroup.manager
        || !cgroup.resource_exists
        || cgroup.finished
        || !cgroup.topology_controls_are_exact()?
        || !cgroup.resource_leaf_is_empty_and_clean()?
    {
        return Err(SandboxError::Unavailable);
    }
    // `/proc/self/exe` pins the running MPK inode. Reopening the installation
    // pathname returned by `current_exe` would permit same-UID replacement
    // between validation and the bootstrap's exec.
    let executable = File::open("/proc/self/exe").map_err(|_| SandboxError::Unavailable)?;
    let executable_metadata = executable
        .metadata()
        .map_err(|_| SandboxError::Unavailable)?;
    if !executable_metadata.is_file() || executable_metadata.mode() & 0o111 == 0 {
        return Err(SandboxError::Unavailable);
    }
    let cgroup_directory = File::open(&cgroup.path).map_err(|_| SandboxError::Unavailable)?;
    let current_directory = File::open(current_directory).map_err(|_| SandboxError::Unavailable)?;
    let address_space_limit = if cgroup.profile == SandboxProfile::Cgroup2Tmpfs
        && args
            .get(3)
            .is_some_and(|argument| argument == CSHARP_BOOTSTRAP_EXECUTABLE)
    {
        CSHARP_RESOURCE_ADDRESS_SPACE_LIMIT
    } else {
        RESOURCE_ADDRESS_SPACE_LIMIT
    };
    let capacity = args.len().checked_add(1).ok_or(SandboxError::Unavailable)?;
    let mut arguments = Vec::with_capacity(capacity);
    arguments.push(OsString::from("mpk"));
    arguments.extend(args.iter().map(OsString::from));
    let environment = environment
        .iter()
        .map(|(key, value)| (OsString::from(key), OsString::from(value)))
        .collect();
    mpk_linux_sandbox::PreparedLaunch::new(
        mpk_linux_sandbox::LaunchFiles {
            executable,
            cgroup: cgroup_directory,
            current_directory,
        },
        arguments,
        environment,
        mpk_linux_sandbox::ProcessControls {
            open_files: RESOURCE_OPEN_FILE_LIMIT,
            address_space_bytes: address_space_limit,
        },
    )
    .map_err(|_| SandboxError::Unavailable)
}

#[cfg(target_os = "linux")]
fn capture_resource_child_mount_namespace(
    child: &mut mpk_linux_sandbox::ResourceChild,
) -> Result<File, SandboxError> {
    let own_namespace = fs::metadata("/proc/self/ns/mnt").map_err(|_| SandboxError::Unavailable)?;
    let child_namespace_path = format!("/proc/{}/ns/mnt", child.id());
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(namespace) = File::open(&child_namespace_path) {
            let metadata = namespace
                .metadata()
                .map_err(|_| SandboxError::Unavailable)?;
            if (metadata.dev(), metadata.ino()) != (own_namespace.dev(), own_namespace.ino()) {
                return Ok(namespace);
            }
        }
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return Err(SandboxError::Unavailable),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            Ok(None) => return Err(SandboxError::Unavailable),
        }
    }
}

#[cfg(target_os = "linux")]
fn request_resource_kill(child: &mpk_linux_sandbox::ResourceChild, cgroup: &CgroupLeaf) -> bool {
    let cgroup_failed = fs::write(cgroup.path.join("cgroup.kill"), b"1\n").is_err();
    let child_failed = match child.kill() {
        Ok(()) => false,
        Err(error) if error.raw_os_error() == Some(rustix::io::Errno::SRCH.raw_os_error()) => false,
        Err(_) => true,
    };
    cgroup_failed || child_failed
}

#[cfg(target_os = "linux")]
fn cleanup_resource_process(
    child: &mut mpk_linux_sandbox::ResourceChild,
    cgroup: &CgroupLeaf,
    resource_mount_namespace: Option<File>,
) -> Result<(), SandboxError> {
    child.stdin.take();
    child.stdout.take();
    child.stderr.take();
    let signal_failed = request_resource_kill(child, cgroup);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            Ok(None) | Err(_) => std::process::abort(),
        }
    }
    let validation = cgroup.terminate_and_validate();
    if cgroup.require_unpopulated().is_err() {
        std::process::abort();
    }
    drop(resource_mount_namespace);
    if signal_failed {
        Err(SandboxError::Killed)
    } else {
        validation
    }
}

#[cfg(target_os = "linux")]
fn capture_child_mount_namespace(child: &mut std::process::Child) -> Result<File, SandboxError> {
    let own_namespace = fs::metadata("/proc/self/ns/mnt").map_err(|_| SandboxError::Unavailable)?;
    let child_namespace_path = format!("/proc/{}/ns/mnt", child.id());
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(namespace) = File::open(&child_namespace_path) {
            let metadata = namespace
                .metadata()
                .map_err(|_| SandboxError::Unavailable)?;
            if (metadata.dev(), metadata.ino()) != (own_namespace.dev(), own_namespace.ino()) {
                return Ok(namespace);
            }
        }
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return Err(SandboxError::Unavailable),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            Ok(None) => return Err(SandboxError::Unavailable),
        }
    }
}

#[cfg(any(test, target_os = "linux"))]
fn kill_execution(child: &mut std::process::Child, resource_cgroup: Option<&CgroupLeaf>) {
    #[cfg(target_os = "linux")]
    if let Some(cgroup) = resource_cgroup {
        cgroup.kill_best_effort();
    } else {
        kill_child_tree(child);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = resource_cgroup;
        kill_child_tree(child);
    }
}

#[cfg(target_os = "linux")]
fn cleanup_spawned_process(
    child: &mut std::process::Child,
    resource_cgroup: Option<&CgroupLeaf>,
    resource_mount_namespace: Option<File>,
) -> Result<(), SandboxError> {
    kill_execution(child, resource_cgroup);
    let wait_failed = child.wait().is_err();
    let validation = match resource_cgroup.as_ref() {
        Some(cgroup) => cgroup.terminate_and_validate(),
        None => Ok(()),
    };
    if resource_cgroup
        .as_ref()
        .is_some_and(|cgroup| cgroup.require_unpopulated().is_err())
    {
        std::process::abort();
    }
    drop(resource_mount_namespace);
    if wait_failed {
        std::process::abort();
    }
    validation
}

fn cleanup_dominates(original: SandboxError, cleanup: Result<(), SandboxError>) -> SandboxError {
    cleanup.err().unwrap_or(original)
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
                    observed = match observed.checked_add(count) {
                        Some(observed) => observed,
                        None => {
                            overflow.store(true, Ordering::Release);
                            usize::MAX
                        }
                    };
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
fn materialize_sources(
    inputs: &[CapturedInput<'_>],
    staged_directories: &[&str],
    staged_placeholders: &[&str],
    root: &Path,
) -> Result<(), SandboxError> {
    fs::create_dir_all(root).map_err(|_| SandboxError::Unavailable)?;
    let mut occupied_paths = BTreeSet::new();
    for relative in staged_directories {
        if !occupied_paths.insert(*relative) {
            return Err(SandboxError::Unavailable);
        }
        let path = private_materialized_path(root, relative)?;
        fs::create_dir_all(path).map_err(|_| SandboxError::Unavailable)?;
    }
    for relative in staged_placeholders {
        if !occupied_paths.insert(*relative) {
            return Err(SandboxError::Unavailable);
        }
        let path = private_materialized_path(root, relative)?;
        let parent = path.parent().ok_or(SandboxError::Unavailable)?;
        fs::create_dir_all(parent).map_err(|_| SandboxError::Unavailable)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| SandboxError::Unavailable)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444))
            .map_err(|_| SandboxError::Unavailable)?;
    }
    let mut folded_paths = BTreeSet::new();
    for input in inputs {
        mpk_vc::validate_manifest_normalized_path(input.normalized_path)
            .map_err(|_| SandboxError::Unavailable)?;
        if !folded_paths.insert(input.normalized_path.to_ascii_lowercase())
            || !occupied_paths.insert(input.normalized_path)
        {
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
fn private_materialized_path(root: &Path, relative: &str) -> Result<PathBuf, SandboxError> {
    let path = Path::new(relative);
    if relative.is_empty()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(root.join(path))
}

#[cfg(target_os = "linux")]
fn seal_read_only_tree(root: &Path) -> Result<(), SandboxError> {
    let mut pending = vec![(root.to_owned(), false)];
    while let Some((path, visited)) = pending.pop() {
        if visited {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o555))
                .map_err(|_| SandboxError::Unavailable)?;
            continue;
        }
        let mut entries = fs::read_dir(&path)
            .map_err(|_| SandboxError::Unavailable)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SandboxError::Unavailable)?;
        entries.sort();
        pending.push((path, true));
        for entry in entries.into_iter().rev() {
            if fs::symlink_metadata(&entry)
                .map_err(|_| SandboxError::Unavailable)?
                .is_dir()
            {
                pending.push((entry, false));
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn unseal_private_tree(root: &Path) -> Result<(), SandboxError> {
    let mut pending = vec![root.to_owned()];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(|_| SandboxError::Unavailable)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(SandboxError::Unavailable);
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .map_err(|_| SandboxError::Unavailable)?;
        let mut entries = fs::read_dir(&path)
            .map_err(|_| SandboxError::Unavailable)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SandboxError::Unavailable)?;
        entries.sort();
        for entry in entries.into_iter().rev() {
            let metadata = fs::symlink_metadata(&entry).map_err(|_| SandboxError::Unavailable)?;
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                pending.push(entry);
            }
        }
    }
    Ok(())
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

/// Source-free fault injection for T07's explicit native gate. This exists
/// only in test executables and uses the same Java leaf, limits, supervision
/// and cleanup as the registered JVM. It cannot select an external payload.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn test_java_resource_boundary(case: &str) -> Result<(), SandboxError> {
    if !["oom", "pids", "timeout", "stdout", "stderr", "tmpfs"].contains(&case) {
        return Err(SandboxError::Unavailable);
    }
    let PreparedSandbox {
        mut resource_session,
        ..
    } = prepare_release_sandbox(mpk_vc::java_release::HOST_ID)?;
    let session = resource_session.as_mut().ok_or(SandboxError::Unavailable)?;
    session.prepare_next_resource_leaf()?;
    let temporary = PrivateTempDir::create("mpk-java-resource-test-")?;
    let started = Instant::now();
    let output = run_resource_process_with_limits(
        temporary.path(),
        &["__mpk_java_resource_fault_v0".to_owned(), case.to_owned()],
        &BTreeMap::new(),
        Duration::from_secs(mpk_vc::java_release::TIMEOUT_SECONDS),
        268_435_456,
        2_097_152,
        session,
        false,
    );
    let elapsed = started.elapsed();
    let pids = read_flat_counters(&session.path.join("pids.events"))?;
    let memory = read_flat_counters(&session.path.join("memory.events.local"))?;
    let observed = match case {
        "oom" => {
            matches!(output, Err(SandboxError::Killed))
                && memory.get("oom_kill").is_some_and(|value| *value > 0)
        }
        "pids" => {
            matches!(output, Err(SandboxError::Killed))
                && pids.get("max").is_some_and(|value| *value > 0)
        }
        "timeout" => {
            matches!(output, Err(SandboxError::Killed))
                && elapsed >= Duration::from_secs(mpk_vc::java_release::TIMEOUT_SECONDS)
        }
        "stdout" | "stderr" => output.is_ok_and(|output| output.stream_limit_exceeded),
        "tmpfs" => output.is_ok_and(|output| {
            output.exit_code == Some(0)
                && !output.signaled
                && output.stdout.is_empty()
                && output.stderr_observed_bytes == 0
        }),
        _ => false,
    };
    temporary.remove();
    // Limit events intentionally make the first result Killed. Cleanup must
    // nevertheless remove the leaf and restore the exact manager/domain state.
    let release = session.finish_resource_after_backing_release();
    session.finish_session()?;
    if !observed
        || (matches!(case, "oom" | "pids") && !matches!(release, Err(SandboxError::Killed)))
        || (!matches!(case, "oom" | "pids") && release.is_err())
    {
        return Err(SandboxError::Unavailable);
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
pub(crate) fn run_java_resource_test_child(case: &str) -> u8 {
    let run = || -> Result<(), u8> {
        let directory = current_cgroup_directory().map_err(|_| 125)?;
        verify_self_in_resource_cgroup(&directory, SandboxProfile::Java25)?;
        verify_resource_process_controls(mpk_vc::java_release::ADDRESS_SPACE_BYTES)?;
        verify_only_standard_descriptors()?;
        match case {
            "oom" => {
                let mut bytes = vec![0_u8; 2 * mpk_vc::java_release::MEMORY_BYTES as usize];
                for page in bytes.chunks_mut(4096) {
                    page[0] = 1;
                    std::hint::black_box(&*page);
                }
                std::hint::black_box(bytes);
                Err(125) // Surviving the real accounting limit is a failed test.
            }
            "pids" => {
                let mut threads = Vec::new();
                for _ in 0..=mpk_vc::java_release::PIDS {
                    match thread::Builder::new()
                        .stack_size(65_536)
                        .spawn(|| thread::sleep(Duration::from_secs(180)))
                    {
                        Ok(thread) => threads.push(thread),
                        Err(error) if error.raw_os_error() == Some(11) => return Ok(()),
                        Err(_) => return Err(125),
                    }
                }
                Err(125)
            }
            "timeout" => {
                thread::sleep(Duration::from_secs(180));
                Err(125)
            }
            "stdout" | "stderr" => {
                let bytes = [b'x'; 65_536];
                let count = if case == "stdout" { 4097 } else { 33 };
                for _ in 0..count {
                    if case == "stdout" {
                        io::stdout().write_all(&bytes).map_err(|_| 125)?;
                    } else {
                        io::stderr().write_all(&bytes).map_err(|_| 125)?;
                    }
                }
                Ok(())
            }
            "tmpfs" => test_java_tmpfs_capacity(),
            _ => Err(125),
        }
    };
    match run() {
        Ok(()) => 0,
        Err(code) => code,
    }
}

#[cfg(all(test, target_os = "linux"))]
#[allow(deprecated)]
fn test_java_tmpfs_capacity() -> Result<(), u8> {
    use rustix::mount::{
        mount, mount_change, unmount, MountFlags, MountPropagationFlags, UnmountFlags,
    };
    rustix::thread::unshare(rustix::thread::UnshareFlags::NEWNS).map_err(|_| 125)?;
    mount_change(
        "/",
        MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
    )
    .map_err(|_| 125)?;
    let root = std::env::current_dir().map_err(|_| 125)?;
    let target = root.join("tmpfs");
    fs::create_dir(&target).map_err(|_| 125)?;
    mount(
        "tmpfs",
        &target,
        "tmpfs",
        MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC,
        Some(c"size=67108864,nr_inodes=262144,noswap,mode=700"),
    )
    .map_err(|_| 125)?;
    validate_resource_tmpfs_mount(
        &target,
        mpk_vc::java_release::TMPFS_BYTES,
        WRITABLE_INODE_LIMIT,
        true,
    )?;
    let mut file = File::create(target.join("bytes")).map_err(|_| 125)?;
    let chunk = [0_u8; 65_536];
    for _ in 0..1024 {
        file.write_all(&chunk).map_err(|_| 125)?;
    }
    if !matches!(file.write_all(&[1]), Err(error) if error.raw_os_error() == Some(28)) {
        return Err(125);
    }
    drop(file);
    unmount(&target, UnmountFlags::empty()).map_err(|_| 125)?;
    fs::remove_dir(target).map_err(|_| 125)?;
    Ok(())
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

    let resource_cgroup = resource_probe_cgroup()?;
    let java_probe = resource_cgroup
        .as_ref()
        .is_some_and(|(_, profile)| *profile == SandboxProfile::Java25);
    if let Some((cgroup, profile)) = resource_cgroup.as_ref() {
        validate_minimum_kernel_version([6, 4, 0])?;
        verify_only_standard_descriptors()?;
        verify_self_in_resource_cgroup(cgroup, *profile)?;
        verify_resource_process_controls(RESOURCE_ADDRESS_SPACE_LIMIT)?;
        unshare(UnshareFlags::NEWNS).map_err(|_| 125)?;
        mount_change(
            "/",
            MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
        )
        .map_err(|_| 125)?;
        let root = std::env::current_dir().map_err(|_| 125)?;
        probe_resource_tmpfs(&root)?;
    } else {
        validate_minimum_kernel_version([5, 10, 0])?;
    }
    let uid = rustix::process::getuid().as_raw();
    let gid = rustix::process::getgid().as_raw();
    if java_probe {
        rustix::thread::set_thread_groups(&[]).map_err(|_| 125)?;
    }
    unshare(UnshareFlags::NEWUSER).map_err(|_| 125)?;
    fs::write("/proc/self/setgroups", b"deny\n").map_err(|_| 125)?;
    let mapped_identity = if java_probe { 65534 } else { 0 };
    fs::write("/proc/self/uid_map", format!("{mapped_identity} {uid} 1\n")).map_err(|_| 125)?;
    fs::write("/proc/self/gid_map", format!("{mapped_identity} {gid} 1\n")).map_err(|_| 125)?;
    unshare(
        UnshareFlags::NEWNS | UnshareFlags::NEWNET | UnshareFlags::NEWIPC | UnshareFlags::NEWUTS,
    )
    .map_err(|_| 125)?;
    if java_probe {
        unshare(UnshareFlags::NEWPID).map_err(|_| 125)?;
    }
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

    if java_probe {
        mpk_linux_sandbox::install_java_probe_policy().map_err(|_| 125)?;
    }
    if let Some((_, profile)) = resource_cgroup {
        println!("{} ok", profile.probe_id());
    } else {
        println!("mpk.release.probe.v0 ok");
    }
    Ok(0)
}

#[cfg(target_os = "linux")]
fn resource_probe_cgroup() -> Result<Option<(PathBuf, SandboxProfile)>, u8> {
    let contents = match fs::read_to_string(RESOURCE_PROBE_MARKER) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(125),
    };
    if contents.len() > 4_096 || !contents.ends_with('\n') {
        return Err(125);
    }
    let mut lines = contents.lines();
    let profile = match lines.next() {
        Some(RESOURCE_PROBE_PROFILE_ID) => SandboxProfile::Cgroup2Tmpfs,
        Some("mpk.release.probe.java25.v0") => SandboxProfile::Java25,
        _ => return Err(125),
    };
    let path = lines.next().ok_or(125)?;
    if lines.next().is_some() {
        return Err(125);
    }
    Ok(Some((PathBuf::from(path), profile)))
}

#[cfg(target_os = "linux")]
fn probe_resource_tmpfs(root: &Path) -> Result<(), u8> {
    use rustix::mount::{mount, unmount, MountFlags, UnmountFlags};

    let target = root.join("resource-tmpfs");
    fs::create_dir(&target).map_err(|_| 125)?;
    mount(
        "tmpfs",
        &target,
        "tmpfs",
        MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC,
        Some(c"size=4096,nr_inodes=4,noswap,mode=700"),
    )
    .map_err(|_| 125)?;
    validate_resource_tmpfs_mount(&target, 4_096, 4, true)?;
    let mut created = Vec::new();
    for index in 0..3 {
        let path = target.join(format!("inode-{index}"));
        File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| 125)?;
        created.push(path);
    }
    let excess = target.join("inode-excess");
    match File::options().write(true).create_new(true).open(excess) {
        Err(error) if error.raw_os_error() == Some(28) => {}
        _ => return Err(125),
    }
    for path in created {
        fs::remove_file(path).map_err(|_| 125)?;
    }
    unmount(&target, UnmountFlags::empty()).map_err(|_| 125)?;
    fs::remove_dir(&target).map_err(|_| 125)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_resource_tmpfs_mount(
    target: &Path,
    allocated_bytes: u64,
    inodes: u64,
    require_noexec: bool,
) -> Result<(), u8> {
    use rustix::fs::{statvfs, StatVfsMountFlags};

    let state = statvfs(target).map_err(|_| 125)?;
    let allocated_capacity = state.f_blocks.checked_mul(state.f_frsize).ok_or(125)?;
    if allocated_capacity != allocated_bytes
        || state.f_files != inodes
        || !state
            .f_flag
            .contains(StatVfsMountFlags::NOSUID | StatVfsMountFlags::NODEV)
        || (require_noexec && !state.f_flag.contains(StatVfsMountFlags::NOEXEC))
        || !mountinfo_has_noswap(target)?
    {
        return Err(125);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn mountinfo_has_noswap(target: &Path) -> Result<bool, u8> {
    let target = target.to_str().ok_or(125)?;
    let mountinfo =
        read_bounded_file(Path::new("/proc/self/mountinfo"), 16 * 1024 * 1024).map_err(|_| 125)?;
    let mountinfo = std::str::from_utf8(&mountinfo).map_err(|_| 125)?;
    let mut matches = mountinfo.lines().filter_map(|line| {
        let (mount_fields, filesystem_fields) = line.split_once(" - ")?;
        let mount_point = mount_fields.split_ascii_whitespace().nth(4)?;
        if mount_point != target {
            return None;
        }
        let mut filesystem_fields = filesystem_fields.split_ascii_whitespace();
        let filesystem = filesystem_fields.next()?;
        let _source = filesystem_fields.next()?;
        let super_options = filesystem_fields.next()?;
        (filesystem_fields.next().is_none()).then_some((filesystem, super_options))
    });
    let Some((filesystem, super_options)) = matches.next() else {
        return Ok(false);
    };
    if matches.next().is_some() || filesystem != "tmpfs" {
        return Ok(false);
    }
    Ok(super_options.split(',').any(|option| option == "noswap"))
}

#[cfg(target_os = "linux")]
fn validate_minimum_kernel_version(minimum: [u32; 3]) -> Result<(), u8> {
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").map_err(|_| 125)?;
    let numeric = release
        .trim()
        .split(['.', '-'])
        .take(3)
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| 125)?;
    if numeric.len() != 3 || numeric.as_slice() < minimum.as_slice() {
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

    let (resource_cgroup, arguments) = match arguments {
        [marker, cgroup, remaining @ ..] if marker == RESOURCE_BOOTSTRAP_MARKER => {
            (Some(Path::new(cgroup)), remaining)
        }
        _ => (None, arguments),
    };
    let Some((executable, arguments)) = arguments.split_first() else {
        return Err(125);
    };
    let resource_frontend = matches!(
        executable.as_str(),
        "bin/rust2vir" | CSHARP_BOOTSTRAP_EXECUTABLE | JAVA_BOOTSTRAP_EXECUTABLE
    );
    let csharp_frontend = executable == CSHARP_BOOTSTRAP_EXECUTABLE;
    let java_frontend = executable == JAVA_BOOTSTRAP_EXECUTABLE;
    let profile = if java_frontend {
        SandboxProfile::Java25
    } else {
        SandboxProfile::Cgroup2Tmpfs
    };
    if !matches!(
        executable.as_str(),
        "bin/go2vir" | "bin/rust2vir" | CSHARP_BOOTSTRAP_EXECUTABLE | JAVA_BOOTSTRAP_EXECUTABLE
    ) || resource_cgroup.is_some() != resource_frontend
    {
        return Err(125);
    }
    if let Some(cgroup) = resource_cgroup {
        verify_only_standard_descriptors()?;
        verify_self_in_resource_cgroup(cgroup, profile)?;
        verify_resource_process_controls(if csharp_frontend {
            CSHARP_RESOURCE_ADDRESS_SPACE_LIMIT
        } else {
            RESOURCE_ADDRESS_SPACE_LIMIT
        })?;
    }
    if java_frontend {
        let expected = &mpk_vc::java_release::ARGV_PREFIX[1..];
        if arguments.len() <= expected.len()
            || !arguments
                .iter()
                .take(expected.len())
                .map(String::as_str)
                .eq(expected.iter().copied())
            || std::env::vars().collect::<BTreeMap<_, _>>() != mpk_vc::java_release::environment()
        {
            return Err(125);
        }
        // Supplementary host groups must not survive the user namespace.
        rustix::thread::set_thread_groups(&[]).map_err(|_| 125)?;
    }
    if executable == "bin/go2vir" {
        for (resource, limit) in [
            (Resource::Core, 0),
            (Resource::Nofile, 256),
            (Resource::As, 4_294_967_296),
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
        for (resource, limit) in [(Resource::Fsize, 536_870_912), (Resource::Nproc, 256)] {
            setrlimit(
                resource,
                Rlimit {
                    current: Some(limit),
                    maximum: Some(limit),
                },
            )
            .map_err(|_| 125)?;
        }
    }
    if resource_frontend {
        unshare(UnshareFlags::NEWNS).map_err(|_| 125)?;
        mount_change(
            "/",
            MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
        )
        .map_err(|_| 125)?;
        let root = std::env::current_dir().map_err(|_| 125)?;
        mount_bind(&root, &root).map_err(|_| 125)?;
        let target = root.join("mpk/tmp");
        let privileged_mount = target.join(".mpk-resource-tmpfs");
        fs::create_dir(&privileged_mount).map_err(|_| 125)?;
        mount(
            "tmpfs",
            &privileged_mount,
            "tmpfs",
            MountFlags::NOSUID | MountFlags::NODEV,
            Some(if java_frontend {
                c"size=67108864,nr_inodes=262144,noswap,mode=700"
            } else {
                c"size=21474836480,nr_inodes=262144,noswap,mode=700"
            }),
        )
        .map_err(|_| 125)?;
        validate_resource_tmpfs_mount(
            &privileged_mount,
            profile.tmpfs(),
            WRITABLE_INODE_LIMIT,
            false,
        )?;
        let writable_root = privileged_mount.join("root");
        fs::create_dir(&writable_root).map_err(|_| 125)?;
        fs::set_permissions(&writable_root, fs::Permissions::from_mode(0o700)).map_err(|_| 125)?;
    }
    let uid = rustix::process::getuid().as_raw();
    let gid = rustix::process::getgid().as_raw();
    unshare(UnshareFlags::NEWUSER).map_err(|_| 125)?;
    fs::write("/proc/self/setgroups", b"deny\n").map_err(|_| 125)?;
    let mapped_identity = if java_frontend { 65534 } else { 0 };
    fs::write("/proc/self/uid_map", format!("{mapped_identity} {uid} 1\n")).map_err(|_| 125)?;
    fs::write("/proc/self/gid_map", format!("{mapped_identity} {gid} 1\n")).map_err(|_| 125)?;
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
    if !resource_frontend {
        mount_bind(&root, &root).map_err(|_| 125)?;
    }
    if resource_frontend {
        let target = root.join("mpk/tmp");
        let writable_root = target.join(".mpk-resource-tmpfs/root");
        mount_bind(&writable_root, &target).map_err(|_| 125)?;
        mount_remount(
            &target,
            MountFlags::BIND | MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC,
            "",
        )
        .map_err(|_| 125)?;
        validate_resource_tmpfs_mount(&target, profile.tmpfs(), WRITABLE_INODE_LIMIT, true)?;
    } else {
        for (relative, options) in [
            ("mpk/cache/go-build", c"size=536870912,mode=700"),
            ("mpk/tmp", c"size=536870912,mode=700"),
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
    if csharp_frontend || java_frontend {
        let urandom = fs::metadata("/dev/urandom").map_err(|_| 125)?;
        if !urandom.file_type().is_char_device() || urandom.rdev() != 265 {
            return Err(125);
        }
        let urandom_target = root.join("dev/urandom");
        mount_bind("/dev/urandom", &urandom_target).map_err(|_| 125)?;
        let mounted_urandom = fs::metadata(&urandom_target).map_err(|_| 125)?;
        if mounted_urandom.dev() != urandom.dev()
            || mounted_urandom.ino() != urandom.ino()
            || mounted_urandom.rdev() != urandom.rdev()
        {
            return Err(125);
        }
        mount_remount(
            &urandom_target,
            MountFlags::BIND | MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NOEXEC,
            "",
        )
        .map_err(|_| 125)?;
    }
    if resource_frontend {
        let runtime_source = root.join("mpk/toolchain/native-runtime");
        let runtime = if csharp_frontend || java_frontend {
            let runtime = root.join("mpk/native-runtime");
            mount_bind(&runtime_source, &runtime).map_err(|_| 125)?;
            mount_remount(
                &runtime,
                MountFlags::BIND | MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NODEV,
                "",
            )
            .map_err(|_| 125)?;
            runtime
        } else {
            runtime_source
        };
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
    std::env::set_current_dir(if csharp_frontend || java_frontend {
        "/mpk/source"
    } else {
        "/"
    })
    .map_err(|_| 125)?;
    set_no_new_privs(true).map_err(|_| 125)?;
    if resource_cgroup.is_some() {
        // The launcher opens this mount namespace before releasing the
        // bootstrap. Its namespace descriptor keeps the aggregate tmpfs alive
        // until all bounded output has been copied.
        let mut acknowledgement = [0xff];
        io::stdin()
            .read_exact(&mut acknowledgement)
            .map_err(|_| 125)?;
        if acknowledgement != [0] {
            return Err(125);
        }
    }
    let status = if csharp_frontend || java_frontend {
        let program = if java_frontend {
            mpk_vc::java_release::PROGRAM
        } else {
            "/mpk/toolchain/dotnet/dotnet"
        };
        let mut child_arguments = Vec::with_capacity(arguments.len() + 1);
        child_arguments.push(OsString::from(program));
        child_arguments.extend(arguments.iter().map(OsString::from));
        let null = File::open("/dev/null").map_err(|_| 125)?;
        let launch = if java_frontend {
            mpk_linux_sandbox::run_java_pid_namespace_process_with_proc
        } else {
            mpk_linux_sandbox::run_pending_pid_namespace_process_with_proc
        };
        launch(Path::new(program), child_arguments, null).map_err(|error| match error {
            mpk_linux_sandbox::LaunchError::CloneOrSetup(_) => 125,
            mpk_linux_sandbox::LaunchError::Exec(_) => 126,
        })?
    } else {
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
        command
            .args(arguments)
            .stdin(Stdio::null())
            .status()
            .map_err(|_| 126)?
    };
    if status.signal().is_some() {
        let _ =
            rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL);
        return Err(125);
    }
    let exit_code = status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .ok_or(125)?;
    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn execution_host_profile_selects_one_closed_sandbox_contract() {
        assert_eq!(
            sandbox_profile(LEGACY_EXECUTION_HOST_PROFILE_ID),
            Ok(SandboxProfile::LegacyNamespaces)
        );
        assert_eq!(
            sandbox_profile(RUST_EXECUTION_HOST_PROFILE_ID),
            Ok(SandboxProfile::Cgroup2Tmpfs)
        );
        assert_eq!(
            sandbox_profile(mpk_vc::java_release::HOST_ID),
            Ok(SandboxProfile::Java25)
        );
        assert_eq!(
            sandbox_profile("mpk.host.linux-x86_64-gnu.glibc2_27.v0"),
            Err(SandboxError::Unavailable)
        );
    }

    #[test]
    fn cgroup_event_observation_is_strict_and_peak_is_inclusive() {
        let pids = parse_flat_counters("max 0\n").expect("pids.events parses");
        let memory =
            parse_flat_counters("low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n")
                .expect("memory.events.local parses");
        assert!(resource_counter_values_are_clean(
            &pids,
            &memory,
            CGROUP_MEMORY_LIMIT
        ));
        assert!(!resource_counter_values_are_clean(
            &pids,
            &memory,
            CGROUP_MEMORY_LIMIT + 1
        ));

        for event in ["high", "max", "oom", "oom_kill"] {
            let mut observed = memory.clone();
            observed.insert(event.to_owned(), 1);
            assert!(!resource_counter_values_are_clean(&pids, &observed, 0));
        }
        let mut pids_exceeded = pids.clone();
        pids_exceeded.insert("max".to_owned(), 1);
        assert!(!resource_counter_values_are_clean(
            &pids_exceeded,
            &memory,
            0
        ));

        for malformed in [
            "max 0",
            "max 0\nmax 0\n",
            "max 0 extra\n",
            "MAX 0\n",
            "max -1\n",
        ] {
            assert!(parse_flat_counters(malformed).is_none());
        }
    }

    #[test]
    fn memory_discharge_uses_byte_gauges_instead_of_per_cpu_stock() {
        let discharged = parse_flat_counters(
            "anon 0\nfile 0\nkernel 32768\nsock 0\nshmem 0\nzswap 0\nzswapped 0\npgfault 42\n",
        )
        .expect("memory.stat parses");
        assert!(resource_memory_values_are_discharged(&discharged));

        for gauge in ["anon", "file", "sock", "shmem", "zswap", "zswapped"] {
            let mut observed = discharged.clone();
            observed.insert(gauge.to_owned(), 1);
            assert!(!resource_memory_values_are_discharged(&observed));
        }

        let mut without_optional_zswap = discharged.clone();
        without_optional_zswap.remove("zswap");
        without_optional_zswap.remove("zswapped");
        assert!(resource_memory_values_are_discharged(
            &without_optional_zswap
        ));
        for required in ["anon", "file", "sock", "shmem"] {
            let mut missing = discharged.clone();
            missing.remove(required);
            assert!(!resource_memory_values_are_discharged(&missing));
        }
    }

    #[test]
    fn rust_resource_profile_constants_match_the_frozen_units() {
        assert_eq!(CGROUP_TASK_LIMIT, 256);
        assert_eq!(CGROUP_MEMORY_LIMIT, 34_359_738_368);
        assert_eq!(CGROUP_SWAP_LIMIT, 0);
        assert_eq!(WRITABLE_ALLOCATED_BYTES_LIMIT, 21_474_836_480);
        assert_eq!(WRITABLE_INODE_LIMIT, 262_144);
    }

    #[test]
    fn java_resource_profile_is_distinct_and_matches_its_frozen_probe() {
        let vector: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../develop/specs/vectors/java-profile-v0.json"
        ))
        .unwrap();
        let probe = &vector["host_probe"];
        assert_eq!(
            SandboxProfile::Java25.memory(),
            probe["cgroup_controls_verified"]["memory.max"]
        );
        assert_eq!(
            SandboxProfile::Java25.pids(),
            probe["cgroup_controls_verified"]["pids.max"]
        );
        assert_eq!(SandboxProfile::Java25.tmpfs(), probe["tmpfs_bytes"]);
        assert_eq!(
            CGROUP_SWAP_LIMIT,
            probe["cgroup_controls_verified"]["memory.swap.max"]
        );
        assert_eq!(RESOURCE_ADDRESS_SPACE_LIMIT, probe["address_space_bytes"]);
        assert_eq!(RESOURCE_OPEN_FILE_LIMIT, probe["open_files"]);
        assert_ne!(
            SandboxProfile::Java25.memory(),
            SandboxProfile::Cgroup2Tmpfs.memory()
        );
        assert_ne!(
            SandboxProfile::Java25.tmpfs(),
            SandboxProfile::Cgroup2Tmpfs.tmpfs()
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cgroup_mount_and_generated_name_checks_reject_hidden_or_ambiguous_roots() {
        assert!(initial_cgroup_namespace_is_exact(0xEFFF_FFFB));
        assert!(!initial_cgroup_namespace_is_exact(0xF000_0001));
        let exact = "31 24 0:29 / /sys/fs/cgroup rw,nosuid,nodev,noexec - cgroup2 cgroup rw\n";
        assert!(global_cgroup2_mountinfo_is_exact(exact));
        for rejected in [
            "31 24 0:29 /delegated /sys/fs/cgroup rw,nosuid,nodev,noexec - cgroup2 cgroup rw\n",
            "31 24 0:29 / /other rw - cgroup2 cgroup rw\n",
            "31 24 0:29 / /sys/fs/cgroup ro - cgroup2 cgroup rw\n",
            "31 24 0:29 / /sys/fs/cgroup rw - cgroup2 cgroup rw\n32 24 0:30 / /other rw - cgroup2 cgroup rw\n",
            "malformed\n",
        ] {
            assert!(!global_cgroup2_mountinfo_is_exact(rejected), "{rejected:?}");
        }

        assert!(valid_resource_cgroup_name("mpk-rust-frontend-123-0"));
        for rejected in [
            "mpk-rust-frontend-",
            "mpk-rust-frontend-0-1",
            "mpk-rust-frontend-01-1",
            "mpk-rust-frontend-1-01",
            "mpk-rust-frontend-1-1-1",
            "mpk-rust-launcher-1-1",
        ] {
            assert!(!valid_resource_cgroup_name(rejected), "{rejected:?}");
        }
    }

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

    #[test]
    fn frontend_runner_freezes_the_first_kill_cause() {
        assert_eq!(
            first_execution_kill_cause(None, false, true),
            Some(ExecutionKillCause::Timeout)
        );
        assert_eq!(
            first_execution_kill_cause(Some(ExecutionKillCause::Timeout), true, true),
            Some(ExecutionKillCause::Timeout)
        );
        assert_eq!(
            first_execution_kill_cause(None, true, true),
            Some(ExecutionKillCause::StreamLimit)
        );
        assert_eq!(
            first_execution_kill_cause(Some(ExecutionKillCause::StreamLimit), true, true),
            Some(ExecutionKillCause::StreamLimit)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn frontend_runner_deadline_is_an_operational_kill() {
        let temporary = tempfile::tempdir().unwrap();
        let arguments = vec![
            "-c".to_owned(),
            "trap '' TERM; while :; do :; done".to_owned(),
        ];
        let started = Instant::now();
        let error = run_closed_process_with_timeout(
            Path::new("/bin/sh"),
            temporary.path(),
            &arguments,
            &BTreeMap::new(),
            Duration::from_millis(10),
            None,
        )
        .expect_err("the wall-clock deadline must kill a live frontend");

        assert_eq!(error, SandboxError::Killed);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn private_source_namespace_preserves_directories_and_noncandidate_names() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("source");
        let inputs = [CapturedInput {
            kind: mpk_vc::InputKind::Source,
            normalized_path: "main.go",
            bytes: b"package vector\n",
        }];
        materialize_sources(
            &inputs,
            &["native.c", "vendor"],
            &["ignored_test.go", "notes.txt", "vendor/modules.txt"],
            &root,
        )
        .unwrap();

        assert!(root.join("native.c").is_dir());
        assert!(root.join("vendor").is_dir());
        assert_eq!(fs::read(root.join("vendor/modules.txt")).unwrap(), b"");
        assert_eq!(fs::read(root.join("ignored_test.go")).unwrap(), b"");
        assert_eq!(fs::read(root.join("notes.txt")).unwrap(), b"");
        assert_eq!(fs::read(root.join("main.go")).unwrap(), b"package vector\n");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn private_source_namespace_seals_and_unseals_a_deep_legal_tree() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("source");
        let mut relative = String::new();
        let mut directories = Vec::new();
        for _ in 0..510 {
            if !relative.is_empty() {
                relative.push('/');
            }
            relative.push('d');
            directories.push(relative.clone());
        }
        let directory_refs = directories.iter().map(String::as_str).collect::<Vec<_>>();
        materialize_sources(&[], &directory_refs, &[], &root).unwrap();

        seal_read_only_tree(&root).unwrap();
        assert_eq!(
            fs::metadata(root.join(&relative))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o555
        );
        unseal_private_tree(&root).unwrap();
        assert_eq!(
            fs::metadata(root.join(relative))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn private_backing_cleanup_retains_ownership_until_absence_is_proven() {
        use std::os::unix::fs::symlink;

        let temporary = PrivateTempDir::create("mpk-private-cleanup-test-").unwrap();
        let path = temporary.path().to_owned();
        let restricted = path.join("restricted");
        fs::create_dir(&restricted).unwrap();
        fs::write(restricted.join("file"), b"private").unwrap();
        symlink("missing", restricted.join("link")).unwrap();
        fs::set_permissions(&restricted, fs::Permissions::from_mode(0o000)).unwrap();

        temporary.remove();
        assert!(matches!(
            fs::symlink_metadata(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound
        ));
    }
}
