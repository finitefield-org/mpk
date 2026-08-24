#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;
#[path = "support/rustc_harness.rs"]
mod rustc_harness;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rust2vir_internal::environment::EvidenceEnvironment;
use rust2vir_internal::json;
use rust2vir_internal::sha256::{digest, hex};
use rustc_driver_adapter::MirLowering;

const SOURCE: &[u8] = br#"pub struct Pair {
    pub left: u8,
    pub right: u8,
}

pub fn differential(
    values: [u8; 4],
    index: usize,
    left: u8,
    right: u8,
    guard: bool,
) -> u8 {
    let sum = left + right;
    let pair = Pair { left: sum, right };
    if guard { pair.left } else { values[index] }
}
"#;
const SENTINEL: &str = "MPK_T04_HOSTILE_SENTINEL_7fcd8921";
const CHILD_MODE_ENV: &str = "MPK_TEST_T04_DIFFERENTIAL_CHILD";
const CHILD_OUTPUT_ENV: &str = "MPK_TEST_T04_DIFFERENTIAL_OUTPUT";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Eq, PartialEq)]
struct CanonicalArtifacts {
    lowering: Vec<u8>,
    source_map: Vec<u8>,
    lowering_sha256: String,
    source_map_sha256: String,
}

#[test]
fn clean_trees_and_hostile_ambient_state_produce_identical_path_clean_artifacts() {
    let contract = tautology_contract("vector::differential");
    let first = lower(&contract);
    let second = lower(&contract);
    assert_eq!(first, second, "two independent clean lowering trees");

    let frozen_environment = EvidenceEnvironment::frozen();
    frozen_environment
        .validate()
        .expect("frozen child environment");
    let hostile = HostileWorkspace::new();
    let under_hostile_state = run_hostile_child(&hostile);
    assert_eq!(
        under_hostile_state, first,
        "hostile environment and cwd changed canonical lowering"
    );

    for bytes in [&first.lowering, &first.source_map] {
        assert_path_clean(bytes, &hostile.root);
    }
}

#[test]
#[ignore = "invoked in an isolated child process by the differential test"]
fn hostile_child_lowering() {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(SENTINEL) {
        return;
    }
    let output =
        PathBuf::from(std::env::var_os(CHILD_OUTPUT_ENV).expect("hostile child output directory"));
    let artifacts = lower(&tautology_contract("vector::differential"));
    let frozen_environment = EvidenceEnvironment::frozen();
    frozen_environment
        .validate()
        .expect("hostile child frozen environment");
    for value in frozen_environment.entries().values() {
        assert!(!value.contains(SENTINEL));
        assert!(!value.contains(output.to_string_lossy().as_ref()));
    }
    write_child_artifacts(&output, &artifacts);
}

fn lower(contract: &[u8]) -> CanonicalArtifacts {
    let lowering = rustc_harness::lower(
        SOURCE,
        "vector::differential",
        &[("contracts/differential.json", contract)],
    )
    .expect("lower differential fixture");
    canonical_artifacts(lowering)
}

fn canonical_artifacts(lowering: MirLowering) -> CanonicalArtifacts {
    let source_map = json::canonical(&lowering.raw_source_map).expect("canonical source map");
    let lowering = json::canonical(&lowering.raw_lowering).expect("canonical lowering");
    CanonicalArtifacts {
        lowering_sha256: hex(&digest(&lowering)),
        source_map_sha256: hex(&digest(&source_map)),
        lowering,
        source_map,
    }
}

