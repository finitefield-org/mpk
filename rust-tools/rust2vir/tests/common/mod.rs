#![allow(dead_code)]

use rust2vir_internal::cli::{parse_lower_args, LowerRequest, SEMANTIC_PROFILE};
use rust2vir_internal::driver_protocol::{DriverComponentIdentity, DriverReleaseIdentity};
use rust2vir_internal::environment::{
    EvidenceEnvironment, ARGUMENT_PROFILE_ID, DRIVER_OUTPUT_ROOT, ENVIRONMENT_PROFILE_ID,
    HOME_ROOT, TARGET_ROOT, TEMP_ROOT,
};
use rust2vir_internal::manifest::{self, ExpectedManifestSelection};
use rust2vir_internal::metadata_request::MetadataRequest;
use rust2vir_internal::module_closure;
use rust2vir_internal::preflight;
use rust2vir_internal::sandbox::{
    component_content_sha256, frontend_bundle_sha256, toolchain_distribution_sha256,
    CandidateDefinition, CargoInvocation, CargoInvocationKind, InjectedCandidate, InventoryFile,
    ProcessOutput, SandboxContext, SandboxError, SandboxExecutor, SandboxLimits, TargetLibrary,
    EXECUTION_HOST_PROFILE_ID, FRONTEND_BUNDLE_ID, LIMIT_PROFILE_ID, RUNTIME_LAYOUT_PROFILE_ID,
    TOOLCHAIN_BUNDLE_ID,
};
use rust2vir_internal::sha256::{digest, hex};
use rust2vir_internal::snapshot::Snapshot;
use std::collections::{BTreeMap, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub struct Fixture {
    root: PathBuf,
    source: PathBuf,
    private_parent: PathBuf,
    request: LowerRequest,
    snapshot: Snapshot,
    metadata_request: Option<MetadataRequest>,
    candidate: InjectedCandidate,
}

impl Fixture {
    pub fn new() -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust2vir-cargo-launcher-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let source = root.join("source");
        let private_parent = root.join("private");
        let frontend = root.join("frontend");
        let toolchain = root.join("toolchain");
        for directory in [&source, &private_parent, &frontend, &toolchain] {
            fs::create_dir(directory).unwrap();
        }
        fs::create_dir(source.join("src")).unwrap();
        fs::create_dir(source.join("contracts")).unwrap();
        fs::write(
            source.join("Cargo.toml"),
            b"[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n\n[lib]\nname='vector'\npath='src/lib.rs'\n\n[features]\ndefault=[]\n",
        )
        .unwrap();
        fs::write(
            source.join("Cargo.lock"),
            b"version=4\n\n[[package]]\nname='vector'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(
            source.join("src/lib.rs"),
            b"pub fn identity(value: u8) -> u8 { value }\n",
        )
        .unwrap();
        fs::write(source.join("contracts/vector.json"), b"{}\n").unwrap();

        let frontend_files = [
            ("bin/rust2vir", b"frontend-main".as_slice(), true),
            ("bin/rust2vir-driver", b"frontend-driver".as_slice(), true),
        ];
        let toolchain_files = [
            ("bin/cargo", b"cargo".as_slice(), true),
            ("bin/rustc", b"rustc".as_slice(), true),
            (
                "lib/librustc_driver.so",
                b"compiler-runtime".as_slice(),
                false,
            ),
            (
                "lib/rustlib/i686-unknown-linux-gnu/lib/libstd.rlib",
                b"i686-std".as_slice(),
                false,
            ),
            (
                "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib",
                b"x86-std".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
                b"loader".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libc.so.6",
                b"libc".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libdl.so.2",
                b"libdl".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libgcc_s.so.1",
                b"libgcc".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libm.so.6",
                b"libm".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libpthread.so.0",
                b"libpthread".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/librt.so.1",
                b"librt".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libstdcxx.so.6",
                b"libstdcxx".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libtinfo.so.5",
                b"libtinfo".as_slice(),
                false,
            ),
            (
                "native-runtime/lib/x86_64-linux-gnu/libz.so.1",
                b"libz".as_slice(),
                false,
            ),
            (
                "native-runtime/lib64/ld-linux-x86-64.so.2",
                b"loader".as_slice(),
                true,
            ),
        ];
        let frontend_inventory = materialize_inventory(&frontend, &frontend_files);
        let toolchain_inventory = materialize_inventory(&toolchain, &toolchain_files);
        let frontend_sha256 = frontend_bundle_sha256(FRONTEND_BUNDLE_ID, &frontend_inventory);
        let distribution_sha256 =
            toolchain_distribution_sha256(TOOLCHAIN_BUNDLE_ID, &toolchain_inventory);
        let i686_content_sha256 = component_content_sha256(
            TOOLCHAIN_BUNDLE_ID,
            "rust-target-i686",
            &toolchain_inventory,
            "lib/rustlib/i686-unknown-linux-gnu/",
        );
        let x86_64_content_sha256 = component_content_sha256(
            TOOLCHAIN_BUNDLE_ID,
            "rust-target-x86_64",
            &toolchain_inventory,
            "lib/rustlib/x86_64-unknown-linux-gnu/",
        );
        let native_runtime_content_sha256 = component_content_sha256(
            TOOLCHAIN_BUNDLE_ID,
            "native-runtime",
            &toolchain_inventory,
            "native-runtime/",
        );
        let compiler_runtime_content_sha256 = component_content_sha256(
            TOOLCHAIN_BUNDLE_ID,
            "rust-compiler-runtime",
            &toolchain_inventory,
            "lib/librustc",
        );
        let cargo_sha256 = toolchain_inventory
            .iter()
            .find(|file| file.path() == "bin/cargo")
            .unwrap()
            .sha256()
            .to_owned();
        let rustc_sha256 = toolchain_inventory
            .iter()
            .find(|file| file.path() == "bin/rustc")
            .unwrap()
            .sha256()
            .to_owned();
        let frontend_binary_sha256 = frontend_inventory
            .iter()
            .find(|file| file.path() == "bin/rust2vir")
            .unwrap()
            .sha256()
            .to_owned();
        seal_tree(&frontend);
        seal_tree(&toolchain);

        let driver_sha256 = frontend_inventory
            .iter()
            .find(|file| file.path() == "bin/rust2vir-driver")
            .unwrap()
            .sha256()
            .to_owned();
        let request = parse_lower_args(arguments(
            &source,
            &toolchain,
            &frontend.join("bin/rust2vir-driver"),
            &driver_sha256,
            &frontend_binary_sha256,
            &distribution_sha256,
        ))
        .unwrap();
        let validated = manifest::validate(&request, preflight::run(&request).unwrap()).unwrap();
        let (closure, expected) = module_closure::discover(validated).unwrap();
        let snapshot = Snapshot::create(&private_parent, &closure).unwrap();
        let metadata_request = MetadataRequest::for_snapshot(&snapshot, expected).unwrap();

        let candidate = InjectedCandidate::from_definition(CandidateDefinition {
            frontend_bundle_id: FRONTEND_BUNDLE_ID.to_owned(),
            frontend_bundle_sha256: frontend_sha256,
            frontend_root: frontend,
            frontend_inventory,
            toolchain_bundle_id: TOOLCHAIN_BUNDLE_ID.to_owned(),
            toolchain_distribution_sha256: distribution_sha256.clone(),
            toolchain_root: toolchain,
            toolchain_inventory,
            target_libraries: vec![
                TargetLibrary::new(
                    rust2vir_internal::cli::RustTarget::I686UnknownLinuxGnu,
                    32,
                    "rust-target-i686",
                    i686_content_sha256.clone(),
                    "lib/rustlib/i686-unknown-linux-gnu/",
                )
                .unwrap(),
                TargetLibrary::new(
                    rust2vir_internal::cli::RustTarget::X86_64UnknownLinuxGnu,
                    64,
                    "rust-target-x86_64",
                    x86_64_content_sha256.clone(),
                    "lib/rustlib/x86_64-unknown-linux-gnu/",
                )
                .unwrap(),
            ],
            execution_host_profile_id: EXECUTION_HOST_PROFILE_ID.to_owned(),
            runtime_layout_profile_id: RUNTIME_LAYOUT_PROFILE_ID.to_owned(),
            environment_profile_id: ENVIRONMENT_PROFILE_ID.to_owned(),
            argument_profile_id: ARGUMENT_PROFILE_ID.to_owned(),
            limit_profile_id: LIMIT_PROFILE_ID.to_owned(),
            compiler_release: rust2vir_internal::EXPECTED_RUSTC_RELEASE.to_owned(),
            compiler_commit: rust2vir_internal::EXPECTED_RUSTC_COMMIT.to_owned(),
            native_runtime_component_root: "native-runtime".to_owned(),
            native_runtime_content_sha256: native_runtime_content_sha256.clone(),
            driver_release_identity: DriverReleaseIdentity {
                frontend_bundle_id: FRONTEND_BUNDLE_ID.to_owned(),
                frontend_binary_sha256,
                driver_binary_sha256: driver_sha256,
                toolchain_bundle_id: TOOLCHAIN_BUNDLE_ID.to_owned(),
                toolchain_distribution_sha256: distribution_sha256,
                toolchain_components: vec![
                    DriverComponentIdentity {
                        kind: "executable".to_owned(),
                        name: "cargo".to_owned(),
                        release: rust2vir_internal::EXPECTED_RUSTC_RELEASE.to_owned(),
                        sha256: cargo_sha256,
                        commit_hash: None,
                    },
                    DriverComponentIdentity {
                        kind: "content".to_owned(),
                        name: "native-runtime".to_owned(),
                        release: "nightly-2025-06-01".to_owned(),
                        sha256: native_runtime_content_sha256.clone(),
                        commit_hash: None,
                    },
                    DriverComponentIdentity {
                        kind: "content".to_owned(),
                        name: "rust-compiler-runtime".to_owned(),
                        release: "nightly-2025-06-01".to_owned(),
                        sha256: compiler_runtime_content_sha256,
                        commit_hash: None,
                    },
                    DriverComponentIdentity {
                        kind: "content".to_owned(),
                        name: "rust-target-i686".to_owned(),
                        release: "nightly-2025-06-01".to_owned(),
                        sha256: i686_content_sha256,
                        commit_hash: None,
                    },
                    DriverComponentIdentity {
                        kind: "content".to_owned(),
                        name: "rust-target-x86_64".to_owned(),
                        release: "nightly-2025-06-01".to_owned(),
                        sha256: x86_64_content_sha256,
                        commit_hash: None,
                    },
                    DriverComponentIdentity {
                        kind: "executable".to_owned(),
                        name: "rustc".to_owned(),
                        release: rust2vir_internal::EXPECTED_RUSTC_RELEASE.to_owned(),
                        sha256: rustc_sha256,
                        commit_hash: Some(rust2vir_internal::EXPECTED_RUSTC_COMMIT.to_owned()),
                    },
                ],
            },
        })
        .unwrap();

        Self {
            root,
            source,
            private_parent,
            request,
            snapshot,
            metadata_request: Some(metadata_request),
            candidate,
        }
    }

    pub fn request(&self) -> &LowerRequest {
        &self.request
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn metadata_request(&mut self) -> MetadataRequest {
        self.metadata_request.take().unwrap()
    }

    pub fn candidate(&self) -> &InjectedCandidate {
        &self.candidate
    }

    pub fn private_parent(&self) -> &Path {
        &self.private_parent
    }

    pub fn toolchain_root(&self) -> &Path {
        self.candidate.toolchain_root()
    }

    pub fn expected(&self) -> &ExpectedManifestSelection {
        self.metadata_request.as_ref().unwrap().expected()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        unseal_tree(&self.root);
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn arguments(
    source: &Path,
    toolchain: &Path,
    driver: &Path,
    driver_sha256: &str,
    frontend_sha256: &str,
    distribution_sha256: &str,
) -> Vec<OsString> {
    [
        "lower".to_owned(),
        source.to_str().unwrap().to_owned(),
        "--manifest-path".to_owned(),
        "Cargo.toml".to_owned(),
        "--package".to_owned(),
        "vector".to_owned(),
        "--semantic-profile".to_owned(),
        SEMANTIC_PROFILE.to_owned(),
        "--target".to_owned(),
        "x86_64-unknown-linux-gnu".to_owned(),
        "--function".to_owned(),
        "vector::identity".to_owned(),
        "--frontend-bundle-id".to_owned(),
        FRONTEND_BUNDLE_ID.to_owned(),
        "--frontend-sha256".to_owned(),
        frontend_sha256.to_owned(),
        "--release-registry-id".to_owned(),
        "mpk.release.registry.v0".to_owned(),
        "--release-registry-sha256".to_owned(),
        "5555555555555555555555555555555555555555555555555555555555555555".to_owned(),
        "--toolchain-bundle-id".to_owned(),
        TOOLCHAIN_BUNDLE_ID.to_owned(),
        "--toolchain-root".to_owned(),
        toolchain.to_str().unwrap().to_owned(),
        "--toolchain-distribution-sha256".to_owned(),
        distribution_sha256.to_owned(),
        "--driver".to_owned(),
        driver.to_str().unwrap().to_owned(),
        "--driver-sha256".to_owned(),
        driver_sha256.to_owned(),
        "--contract".to_owned(),
        "contracts/vector.json".to_owned(),
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}

fn materialize_inventory(root: &Path, files: &[(&str, &[u8], bool)]) -> Vec<InventoryFile> {
    let mut inventory = Vec::new();
    for (relative, bytes, executable) in files {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        fs::set_permissions(
            &path,
            fs::Permissions::from_mode(if *executable { 0o555 } else { 0o444 }),
        )
        .unwrap();
        inventory.push(
            InventoryFile::new(
                *relative,
                *executable,
                bytes.len() as u64,
                hex(&digest(bytes)),
            )
            .unwrap(),
        );
    }
    inventory.sort_by(|left, right| left.path().as_bytes().cmp(right.path().as_bytes()));
    inventory
}

fn seal_tree(root: &Path) {
    let mut entries = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if fs::symlink_metadata(&path).unwrap().is_dir() {
            seal_tree(&path);
        }
    }
    fs::set_permissions(root, fs::Permissions::from_mode(0o555)).unwrap();
}

fn unseal_tree(root: &Path) {
    if !root.exists() {
        return;
    }
    let metadata = fs::symlink_metadata(root).unwrap();
    if metadata.is_dir() {
        fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
        for entry in fs::read_dir(root).unwrap() {
            unseal_tree(&entry.unwrap().path());
        }
    } else {
        fs::set_permissions(root, fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedCall {
    pub invocation_id: u64,
    pub kind: CargoInvocationKind,
    pub executable: String,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub snapshot_root: PathBuf,
    pub frontend_root: PathBuf,
    pub toolchain_root: PathBuf,
    pub writable: BTreeMap<String, PathBuf>,
    pub limits: SandboxLimits,
}

#[derive(Default)]
pub struct RecordingExecutor {
    pub calls: Vec<RecordedCall>,
    responses: VecDeque<Result<ProcessOutput, SandboxError>>,
}

impl RecordingExecutor {
    pub fn with_responses(
        responses: impl IntoIterator<Item = Result<ProcessOutput, SandboxError>>,
    ) -> Self {
        Self {
            calls: Vec::new(),
            responses: responses.into_iter().collect(),
        }
    }
}

impl SandboxExecutor for RecordingExecutor {
    fn execute(
        &mut self,
        context: &SandboxContext<'_>,
        invocation: &CargoInvocation,
    ) -> Result<ProcessOutput, SandboxError> {
        let writable = [
            HOME_ROOT,
            "/mpk/cargo-home",
            TEMP_ROOT,
            TARGET_ROOT,
            DRIVER_OUTPUT_ROOT,
        ]
        .into_iter()
        .map(|path| {
            (
                path.to_owned(),
                context.writable_host_path(path).unwrap().to_path_buf(),
            )
        })
        .collect();
        self.calls.push(RecordedCall {
            invocation_id: context.invocation_id(),
            kind: invocation.kind(),
            executable: invocation.executable().to_owned(),
            arguments: invocation.arguments().to_vec(),
            environment: context.environment().entries().clone(),
            snapshot_root: context.snapshot_root().to_path_buf(),
            frontend_root: context.candidate().frontend_root().to_path_buf(),
            toolchain_root: context.candidate().toolchain_root().to_path_buf(),
            writable,
            limits: context.limits(),
        });
        self.responses
            .pop_front()
            .unwrap_or(Err(SandboxError::Spawn))
    }
}

pub fn metadata_json() -> Vec<u8> {
    format!(
        concat!(
            "{{\"metadata\":null,\"packages\":[{{",
            "\"authors\":[],\"categories\":[],\"default_run\":null,",
            "\"dependencies\":[],\"description\":null,\"documentation\":null,",
            "\"edition\":\"2021\",\"features\":{{\"default\":[]}},",
            "\"homepage\":null,\"id\":\"path+file:///mpk/input#vector@0.1.0\",",
            "\"keywords\":[],\"license\":null,\"license_file\":null,",
            "\"links\":null,\"manifest_path\":\"/mpk/input/Cargo.toml\",",
            "\"metadata\":null,\"name\":\"vector\",\"publish\":null,",
            "\"readme\":null,\"repository\":null,\"rust_version\":null,",
            "\"source\":null,\"targets\":[{}],\"version\":\"0.1.0\"}}],",
            "\"resolve\":null,\"target_directory\":\"/mpk/target\",\"version\":1,",
            "\"workspace_default_members\":[\"path+file:///mpk/input#vector@0.1.0\"],",
            "\"workspace_members\":[\"path+file:///mpk/input#vector@0.1.0\"],",
            "\"workspace_root\":\"/mpk/input\"}}"
        ),
        target_json()
    )
    .into_bytes()
}

pub fn target_json() -> String {
    concat!(
        "{\"crate_types\":[\"lib\"],\"doc\":true,\"doctest\":true,",
        "\"edition\":\"2021\",\"kind\":[\"lib\"],\"name\":\"vector\",",
        "\"src_path\":\"/mpk/input/src/lib.rs\",\"test\":true}"
    )
    .to_owned()
}

pub fn successful_check_stream() -> Vec<u8> {
    format!(
        concat!(
            "{{\"executable\":null,\"features\":[],",
            "\"filenames\":[],",
            "\"fresh\":false,\"manifest_path\":\"/mpk/input/Cargo.toml\",",
            "\"package_id\":\"path+file:///mpk/input#vector@0.1.0\",",
            "\"profile\":{{\"debug_assertions\":true,\"debuginfo\":2,",
            "\"opt_level\":\"0\",\"overflow_checks\":true,\"test\":false}},",
            "\"reason\":\"compiler-artifact\",\"target\":{}}}\n",
            "{{\"reason\":\"build-finished\",\"success\":true}}\n"
        ),
        target_json()
    )
    .into_bytes()
}

pub fn failed_check_stream() -> Vec<u8> {
    failed_check_stream_with_codes(&["E0308"])
}

pub fn failed_check_stream_with_codes(codes: &[&str]) -> Vec<u8> {
    let mut stream = String::new();
    for code in codes {
        stream.push_str(&format!(
            concat!(
                "{{\"manifest_path\":\"/mpk/input/Cargo.toml\",",
                "\"message\":{{\"$message_type\":\"diagnostic\",",
                "\"code\":{{\"code\":\"{}\",\"explanation\":null}},\"level\":\"error\",",
                "\"message\":\"classified compiler error\",\"rendered\":null,",
                "\"spans\":[],\"children\":[]}},",
                "\"package_id\":\"path+file:///mpk/input#vector@0.1.0\",",
                "\"reason\":\"compiler-message\",\"target\":{}}}\n"
            ),
            code,
            target_json()
        ));
    }
    stream.push_str("{\"reason\":\"build-finished\",\"success\":false}\n");
    stream.into_bytes()
}

pub fn failed_process_output() -> ProcessOutput {
    ProcessOutput {
        exit_code: Some(101),
        signaled: false,
        stdout: failed_check_stream(),
        stderr_observed_bytes: 128,
        stdout_limit_exceeded: false,
        stderr_limit_exceeded: false,
    }
}

pub fn frozen_environment() -> BTreeMap<String, String> {
    EvidenceEnvironment::frozen().entries().clone()
}
