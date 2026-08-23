use rust2vir_internal::cli::{parse_lower_args, LowerRequest, SEMANTIC_PROFILE};
use rust2vir_internal::manifest::{self, ManifestCode, ManifestStatus, ValidatedManifest};
use rust2vir_internal::metadata_request::{MetadataRequest, SNAPSHOT_MANIFEST_PATH};
use rust2vir_internal::module_closure;
use rust2vir_internal::preflight;
use rust2vir_internal::snapshot::Snapshot;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALID_MANIFEST: &[u8] = include_bytes!("../testdata/cargo-preflight/valid/Cargo.toml");
const VALID_LOCKFILE: &[u8] = include_bytes!("../testdata/cargo-preflight/valid/Cargo.lock");
const MINIMAL_MANIFEST: &str = "[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n";
const MINIMAL_LOCKFILE: &str = "version=4\n\n[[package]]\nname='vector'\nversion='0.1.0'\n";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rust2vir-manifest-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn package(manifest: &[u8], lockfile: &[u8]) -> Self {
        let root = Self::new("source");
        fs::create_dir(root.path.join("contracts")).unwrap();
        fs::create_dir(root.path.join("src")).unwrap();
        fs::write(root.path.join("Cargo.toml"), manifest).unwrap();
        fs::write(root.path.join("Cargo.lock"), lockfile).unwrap();
        fs::write(root.path.join("contracts/vector.json"), b"{}\n").unwrap();
        fs::write(
            root.path.join("src/lib.rs"),
            b"pub fn identity(value: u8) -> u8 { value }\n",
        )
        .unwrap();
        root
    }

    fn minimal() -> Self {
        Self::package(MINIMAL_MANIFEST.as_bytes(), MINIMAL_LOCKFILE.as_bytes())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).unwrap();
        }
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

fn validate(root: &TestRoot) -> Result<ValidatedManifest, manifest::ManifestError> {
    let request = request(root.path());
    manifest::validate(&request, preflight::run(&request).unwrap())
}

fn assert_manifest_error(manifest: &str, lockfile: &str, code: ManifestCode) {
    let root = TestRoot::package(manifest.as_bytes(), lockfile.as_bytes());
    let error = validate(&root).unwrap_err();
    assert_eq!(error.code, code, "manifest:\n{manifest}\nlock:\n{lockfile}");
    assert_eq!(error.code.status(), code.status());
}

fn with_manifest_suffix(suffix: &str) -> String {
    format!("{MINIMAL_MANIFEST}{suffix}")
}

#[test]
fn valid_closed_manifest_produces_only_the_expected_selection_and_metadata_request() {
    let root = TestRoot::package(VALID_MANIFEST, VALID_LOCKFILE);
    let validated = validate(&root).unwrap();
    let expected = validated.selection();
    assert_eq!(expected.package_name(), "vector");
    assert_eq!(expected.package_version(), "1.2.3-alpha.1+fixture");
    assert_eq!(expected.crate_name(), "vector");
    assert_eq!(expected.library_path().as_str(), "src/lib.rs");
    assert_eq!(expected.edition(), "2021");
    assert_eq!(expected.kind(), "lib");

    let (closure, expected) = module_closure::discover(validated).unwrap();
    assert!(closure
        .inputs
        .iter()
        .all(|input| input.normalized_path.as_str() != "tools/unselected.rs"));
    let parent = TestRoot::new("snapshot-parent");
    let snapshot = Snapshot::create(parent.path(), &closure).unwrap();
    let metadata = MetadataRequest::for_snapshot(&snapshot, expected).unwrap();
    assert_eq!(
        metadata.arguments(),
        [
            "metadata",
            "--manifest-path",
            SNAPSHOT_MANIFEST_PATH,
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
            "--no-default-features",
            "--color",
            "never",
        ]
    );
    assert!(metadata
        .arguments()
        .iter()
        .all(|argument| !argument.contains(root.path().to_str().unwrap())));
    assert_eq!(metadata.expected().package_name(), "vector");
}

#[test]
fn cargo_toml_inline_table_semantics_preserve_the_same_closed_selection() {
    let root = TestRoot::package(
        b"package={name='vector',version='0.1.0',edition='2021',publish=false}\n\
          lib={name='vector',path='src/lib.rs'}\n\
          features={default=[]}\n\
          dependencies={}\n",
        MINIMAL_LOCKFILE.as_bytes(),
    );
    let validated = validate(&root).unwrap();
    assert_eq!(validated.selection().package_name(), "vector");
    assert_eq!(validated.selection().library_path().as_str(), "src/lib.rs");
}

#[test]
fn malformed_manifest_or_lockfile_has_source_error_precedence() {
    assert_manifest_error(
        "[package]\nname=\"unterminated\n[workspace]\n",
        MINIMAL_LOCKFILE,
        ManifestCode::SourceManifestParse,
    );
    assert_manifest_error(
        MINIMAL_MANIFEST,
        "version=4\n[[package]]\nname=\"unterminated\n",
        ManifestCode::SourceManifestParse,
    );
    assert_eq!(
        ManifestCode::SourceManifestParse.status(),
        ManifestStatus::SourceError
    );

    let missing_lock = TestRoot::minimal();
    fs::remove_file(missing_lock.path.join("Cargo.lock")).unwrap();
    assert_eq!(
        validate(&missing_lock).unwrap_err().code,
        ManifestCode::Lockfile
    );

    fs::write(
        missing_lock.path.join("Cargo.toml"),
        b"[package]\nname=\"unterminated\n",
    )
    .unwrap();
    assert_eq!(
        validate(&missing_lock).unwrap_err().code,
        ManifestCode::SourceManifestParse
    );
}

