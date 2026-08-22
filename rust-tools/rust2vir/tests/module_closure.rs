use rust2vir_internal::cli::{parse_lower_args, LowerRequest, SEMANTIC_PROFILE};
use rust2vir_internal::module_closure::{self, ClosureStatus, ModuleClosure, ModuleClosureCode};
use rust2vir_internal::preflight;
use rust2vir_internal::sha256;
use rust2vir_internal::source_capture::InputKind;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const LOCKFILE: &[u8] = b"version = 4\n\n[[package]]\nname = \"vector\"\nversion = \"0.1.0\"\n";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rust2vir-module-{}-{serial}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn package(manifest: &[u8], library_path: &str, library: &[u8]) -> Self {
        let root = Self::new();
        fs::write(root.path.join("Cargo.toml"), manifest).unwrap();
        fs::write(root.path.join("Cargo.lock"), LOCKFILE).unwrap();
        fs::create_dir(root.path.join("contracts")).unwrap();
        fs::write(root.path.join("contracts/vector.json"), b"{}\n").unwrap();
        let target = root.path.join(library_path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, library).unwrap();
        root
    }

    fn default_package(library: &[u8]) -> Self {
        Self::package(
            b"[package]\nname = \"vector\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            "src/lib.rs",
            library,
        )
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
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
        "--frontend-bundle-id",
        "frontend.rust.rust2vir.v0",
        "--frontend-sha256",
        SHA256,
        "--release-registry-id",
        "mpk.release.registry.v0",
        "--release-registry-sha256",
        SHA256,
        "--toolchain-bundle-id",
        "toolchain.rust.nightly-2025-06-01.v0",
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

fn discover(root: &TestRoot) -> Result<ModuleClosure, module_closure::ModuleClosureError> {
    module_closure::discover(preflight::run(&request(root.path())).unwrap())
}

fn assert_error(root: &TestRoot, code: ModuleClosureCode) {
    let error = discover(root).unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(error.code.status(), code.status());
}

fn padded_module(bytes: usize, has_child: bool) -> Vec<u8> {
    let prefix: &[u8] = if has_child { b"mod m;\n/*" } else { b"/*" };
    let suffix = b"*/\n";
    assert!(bytes >= prefix.len() + suffix.len());
    let mut source = Vec::with_capacity(bytes);
    source.extend_from_slice(prefix);
    source.resize(bytes - suffix.len(), b'x');
    source.extend_from_slice(suffix);
    source
}

fn child_path(parent: &str) -> String {
    format!("{}/m.rs", parent.strip_suffix(".rs").unwrap())
}

#[test]
fn default_rules_capture_only_the_recursive_module_closure() {
    let root = TestRoot::default_package(b"mod first;\nmod inline { mod nested; }\n");
    root.write("src/first.rs", b"mod child;\n");
    root.write("src/first/child.rs", b"pub fn child() {}\n");
    root.write("src/inline/nested.rs", b"pub fn nested() {}\n");
    root.write("src/unrelated.rs", b"this file is deliberately invalid\n");

    let closure = discover(&root).unwrap();
    let paths = closure
        .inputs
        .iter()
        .map(|input| input.normalized_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "Cargo.lock",
            "Cargo.toml",
            "contracts/vector.json",
            "src/first.rs",
            "src/first/child.rs",
            "src/inline/nested.rs",
            "src/lib.rs",
        ]
    );
    assert!(!paths.contains(&"src/unrelated.rs"));
    assert_eq!(closure.source_inputs().count(), 4);
    for input in &closure.inputs {
        assert_eq!(input.sha256, sha256::digest(&input.bytes));
        assert_eq!(input.sha256_hex().len(), 64);
    }
}

#[test]
fn allowlisted_library_path_and_non_mod_file_rules_are_exact() {
    let manifest = b"[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n\
        [lib]\npath = 'library/root.rs'\n";
    let root = TestRoot::package(
        manifest,
        "library/root.rs",
        b"mod child;\nmod inside { mod leaf; }\n",
    );
    root.write("library/root/child.rs", b"pub fn child() {}\n");
    root.write("library/root/inside/leaf.rs", b"pub fn leaf() {}\n");

    let closure = discover(&root).unwrap();
    assert_eq!(closure.library_root.as_str(), "library/root.rs");
    let sources = closure
        .source_inputs()
        .map(|input| input.normalized_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        [
            "library/root.rs",
            "library/root/child.rs",
            "library/root/inside/leaf.rs",
        ]
    );
}

