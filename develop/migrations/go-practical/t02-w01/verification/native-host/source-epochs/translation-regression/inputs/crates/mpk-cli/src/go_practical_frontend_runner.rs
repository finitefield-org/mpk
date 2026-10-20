//! Private practical-Go native-host preparation. This is not a registered
//! semantic context, an input adapter or a source/program proof route.

use super::{prepare_release_sandbox, PreparedSandbox, SandboxError};

pub(super) const HOST_ID: &str = "mpk.host.linux-x86_64-gnu.go_practical.v1";
pub(super) const PROBE_ID: &str = "mpk.release.probe.go_practical.v1";
pub(super) const FRONTEND_MEMORY: u64 = 2_147_483_648;
pub(super) const FRONTEND_PIDS: u64 = 128;
pub(super) const FRONTEND_TMPFS: u64 = 536_870_912;
pub(super) const FRONTEND_INODES: u64 = 131_072;
pub(super) const FRONTEND_SECONDS: u64 = 120;
pub(super) const FRONTEND_TMPFS_OPTIONS: &std::ffi::CStr =
    c"size=536870912,nr_inodes=131072,noswap,mode=700";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoHostFailure {
    Environment,
    ResourcePending,
}

impl From<SandboxError> for GoHostFailure {
    fn from(error: SandboxError) -> Self {
        match error {
            SandboxError::Unavailable | SandboxError::Spawn => Self::Environment,
            SandboxError::Killed => Self::ResourcePending,
        }
    }
}

// No Clone/Deserialize, caller pathname, budget dial or synthesized receipt.
// A later complete original-input/context consumer must consume this owned
// preparation before publishing any practical-Go source-bound result.
pub(super) struct PreparedGoPracticalHost {
    _sandbox: PreparedSandbox,
}

#[cfg(target_os = "linux")]
pub(super) fn native_host_cpuinfo(cpuinfo: &str) -> bool {
    super::native_x86_64_cpuinfo(cpuinfo)
}

#[cfg(target_os = "linux")]
pub(super) fn require_native_host() -> Result<(), SandboxError> {
    let raw = super::read_bounded_file(std::path::Path::new("/proc/cpuinfo"), 16 * 1024 * 1024)?;
    let cpuinfo = std::str::from_utf8(&raw).map_err(|_| SandboxError::Unavailable)?;
    if native_host_cpuinfo(cpuinfo) {
        Ok(())
    } else {
        Err(SandboxError::Unavailable)
    }
}

pub(super) fn prepare() -> Result<PreparedGoPracticalHost, GoHostFailure> {
    let sandbox = prepare_release_sandbox(HOST_ID)?;
    Ok(PreparedGoPracticalHost { _sandbox: sandbox })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_practical_native_contract_matches_the_complete_frozen_frontend_limits() {
        let frozen: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../develop/migrations/go-practical/t01-w04/program-binding/host-resources.json"
        ))
        .expect("frozen original native host contract");
        let frontend = &frozen["frontend"];
        assert_eq!(frozen["host_profile_id"], HOST_ID);
        assert_eq!(frozen["probe_profile_id"], PROBE_ID);
        for (name, actual) in [
            ("cgroup_memory_max_bytes", FRONTEND_MEMORY),
            ("cgroup_memory_swap_max_bytes", 0),
            ("cgroup_pids_max", FRONTEND_PIDS),
            ("tmpfs_bytes", FRONTEND_TMPFS),
            ("tmpfs_inodes", FRONTEND_INODES),
            ("wall_ns", FRONTEND_SECONDS * 1_000_000_000),
            (
                "rlimit_as_bytes",
                super::super::RESOURCE_ADDRESS_SPACE_LIMIT,
            ),
            ("rlimit_nofile", super::super::RESOURCE_OPEN_FILE_LIMIT),
            ("rlimit_core_bytes", 0),
            ("direct_child_parallelism", 1),
        ] {
            assert_eq!(frontend[name], actual, "frozen frontend {name}");
        }
        assert_eq!(frontend["GOMAXPROCS"], "1");
        assert_eq!(frontend["GOMEMLIMIT"], "1610612736");
        assert_eq!(frozen["minimum_kernel_abi"], "6.4.0");
        assert_eq!(frozen["native_execution_required"], true);
        assert_eq!(
            FRONTEND_TMPFS_OPTIONS.to_bytes(),
            b"size=536870912,nr_inodes=131072,noswap,mode=700"
        );
    }

    #[test]
    fn go_practical_host_failures_never_become_source_or_theorem_rejections() {
        assert_eq!(
            GoHostFailure::from(SandboxError::Unavailable),
            GoHostFailure::Environment
        );
        assert_eq!(
            GoHostFailure::from(SandboxError::Spawn),
            GoHostFailure::Environment
        );
        assert_eq!(
            GoHostFailure::from(SandboxError::Killed),
            GoHostFailure::ResourcePending
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn go_practical_observed_translation_is_not_native_cpu_evidence() {
        let cpuinfo =
            std::fs::read_to_string("/proc/cpuinfo").expect("complete original CPU observation");
        if !cpuinfo.lines().any(|line| {
            line.split_once(':').is_some_and(|(key, value)| {
                key.trim() == "vendor_id" && value.trim() == "VirtualApple"
            })
        }) {
            eprintln!(
                "observed translation control is inapplicable; no native acceptance inferred"
            );
            return;
        }
        eprintln!("complete original translated CPU observation:\n{cpuinfo}");
        assert!(
            !native_host_cpuinfo(&cpuinfo),
            "translated x86 flags do not prove native execution"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn go_practical_host_has_no_non_linux_fallback() {
        assert!(matches!(prepare(), Err(GoHostFailure::Environment)));
    }
}