#[test]
fn every_workspace_and_dependency_shape_rejects_structurally() {
    for suffix in [
        "[workspace]\nmembers=[]\n",
        "[package.metadata]\nx='y'\n[workspace]\n",
    ] {
        assert_manifest_error(
            &with_manifest_suffix(suffix),
            MINIMAL_LOCKFILE,
            if suffix.contains("package.metadata") {
                ManifestCode::ManifestField
            } else {
                ManifestCode::Workspace
            },
        );
    }
    assert_manifest_error(
        "[package]\nname='vector'\nedition='2021'\nversion.workspace=true\n",
        MINIMAL_LOCKFILE,
        ManifestCode::Workspace,
    );

    for suffix in [
        "[dependencies]\nserde='1'\n",
        "[dev-dependencies]\nhelper={path='helper'}\n",
        "[build-dependencies.helper]\nversion='1'\n",
        "[target.'cfg(unix)'.dependencies]\nserde='1'\n",
    ] {
        assert_manifest_error(
            &with_manifest_suffix(suffix),
            MINIMAL_LOCKFILE,
            ManifestCode::Dependency,
        );
    }
    assert_manifest_error(
        "dependencies={serde='1'}\n[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n",
        MINIMAL_LOCKFILE,
        ManifestCode::Dependency,
    );
}

#[test]
fn features_build_scripts_crate_types_and_unknown_fields_are_closed() {
    for suffix in [
        "[features]\ndefault=['x']\n",
        "[features]\nextra=[]\n",
        "[features]\n",
    ] {
        assert_manifest_error(
            &with_manifest_suffix(suffix),
            MINIMAL_LOCKFILE,
            ManifestCode::Feature,
        );
    }

    assert_manifest_error(
        "[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\nbuild='builder.rs'\n",
        MINIMAL_LOCKFILE,
        ManifestCode::BuildScript,
    );
    for suffix in [
        "[lib]\ncrate-type=['lib']\n",
        "[lib]\nproc-macro=true\n",
        "[[lib]]\nname='second'\npath='src/second.rs'\n",
    ] {
        assert_manifest_error(
            &with_manifest_suffix(suffix),
            MINIMAL_LOCKFILE,
            ManifestCode::Target,
        );
    }
    for suffix in [
        "resolver='2'\n",
        "[profile.release]\nopt-level=3\n",
        "rust-version='1.89'\n",
        "[lib]\ndoctest=false\n",
        "[[bin]]\nname='helper'\npath='src/helper.rs'\nharness=false\n",
    ] {
        assert_manifest_error(
            &with_manifest_suffix(suffix),
            MINIMAL_LOCKFILE,
            ManifestCode::ManifestField,
        );
    }
}

#[test]
fn package_library_path_and_lock_selection_are_exact() {
    for manifest in [
        "[package]\nname='another'\nversion='0.1.0'\nedition='2021'\n",
        "[package]\nname='vector'\nversion='01.0.0'\nedition='2021'\n",
        "[package]\nname='vector'\nversion='0.1.0'\nedition='2024'\n",
        "[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n[lib]\nname='other'\n",
    ] {
        assert_manifest_error(manifest, MINIMAL_LOCKFILE, ManifestCode::Target);
    }
    assert_manifest_error(
        &with_manifest_suffix("[lib]\npath='../outside.rs'\n"),
        MINIMAL_LOCKFILE,
        ManifestCode::PreflightPath,
    );

    for lockfile in [
        "version=3\n[[package]]\nname='vector'\nversion='0.1.0'\n",
        "version=4\n[[package]]\nname='other'\nversion='0.1.0'\n",
        "version=4\n[[package]]\nname='vector'\nversion='0.1.0'\nsource='registry+x'\n",
        "version=4\n[[package]]\nname='vector'\nversion='0.1.0'\n[[package]]\nname='other'\nversion='0.1.0'\n",
    ] {
        assert_manifest_error(MINIMAL_MANIFEST, lockfile, ManifestCode::Lockfile);
    }
}

#[test]
fn implicit_build_rs_rejects_and_no_preflight_path_executes_cargo() {
    let root = TestRoot::minimal();
    fs::write(root.path.join("build.rs"), b"fn main() {}\n").unwrap();
    assert_eq!(validate(&root).unwrap_err().code, ManifestCode::BuildScript);

    fs::remove_file(root.path.join("build.rs")).unwrap();
    let fake_bin = TestRoot::new("fake-bin");
    let marker = fake_bin.path.join("cargo-ran");
    let cargo = fake_bin.path.join("cargo");
    fs::write(
        &cargo,
        b"#!/bin/sh\n: > \"$RUST2VIR_TEST_CARGO_MARKER\"\nexit 99\n",
    )
    .unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(arguments(root.path()))
        .env("PATH", fake_bin.path())
        .env("RUST2VIR_TEST_CARGO_MARKER", &marker)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert!(!marker.exists(), "structural preflight executed Cargo");
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("RUST_TOOLCHAIN_COMPONENT"));
}

#[test]
fn cli_manifest_diagnostics_are_path_free_and_status_exact() {
    let root = TestRoot::package(
        b"[package]\nname=\"unterminated\n",
        MINIMAL_LOCKFILE.as_bytes(),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(arguments(root.path()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stderr.is_empty());
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(body.contains("\"code\":\"RUST_SOURCE_MANIFEST_PARSE\""));
    assert!(body.contains("\"status\":\"source-error\""));
    assert!(!body.contains(root.path().to_str().unwrap()));

    fs::write(
        root.path.join("Cargo.toml"),
        with_manifest_suffix("[dependencies]\nserde='1'\n"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rust2vir"))
        .args(arguments(root.path()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(body.contains("\"code\":\"RUST_PREFLIGHT_DEPENDENCY\""));
    assert!(body.contains("\"status\":\"rejected\""));
}