fn run_hostile_child(workspace: &HostileWorkspace) -> CanonicalArtifacts {
    let output = Command::new(std::env::current_exe().expect("differential test executable"))
        .args([
            "--exact",
            "hostile_child_lowering",
            "--ignored",
            "--nocapture",
        ])
        .current_dir(workspace.root.join("work"))
        .env(CHILD_MODE_ENV, SENTINEL)
        .env(CHILD_OUTPUT_ENV, workspace.root.join("output"))
        .env("ALL_PROXY", "http://hostile.example.invalid/all")
        .env("CARGO_BUILD_TARGET", "hostile-target.json")
        .env("CARGO_ENCODED_RUSTFLAGS", "--cfg\u{1f}hostile_sentinel")
        .env("CARGO_HOME", workspace.root.join("cargo-home"))
        .env("CARGO_HTTP_PROXY", "http://hostile.example.invalid/cargo")
        .env("CARGO_NET_OFFLINE", "false")
        .env("CARGO_PROFILE_DEV_OVERFLOW_CHECKS", "false")
        .env(
            "CARGO_REGISTRIES_CRATES_IO_INDEX",
            "https://hostile.example.invalid/index",
        )
        .env("CARGO_REGISTRY_TOKEN", SENTINEL)
        .env("CARGO_TARGET_DIR", workspace.root.join("hostile-target"))
        .env("GIT_ASKPASS", SENTINEL)
        .env("HOME", workspace.root.join("home"))
        .env("HOSTNAME", "hostile.example.invalid")
        .env("HTTPS_PROXY", "http://hostile.example.invalid/https")
        .env("HTTP_PROXY", "http://hostile.example.invalid/http")
        .env("LANG", "ja_JP.UTF-8")
        .env("LC_ALL", "tr_TR.UTF-8")
        .env("MPK_ENV_SENTINEL", SENTINEL)
        .env("RUSTC", "/hostile/toolchain/bin/rustc")
        .env("RUSTC_BOOTSTRAP", "1")
        .env("RUSTC_WRAPPER", "/hostile/bin/rustc-wrapper")
        .env("RUSTC_WORKSPACE_WRAPPER", "/hostile/bin/workspace-wrapper")
        .env("RUSTFLAGS", "--cfg hostile_sentinel")
        .env("RUSTUP_HOME", workspace.root.join("rustup-home"))
        .env("RUSTUP_TOOLCHAIN", "hostile-toolchain")
        .env("SOURCE_DATE_EPOCH", "4102444799")
        .env("TEMP", workspace.root.join("temp"))
        .env("TERM", "xterm-hostile")
        .env("TMP", workspace.root.join("temp"))
        .env("TMPDIR", workspace.root.join("temp"))
        .env("TZ", "Asia/Tokyo")
        .output()
        .expect("run hostile differential child");
    assert!(
        output.status.success(),
        "hostile child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    read_child_artifacts(&workspace.root.join("output"))
}

fn write_child_artifacts(output: &Path, artifacts: &CanonicalArtifacts) {
    fs::write(output.join("lowering.json"), &artifacts.lowering).expect("write hostile lowering");
    fs::write(output.join("source-map.json"), &artifacts.source_map)
        .expect("write hostile source map");
    fs::write(
        output.join("lowering.sha256"),
        artifacts.lowering_sha256.as_bytes(),
    )
    .expect("write hostile lowering hash");
    fs::write(
        output.join("source-map.sha256"),
        artifacts.source_map_sha256.as_bytes(),
    )
    .expect("write hostile source-map hash");
}

fn read_child_artifacts(output: &Path) -> CanonicalArtifacts {
    let lowering = fs::read(output.join("lowering.json")).expect("read hostile lowering");
    let source_map = fs::read(output.join("source-map.json")).expect("read hostile source map");
    let artifacts = CanonicalArtifacts {
        lowering_sha256: hex(&digest(&lowering)),
        source_map_sha256: hex(&digest(&source_map)),
        lowering,
        source_map,
    };
    assert_eq!(
        fs::read(output.join("lowering.sha256")).expect("read hostile lowering hash"),
        artifacts.lowering_sha256.as_bytes()
    );
    assert_eq!(
        fs::read(output.join("source-map.sha256")).expect("read hostile source-map hash"),
        artifacts.source_map_sha256.as_bytes()
    );
    artifacts
}

fn tautology_contract(function: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"mpk.rust.contract.v0\",\"semantic_profile\":\"mpk.rust.checked.v0\",\"target_pointer_width\":64,\"function\":\"{function}\",\"requires\":[],\"ensures\":[{{\"op\":\"eq\",\"args\":[{{\"result\":0}},{{\"result\":0}}]}}],\"modifies\":[],\"panic\":\"forbidden\",\"termination\":\"total\",\"loops\":[]}}"
    )
    .into_bytes()
}

fn assert_path_clean(bytes: &[u8], hostile_root: &Path) {
    let text = std::str::from_utf8(bytes).expect("canonical artifact UTF-8");
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .to_string_lossy()
        .into_owned();
    let hostile_root = hostile_root.to_string_lossy().into_owned();
    for forbidden in [
        workspace.as_str(),
        hostile_root.as_str(),
        "/root/",
        "/tmp/",
        "/mpk/input",
        "/mpk/toolchain",
        SENTINEL,
        "hostile.example.invalid",
        "Asia/Tokyo",
        "2099-12-31T23:59:59Z",
        "\"timestamp\"",
        "\"generated_at\"",
        "\"hostname\"",
    ] {
        assert!(!text.contains(forbidden), "artifact leaked {forbidden:?}");
    }
}

struct HostileWorkspace {
    root: PathBuf,
}

impl HostileWorkspace {
    fn new() -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust2vir-differential-hostile-{}-{serial}",
            std::process::id()
        ));
        for child in [
            "cargo-home",
            "home",
            "output",
            "rustup-home",
            "temp",
            "work",
        ] {
            fs::create_dir_all(root.join(child)).expect("create hostile directory");
        }
        Self { root }
    }
}

impl Drop for HostileWorkspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove hostile test root");
    }
}
