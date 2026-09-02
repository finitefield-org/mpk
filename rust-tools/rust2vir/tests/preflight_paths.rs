use rust2vir_internal::cli::{parse_lower_args, LowerRequest, SEMANTIC_PROFILE};
use rust2vir_internal::preflight::{self, InputKind, PreflightCode};
use rust2vir_internal::successor::{
    FRONTEND_ID, PROFILE_ENTRY_SHA256, PROFILE_REGISTRY_ID, PROFILE_REGISTRY_SHA256, TOOLCHAIN_ID,
};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const MANIFEST: &[u8] = b"[package]\nname = \"vector\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const LOCKFILE: &[u8] = b"version = 4\n\n[[package]]\nname = \"vector\"\nversion = \"0.1.0\"\n";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rust2vir-preflight-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn valid() -> Self {
        let root = Self::new();
        fs::create_dir(root.path.join("contracts")).unwrap();
        fs::write(root.path.join("Cargo.toml"), MANIFEST).unwrap();
        fs::write(root.path.join("Cargo.lock"), LOCKFILE).unwrap();
        fs::write(root.path.join("contracts/vector.json"), b"{}\n").unwrap();
        root
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}

fn arguments(root: &Path) -> Vec<OsString> {
    let root = root.to_str().unwrap();
    [
        "lower",
        root,
        "--manifest-path",
        "Cargo.toml",
        "--package",
        "vector",
        "--semantic-profile",
        SEMANTIC_PROFILE,
        "--target",
        "x86_64-unknown-linux-gnu",
        "--function",
        "vector::identity",
        "--profile-registry-id",
        PROFILE_REGISTRY_ID,
        "--profile-registry-revision",
        "2",
        "--profile-registry-sha256",
        PROFILE_REGISTRY_SHA256,
        "--profile-entry-sha256",
        PROFILE_ENTRY_SHA256,
        "--frontend-bundle-id",
        FRONTEND_ID,
        "--frontend-sha256",
        SHA256,
        "--release-registry-id",
        "mpk.release.registry.v1",
        "--release-registry-sha256",
        SHA256,
        "--toolchain-bundle-id",
        TOOLCHAIN_ID,
        "--toolchain-root",
        "/mpk/toolchain",
        "--toolchain-distribution-sha256",
        SHA256,
        "--driver",
        "/mpk/frontend/rust2vir-driver",
        "--driver-sha256",
        SHA256,
        "--contract",
        "contracts/vector.json",
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}

fn request(root: &Path) -> LowerRequest {
    parse_lower_args(arguments(root)).unwrap()
}

fn replace_contract(request: &mut LowerRequest, contracts: &[&str]) {
    request.contracts = contracts.iter().map(|value| (*value).to_owned()).collect();
}

fn assert_code(
    result: Result<preflight::StructuralPreflight, preflight::PreflightError>,
    code: PreflightCode,
) {
    assert_eq!(result.unwrap_err().code, code);
}

#[test]
fn valid_structural_inputs_are_captured_in_portable_order() {
    let root = TestRoot::valid();
    let preflight = preflight::run(&request(root.path())).unwrap();
    let paths = preflight
        .inputs
        .iter()
        .map(|input| input.normalized_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, ["Cargo.lock", "Cargo.toml", "contracts/vector.json"]);
    assert_eq!(preflight.inputs[0].kind, InputKind::Lockfile);
    assert_eq!(preflight.inputs[1].bytes.as_ref(), MANIFEST);
    assert_eq!(preflight.inputs[2].kind, InputKind::Contract);
}

#[test]
fn portable_path_failures_and_case_collisions_are_rejected() {
    let root = TestRoot::valid();
    for value in [
        "/tmp/contract.json",
        "../contract.json",
        "contracts\\vector.json",
        "contracts/CON.json",
        "contracts/vector.",
        "contracts/percent%2fescape.json",
    ] {
        let mut selected = request(root.path());
        replace_contract(&mut selected, &[value]);
        assert_code(preflight::run(&selected), PreflightCode::Path);
    }

    fs::write(root.path.join("contracts/Vector.json"), b"{}\n").unwrap();
    let mut selected = request(root.path());
    replace_contract(
        &mut selected,
        &["contracts/vector.json", "contracts/Vector.json"],
    );
    assert_code(preflight::run(&selected), PreflightCode::Path);
}

#[test]
fn path_and_contract_count_limits_precede_filesystem_access() {
    let root = TestRoot::valid();
    let mut selected = request(root.path());
    replace_contract(&mut selected, &[&"a".repeat(1_025)]);
    assert_code(preflight::run(&selected), PreflightCode::LimitPath);

    selected.contracts = (0..129).map(|index| format!("c/{index}.json")).collect();
    assert_code(preflight::run(&selected), PreflightCode::LimitContract);
}

#[test]
fn file_type_precedes_non_limit_path_failures() {
    let root = TestRoot::valid();
    fs::remove_file(root.path.join("Cargo.lock")).unwrap();
    fs::create_dir(root.path.join("Cargo.lock")).unwrap();
    let mut selected = request(root.path());
    replace_contract(&mut selected, &["../outside.json"]);
    assert_code(preflight::run(&selected), PreflightCode::FileType);
}

#[test]
fn required_and_contract_files_must_be_regular_no_follow_inputs() {
    let root = TestRoot::valid();
    fs::remove_file(root.path.join("contracts/vector.json")).unwrap();
    let outside = TestRoot::new();
    fs::write(outside.path.join("secret"), b"must not be read").unwrap();
    symlink(
        outside.path.join("secret"),
        root.path.join("contracts/vector.json"),
    )
    .unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::FileType,
    );

    fs::remove_file(root.path.join("contracts/vector.json")).unwrap();
    fs::create_dir(root.path.join("contracts/vector.json")).unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::FileType,
    );
}