#[test]
fn missing_ambiguous_duplicate_and_identity_cycles_have_exact_source_codes() {
    let missing = TestRoot::default_package(b"mod absent;\n");
    assert_error(&missing, ModuleClosureCode::SourceModuleMissing);

    let ambiguous = TestRoot::default_package(b"mod both;\n");
    ambiguous.write("src/both.rs", b"pub fn one() {}\n");
    ambiguous.write("src/both/mod.rs", b"pub fn two() {}\n");
    assert_error(&ambiguous, ModuleClosureCode::SourceModuleAmbiguous);

    let duplicate = TestRoot::default_package(b"mod repeated;\nmod repeated;\n");
    duplicate.write("src/repeated.rs", b"pub fn value() {}\n");
    assert_error(&duplicate, ModuleClosureCode::SourceModuleDuplicate);

    let cycle = TestRoot::default_package(b"mod alias;\n");
    fs::hard_link(
        cycle.path.join("src/lib.rs"),
        cycle.path.join("src/alias.rs"),
    )
    .unwrap();
    assert_error(&cycle, ModuleClosureCode::SourceModuleCycle);
    assert_eq!(
        ModuleClosureCode::SourceModuleCycle.status(),
        ClosureStatus::SourceError
    );
}

#[test]
fn expansion_forms_reject_before_any_child_is_followed() {
    let cfg = TestRoot::default_package(b"#[cfg(unix)] mod missing;\n");
    assert_error(&cfg, ModuleClosureCode::SubsetCfg);

    let explicit = TestRoot::default_package(b"#[path = \"outside.rs\"] mod missing;\n");
    assert_error(&explicit, ModuleClosureCode::SubsetPath);

    let macro_source =
        TestRoot::default_package(b"macro_rules! modules { () => {} }\nmodules!();\n");
    assert_error(&macro_source, ModuleClosureCode::SubsetMacro);

    let macro_definition = TestRoot::default_package(b"macro modules() { mod hidden; }\n");
    assert_error(&macro_definition, ModuleClosureCode::SubsetMacro);

    let attribute = TestRoot::default_package(b"#[inline] mod missing;\n");
    assert_error(&attribute, ModuleClosureCode::SubsetAttribute);
}

#[test]
fn candidate_links_and_ascii_fold_collisions_are_preflight_rejections() {
    let linked = TestRoot::default_package(b"mod linked;\n");
    let outside = TestRoot::new();
    outside.write("source.rs", b"pub fn outside() {}\n");
    symlink(
        outside.path.join("source.rs"),
        linked.path.join("src/linked.rs"),
    )
    .unwrap();
    assert_error(&linked, ModuleClosureCode::PreflightFileType);

    let collision = TestRoot::default_package(b"mod A;\nmod a;\n");
    collision.write("src/A.rs", b"pub fn upper() {}\n");
    collision.write("src/a.rs", b"pub fn lower() {}\n");
    assert_error(&collision, ModuleClosureCode::PreflightPath);
}

#[test]
fn source_error_is_emitted_with_capture_phase_and_exit_four() {
    let root = TestRoot::default_package(b"mod absent;\n");
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(arguments(root.path()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stderr.is_empty());
    let envelope = String::from_utf8(output.stdout).unwrap();
    assert!(envelope.contains("\"status\":\"source-error\""));
    assert!(envelope.contains("\"phase\":\"capture\""));
    assert!(envelope.contains("RUST_SOURCE_MODULE_MISSING"));
    assert!(!envelope.contains(root.path().to_str().unwrap()));
}

#[test]
fn captured_kinds_include_one_immutable_source_buffer_per_file() {
    let root = TestRoot::default_package(b"pub fn identity(value: u8) -> u8 { value }\n");
    let closure = discover(&root).unwrap();
    let source = closure.source_inputs().next().unwrap();
    assert_eq!(source.kind, InputKind::Source);
    assert_eq!(source.normalized_path.as_str(), "src/lib.rs");
    assert!(!source.has_same_original_identity(&closure.inputs[0]));
}

#[test]
fn source_file_count_and_byte_limits_are_enforced_deterministically() {
    let oversized = TestRoot::default_package(b"mod huge;\n");
    oversized.write("src/huge.rs", &padded_module(1_048_577, false));
    assert_error(&oversized, ModuleClosureCode::LimitInputBytes);

    let count = TestRoot::default_package(b"mod m;\n");
    let mut path = "src/m.rs".to_owned();
    for index in 0..256 {
        let bytes: &[u8] = if index == 255 {
            b"/* end */\n"
        } else {
            b"mod m;\n"
        };
        count.write(&path, bytes);
        path = child_path(&path);
    }
    assert_error(&count, ModuleClosureCode::LimitInputCount);

    let aggregate = TestRoot::default_package(&padded_module(1_048_576, true));
    let mut path = "src/m.rs".to_owned();
    for index in 0..16 {
        aggregate.write(&path, &padded_module(1_048_576, index != 15));
        path = child_path(&path);
    }
    assert_error(&aggregate, ModuleClosureCode::LimitInputBytes);
}
