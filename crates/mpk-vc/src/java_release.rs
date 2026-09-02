//! Frozen Java release identities and pure launcher construction.
//! No caller-selected executable, registry, or toolchain path is exposed here.

use crate::release_bundle_v1::{
    ResolvedSuccessorRelease, SuccessorBundleCandidate, SuccessorFrontendBundle,
    SuccessorToolchainBundle, ValidatedSuccessorReleaseRegistry,
};
use crate::semantic_profile_registry::SelectionEnvelope;
use crate::{ExecutionHostProfile, NativeRuntimeLayoutProfile};
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const FRONTEND_ID: &str = "frontend.java.java2vir.candidate.v2";
pub const TOOLCHAIN_ID: &str = "toolchain.java.temurin-25_0_4_1_1.candidate.v1";
pub const HOST_ID: &str = "mpk.host.linux-x86_64-gnu.java25.v0";
pub const LAYOUT_ID: &str = "mpk.runtime.linux-x86_64-gnu.java25.v0";
pub const PROGRAM: &str = "/mpk/toolchain/jdk/bin/java";
pub const ARGUMENT_BYTES_MAX: usize = 131_072;
pub const MEMORY_BYTES: u64 = 1_073_741_824;
pub const PIDS: u64 = 128;
pub const TMPFS_BYTES: u64 = 67_108_864;
pub const TIMEOUT_SECONDS: u64 = 120;
pub const ADDRESS_SPACE_BYTES: u64 = 17_179_869_184;
pub const OPEN_FILES: u64 = 1024;

pub const ARGV_PREFIX: &[&str] = &[
    PROGRAM,
    "-Xint",
    "-Xshare:off",
    "-XX:+UseSerialGC",
    "-XX:ActiveProcessorCount=1",
    "-XX:+DisableAttachMechanism",
    "-XX:-UsePerfData",
    "-Xms32m",
    "-Xmx512m",
    "-Xss1m",
    "-Dfile.encoding=UTF-8",
    "-Duser.language=en",
    "-Duser.country=US",
    "-Duser.timezone=UTC",
    "-Djava.io.tmpdir=/mpk/tmp",
    "-Duser.home=/mpk/empty-home",
    "-Djava.library.path=/nonexistent",
    "-XX:ErrorFile=/mpk/tmp/hs_err.log",
    "-XX:-CreateCoredumpOnCrash",
    "-XX:-HeapDumpOnOutOfMemoryError",
    "--limit-modules",
    "java.base,java.compiler,jdk.compiler,jdk.zipfs",
    "--add-modules",
    "java.compiler,jdk.compiler,jdk.zipfs",
    "-cp",
    "/mpk/frontend/java2vir.jar",
    "mpk.java2vir.Main",
];

pub fn environment() -> BTreeMap<String, String> {
    [
        ("HOME", "/mpk/empty-home"),
        ("TMPDIR", "/mpk/tmp"),
        ("PATH", "/nonexistent"),
        ("LANG", "C.UTF-8"),
        ("LC_ALL", "C.UTF-8"),
        ("TZ", "UTC"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

pub fn candidate() -> &'static SuccessorBundleCandidate {
    static VALUE: OnceLock<SuccessorBundleCandidate> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_slice(include_bytes!(
            "../../../release/bundles/candidates/java.json"
        ))
        .expect("compiled Java candidate descriptor")
    })
}

pub(crate) fn exact_release(
    frontend: &SuccessorFrontendBundle,
    toolchain: &SuccessorToolchainBundle,
    hosts: &[ExecutionHostProfile],
    layouts: &[NativeRuntimeLayoutProfile],
) -> bool {
    let expected = candidate();
    frontend == &expected.frontend_bundles[0]
        && toolchain == &expected.toolchain_bundles[0]
        && hosts.iter().find(|host| host.id == HOST_ID) == expected.execution_host_profiles.first()
        && layouts.iter().find(|layout| layout.id == LAYOUT_ID)
            == expected.native_runtime_layout_profiles.first()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaLaunchError {
    Release,
    Selection,
    Arguments,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaLauncherPlan {
    argv: Vec<String>,
    environment: BTreeMap<String, String>,
}

impl JavaLauncherPlan {
    pub fn argv(&self) -> &[String] {
        &self.argv
    }
    pub fn environment(&self) -> &BTreeMap<String, String> {
        &self.environment
    }
}

pub fn launcher_plan(
    release: &ValidatedSuccessorReleaseRegistry,
    resolved: &ResolvedSuccessorRelease<'_>,
    selection: &SelectionEnvelope,
) -> Result<JavaLauncherPlan, JavaLaunchError> {
    if !exact_release(
        resolved.frontend,
        resolved.toolchain,
        &release.registry().execution_host_profiles,
        &release.registry().native_runtime_layout_profiles,
    ) || serde_json::to_value(&resolved.semantic_context)
        .map_err(|_| JavaLaunchError::Release)?
        != candidate().tuples[0].semantic_context
    {
        return Err(JavaLaunchError::Release);
    }
    if selection.schema() != "mpk.selection.java_methods.v0" {
        return Err(JavaLaunchError::Selection);
    }
    let value = selection.value();
    let compilation = value["compilation"]
        .as_str()
        .ok_or(JavaLaunchError::Selection)?;
    let mut argv = ARGV_PREFIX
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    argv.extend(
        [
            "lower",
            "/mpk/source",
            "--semantic-profile",
            "mpk.java.scalar.v0",
            "--target",
            "linux-x64",
            "--compilation",
            compilation,
        ]
        .map(str::to_owned),
    );
    for (field, flag) in [
        ("sources", "--source"),
        ("contracts", "--contract"),
        ("methods", "--method"),
    ] {
        for item in value[field].as_array().ok_or(JavaLaunchError::Selection)? {
            argv.push(flag.to_owned());
            argv.push(item.as_str().ok_or(JavaLaunchError::Selection)?.to_owned());
        }
    }
    let profile = resolved.semantic_context.profile_registry();
    let revision = profile.revision().to_string();
    for (flag, value) in [
        ("--profile-registry-id", profile.id()),
        ("--profile-registry-revision", &revision),
        ("--profile-registry-sha256", profile.registry_sha256()),
        (
            "--profile-entry-sha256",
            resolved.semantic_context.profile_entry_sha256(),
        ),
        ("--frontend-bundle-id", FRONTEND_ID),
        ("--frontend-sha256", &resolved.frontend.main.binary_sha256),
        ("--release-registry-id", "mpk.release.registry.v1"),
        ("--release-registry-sha256", release.registry_sha256()),
        ("--toolchain-bundle-id", TOOLCHAIN_ID),
        ("--toolchain-root", "/mpk/toolchain"),
        (
            "--toolchain-distribution-sha256",
            &resolved.toolchain.distribution_sha256,
        ),
    ] {
        argv.push(flag.to_owned());
        argv.push(value.to_owned());
    }
    let bytes = argv.iter().try_fold(0_usize, |total, value| {
        total.checked_add(value.len().checked_add(1)?)
    });
    if bytes.is_none_or(|bytes| bytes > ARGUMENT_BYTES_MAX)
        || argv.iter().any(|value| value.as_bytes().contains(&0))
    {
        return Err(JavaLaunchError::Arguments);
    }
    Ok(JavaLauncherPlan {
        argv,
        environment: environment(),
    })
}
