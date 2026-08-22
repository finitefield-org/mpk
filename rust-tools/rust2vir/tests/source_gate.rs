use rust2vir_internal::driver_protocol::DriverInputIdentity;
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceLoaderCode, SourceLoaderStatus};
use rust2vir_internal::sha256::{digest, hex};
use rust2vir_internal::source_gate::{
    validate_source, SourceGateCode, SourceGateStatus, SourceRole,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rust2vir-source-gate-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
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

fn assert_gate(source: &str, role: SourceRole, expected: SourceGateCode) {
    let error = validate_source(source.as_bytes(), role).unwrap_err();
    assert_eq!(error.code, expected);
}

fn inventory(path: &str, bytes: &[u8]) -> DriverInputIdentity {
    DriverInputIdentity {
        kind: "source".to_owned(),
        normalized_path: path.to_owned(),
        size_bytes: bytes.len() as u64,
        sha256: hex(&digest(bytes)),
    }
}

#[test]
fn accepted_inert_attributes_comments_and_literals_do_not_trigger_the_gate() {
    let root = br##"#![no_std]
#![doc = "crate docs"]
//! inner docs mentioning #[cfg(unix)] and a!()
#[doc = r#"item docs"#]
pub fn f(value: u8) -> u8 {
    let text = "macro_rules! m { () => {} }";
    let _character = '#';
    let _ = text;
    value
}
"##;
    assert_eq!(validate_source(root, SourceRole::CrateRoot), Ok(()));
    assert_eq!(
        validate_source(
            b"/// docs\npub fn child(x: u8) -> u8 { x }\n",
            SourceRole::Module
        ),
        Ok(())
    );
}

#[test]
fn well_formed_later_subset_forms_are_not_misclassified_as_parse_errors() {
    let source = b"struct S { value: u8 }\n\
impl S {\n\
    fn method<'a>(&'a mut self, callback: fn(u8, u8) -> u8) -> u8 {\n\
        callback(self.value, 1)\n\
    }\n\
}\n";
    assert_eq!(validate_source(source, SourceRole::CrateRoot), Ok(()));
}

#[test]
fn every_expansion_affecting_family_has_the_frozen_source_code() {
    let cases = [
        (
            "#[cfg(unix)] pub fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetCfg,
        ),
        (
            "#[cfg_attr(unix, inline)] pub fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetCfg,
        ),
        (
            "#[path = \"other.rs\"] mod other;",
            SourceGateCode::SubsetPath,
        ),
        (
            "#[derive(Clone)] struct S { value: u8 }",
            SourceGateCode::SubsetAttribute,
        ),
        (
            "macro_rules! m {()=>{1}} pub fn f(_:u8)->u8{m!()}",
            SourceGateCode::SubsetMacro,
        ),
        (
            "macro m() { 1 } pub fn f(_:u8)->u8{m!()}",
            SourceGateCode::SubsetMacro,
        ),
        (
            "pub fn f(_:u8)->u8{include!(\"fragment.rs\")}",
            SourceGateCode::SubsetMacro,
        ),
        (
            "#[doc = include_str!(\"README.md\")] pub fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetMacro,
        ),
        (
            "#[allow(unused_variables)] pub fn f(x:u8)->u8{0}",
            SourceGateCode::SubsetAttribute,
        ),
        (
            "use core::cmp; pub fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetImport,
        ),
        (
            "extern crate core; pub fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetImport,
        ),
        (
            "pub(crate) fn f(x:u8)->u8{x}",
            SourceGateCode::SubsetVisibility,
        ),
    ];
    for (source, code) in cases {
        assert_gate(source, SourceRole::CrateRoot, code);
        assert_eq!(code.status(), SourceGateStatus::Rejected);
        assert_eq!(code.phase(), "source");
    }
}

#[test]
fn parse_failure_precedes_subset_findings_and_source_findings_use_table_order() {
    assert_gate(
        "macro_rules! m {()=>{1}} pub fn f( -> u8 { m!() }",
        SourceRole::CrateRoot,
        SourceGateCode::SourceParse,
    );
    assert_gate(
        "macro_rules! m {()=>{1}} pub fn f(x u8) -> u8 { m!() }",
        SourceRole::CrateRoot,
        SourceGateCode::SourceParse,
    );
    assert_gate(
        "pub fn f(x:) -> u8 { x }",
        SourceRole::CrateRoot,
        SourceGateCode::SourceParse,
    );
    assert_eq!(
        SourceGateCode::SourceParse.status(),
        SourceGateStatus::SourceError
    );
    assert_gate(
        "m!(); #[cfg(unix)] pub fn f(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetCfg,
    );
    assert_gate(
        "#[inline] use core::cmp; pub fn f(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetAttribute,
    );
}

#[test]
fn no_std_is_exactly_one_root_only_attribute_and_identifiers_are_canonical() {
    assert_gate(
        "#![no_std]\npub fn f(x:u8)->u8{x}",
        SourceRole::Module,
        SourceGateCode::SubsetAttribute,
    );
    assert_gate(
        "#![no_std = \"yes\"]\npub fn f(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetAttribute,
    );
    assert_gate(
        "mod nested { #![no_std] pub fn f(x:u8)->u8{x} }",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetAttribute,
    );
    assert_gate(
        "pub fn first(x:u8)->u8{x}\n#![no_std]",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetAttribute,
    );
    assert_gate(
        "#![no_std]\n#![no_std]\npub fn f(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetAttribute,
    );
    assert_gate(
        "pub fn r#f(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetIdentifier,
    );
    assert_gate(
        "pub fn café(x:u8)->u8{x}",
        SourceRole::CrateRoot,
        SourceGateCode::SubsetIdentifier,
    );
    assert_eq!(SourceGateCode::SubsetIdentifier.phase(), "subset");
}

#[test]
fn loader_returns_only_preflight_inventory_from_retained_bytes() {
    let root = TestRoot::new();
    let root_bytes = b"mod child;\npub fn f(x:u8)->u8{child::f(x)}\n";
    let child_bytes = b"pub fn f(x:u8)->u8{x}\n";
    root.write("src/lib.rs", root_bytes);
    root.write("src/child.rs", child_bytes);
    let expected = [
        inventory("src/lib.rs", root_bytes),
        inventory("src/child.rs", child_bytes),
    ];
    let loader = SnapshotFileLoader::open(root.path(), "src/lib.rs", &expected).unwrap();

    root.write("src/lib.rs", b"#[cfg(unix)] pub fn replaced() {}\n");
    root.write("src/child.rs", b"compile_error!(\"changed\");\n");
    assert_eq!(
        loader.read_utf8(Path::new("src/lib.rs")).unwrap(),
        String::from_utf8_lossy(root_bytes)
    );
    assert_eq!(
        loader.read_utf8(Path::new("src/child.rs")).unwrap(),
        String::from_utf8_lossy(child_bytes)
    );
    loader
        .validate_root_ast(Path::new("src/lib.rs"), root_bytes)
        .unwrap();
    assert_eq!(
        loader.observed_paths(),
        ["src/child.rs".to_owned(), "src/lib.rs".to_owned()]
    );
    assert_eq!(loader.verify_inventory(), Ok(()));
}

#[test]
fn loader_refuses_external_synthetic_binary_and_unsnapshotted_reads() {
    let root = TestRoot::new();
    let bytes = b"pub fn f(x:u8)->u8{x}\n";
    root.write("src/lib.rs", bytes);
    root.write("src/undiscovered.rs", b"pub fn hidden() {}\n");
    let loader =
        SnapshotFileLoader::open(root.path(), "src/lib.rs", &[inventory("src/lib.rs", bytes)])
            .unwrap();

    assert!(loader.file_exists(Path::new("src/lib.rs")));
    assert!(!loader.file_exists(Path::new("src/undiscovered.rs")));
    for path in ["src/undiscovered.rs", "<synthetic>", "/tmp/outside.rs"] {
        let error = loader.read_utf8(Path::new(path)).unwrap_err();
        assert_eq!(error.code, SourceLoaderCode::FrontendSourceInventory);
    }
    assert_eq!(
        loader
            .read_binary(Path::new("src/lib.rs"))
            .unwrap_err()
            .code,
        SourceLoaderCode::FrontendSourceInventory
    );
}

#[test]
fn loader_gate_runs_before_return_and_inventory_mismatch_is_frontend_error() {
    let rejected = TestRoot::new();
    let cfg = b"#[cfg(unix)] pub fn f(x:u8)->u8{x}\n";
    rejected.write("src/lib.rs", cfg);
    let error = SnapshotFileLoader::open(
        rejected.path(),
        "src/lib.rs",
        &[inventory("src/lib.rs", cfg)],
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        SourceLoaderCode::Gate(SourceGateCode::SubsetCfg)
    );
    assert_eq!(error.code.status(), SourceLoaderStatus::Rejected);

    let valid = TestRoot::new();
    let root_bytes = b"mod child;\npub fn f(x:u8)->u8{x}\n";
    let child_bytes = b"pub fn child() {}\n";
    valid.write("src/lib.rs", root_bytes);
    valid.write("src/child.rs", child_bytes);
    let loader = SnapshotFileLoader::open(
        valid.path(),
        "src/lib.rs",
        &[
            inventory("src/lib.rs", root_bytes),
            inventory("src/child.rs", child_bytes),
        ],
    )
    .unwrap();
    loader.read_utf8(Path::new("src/lib.rs")).unwrap();
    loader
        .validate_root_ast(Path::new("src/lib.rs"), root_bytes)
        .unwrap();
    let error = loader.verify_inventory().unwrap_err();
    assert_eq!(error.code, SourceLoaderCode::FrontendSourceInventory);
    assert_eq!(error.code.status(), SourceLoaderStatus::FrontendError);
}

#[test]
fn refused_compiler_read_poisoning_survives_complete_later_inventory_reads() {
    let root = TestRoot::new();
    let bytes = b"pub fn f(x:u8)->u8{x}\n";
    root.write("src/lib.rs", bytes);
    let loader =
        SnapshotFileLoader::open(root.path(), "src/lib.rs", &[inventory("src/lib.rs", bytes)])
            .unwrap();
    assert!(loader.read_file(Path::new("src/unexpected.rs")).is_err());
    assert_eq!(
        loader.failure().unwrap().code,
        SourceLoaderCode::FrontendSourceInventory
    );
    loader.read_file(Path::new("src/lib.rs")).unwrap();
    loader
        .validate_root_ast(Path::new("src/lib.rs"), bytes)
        .unwrap();
    assert_eq!(
        loader.verify_inventory().unwrap_err().code,
        SourceLoaderCode::FrontendSourceInventory
    );
}

#[test]
fn root_callback_reapplies_the_same_gate_and_requires_exact_snapshot_bytes() {
    let root = TestRoot::new();
    let bytes = b"pub fn f(x:u8)->u8{x}\n";
    root.write("src/lib.rs", bytes);
    let loader =
        SnapshotFileLoader::open(root.path(), "src/lib.rs", &[inventory("src/lib.rs", bytes)])
            .unwrap();
    assert_eq!(
        loader
            .validate_root_ast(Path::new("src/lib.rs"), b"pub fn changed() {}\n")
            .unwrap_err()
            .code,
        SourceLoaderCode::FrontendSourceInventory
    );
    loader
        .validate_root_ast(Path::new("src/lib.rs"), bytes)
        .unwrap();
    assert_eq!(
        loader
            .validate_root_ast(Path::new("src/lib.rs"), bytes)
            .unwrap_err()
            .code,
        SourceLoaderCode::FrontendSourceInventory
    );
}

#[test]
fn snapshot_size_digest_and_root_disagreement_never_widen_the_scan() {
    let root = TestRoot::new();
    let bytes = b"pub fn f(x:u8)->u8{x}\n";
    root.write("src/lib.rs", bytes);
    let mut wrong_size = inventory("src/lib.rs", bytes);
    wrong_size.size_bytes += 1;
    assert_eq!(
        SnapshotFileLoader::open(root.path(), "src/lib.rs", &[wrong_size])
            .unwrap_err()
            .code,
        SourceLoaderCode::FrontendSourceInventory
    );
    let mut wrong_digest = inventory("src/lib.rs", bytes);
    wrong_digest.sha256 = "00".repeat(32);
    assert_eq!(
        SnapshotFileLoader::open(root.path(), "src/lib.rs", &[wrong_digest])
            .unwrap_err()
            .code,
        SourceLoaderCode::FrontendSourceInventory
    );
    assert_eq!(
        SnapshotFileLoader::open(
            root.path(),
            "src/missing.rs",
            &[inventory("src/lib.rs", bytes)]
        )
        .unwrap_err()
        .code,
        SourceLoaderCode::FrontendSourceInventory
    );
}