#[test]
fn hard_link_aliases_inside_the_input_set_are_rejected() {
    let root = TestRoot::valid();
    fs::hard_link(
        root.path.join("contracts/vector.json"),
        root.path.join("contracts/alias.json"),
    )
    .unwrap();
    let mut selected = request(root.path());
    replace_contract(
        &mut selected,
        &["contracts/vector.json", "contracts/alias.json"],
    );
    assert_code(preflight::run(&selected), PreflightCode::FileType);
}

#[test]
fn manifest_lock_and_contract_byte_limits_are_checked_before_parsing() {
    let root = TestRoot::valid();
    fs::write(root.path.join("Cargo.toml"), vec![b'x'; 1_048_577]).unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::LimitInputBytes,
    );

    fs::write(root.path.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(
        root.path.join("contracts/vector.json"),
        vec![b'x'; 1_048_577],
    )
    .unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::LimitContract,
    );

    fs::write(
        root.path.join("contracts/vector.json"),
        vec![b'x'; 1_048_576],
    )
    .unwrap();
    for index in 0..8 {
        fs::write(
            root.path.join(format!("contracts/{index}.json")),
            vec![b'x'; 1_048_576],
        )
        .unwrap();
    }
    let mut selected = request(root.path());
    selected.contracts = (0..8)
        .map(|index| format!("contracts/{index}.json"))
        .chain(std::iter::once("contracts/vector.json".to_owned()))
        .collect();
    assert_code(preflight::run(&selected), PreflightCode::LimitContract);
}

#[test]
fn cargo_authority_files_are_rejected_in_fixed_precedence() {
    let root = TestRoot::valid();
    fs::write(root.path.join("Cargo.toml"), b"[workspace]\nmembers = []\n").unwrap();
    fs::create_dir(root.path.join("nested")).unwrap();
    fs::write(root.path.join("nested/Cargo.toml"), MANIFEST).unwrap();
    fs::create_dir(root.path.join(".cargo")).unwrap();
    fs::write(root.path.join(".cargo/config.toml"), b"[build]\n").unwrap();
    fs::write(root.path.join("rust-toolchain.toml"), b"[toolchain]\n").unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::Workspace,
    );

    fs::remove_dir_all(root.path.join("nested")).unwrap();
    assert_code(preflight::run(&request(root.path())), PreflightCode::Config);

    fs::remove_dir_all(root.path.join(".cargo")).unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::ToolchainFile,
    );
}

#[test]
fn nested_manifest_is_workspace_authority_but_directory_symlinks_are_not_followed() {
    let root = TestRoot::valid();
    fs::create_dir(root.path.join("nested")).unwrap();
    fs::write(root.path.join("nested/Cargo.toml"), MANIFEST).unwrap();
    assert_code(
        preflight::run(&request(root.path())),
        PreflightCode::Workspace,
    );

    fs::remove_dir_all(root.path.join("nested")).unwrap();
    let outside = TestRoot::new();
    fs::write(outside.path.join("Cargo.toml"), b"[workspace]\n").unwrap();
    symlink(outside.path(), root.path.join("nested")).unwrap();
    preflight::run(&request(root.path())).unwrap();
}

#[test]
fn a_symlinked_or_unsupported_source_root_is_rejected() {
    let root = TestRoot::valid();
    let parent = TestRoot::new();
    let link = parent.path.join("source");
    symlink(root.path(), &link).unwrap();
    assert_code(preflight::run(&request(&link)), PreflightCode::FileType);

    assert_code(
        preflight::run(&request(Path::new("/proc"))),
        PreflightCode::FileType,
    );
}

#[test]
fn structural_rejection_uses_the_exact_path_free_envelope() {
    let root = TestRoot::valid();
    fs::create_dir(root.path.join(".cargo")).unwrap();
    fs::write(root.path.join(".cargo/config"), b"[build]\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(arguments(root.path()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let expected = concat!(
        "{\"diagnostics\":[],\"phase\":\"capture\",",
        "\"rejected_features\":[{\"code\":\"RUST_PREFLIGHT_CONFIG\",",
        "\"message\":\"Cargo configuration is not permitted\"}],",
        "\"schema\":\"mpk.frontend.cli.v1\",",
        "\"selection\":{\"schema\":\"mpk.selection.rust_function.v0\",",
        "\"value\":{\"crate\":\"vector\",\"function\":\"vector::identity\",",
        "\"kind\":\"lib\",\"package\":\"vector\"}},",
        "\"semantic_context\":{",
        "\"profile_entry_sha256\":\"1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057\",",
        "\"profile_registry\":{\"id\":\"mpk.semantic_profile.registry.v1\",",
        "\"registry_sha256\":\"fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557\",",
        "\"revision\":3,\"schema\":\"mpk.semantic_profile.registry.v1\"},",
        "\"semantic_parameters\":{\"schema\":\"mpk.semantic_parameters.rust_checked.v0\",",
        "\"value\":{\"overflow_mode\":\"checked\",",
        "\"panic_mode\":\"abort\",\"pointer_width\":64,",
        "\"target_id\":\"x86_64-unknown-linux-gnu\"}},",
        "\"semantic_profile\":\"mpk.rust.checked.v0\",",
        "\"source_language\":\"rust\"},\"status\":\"rejected\"}\n"
    );
    assert_eq!(output.stdout, expected.as_bytes());
    let envelope = String::from_utf8(output.stdout).unwrap();
    assert!(!envelope.contains(root.path().to_str().unwrap()));
    assert!(!envelope.contains("/mpk/toolchain"));
    assert!(!envelope.contains("/mpk/frontend/rust2vir-driver"));
    assert!(!envelope.contains("contracts/vector.json"));
}
