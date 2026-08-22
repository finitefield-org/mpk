use rust2vir_internal::cli::{parse_lower_args, LowerRequest, SEMANTIC_PROFILE};
use rust2vir_internal::manifest;
use rust2vir_internal::module_closure::{self, ModuleClosure};
use rust2vir_internal::preflight;
use rust2vir_internal::sha256;
use rust2vir_internal::snapshot::{Snapshot, SnapshotError};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ORIGINAL_SOURCE: &[u8] = b"mod value;\npub fn identity(value: u8) -> u8 { value }\n";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rust2vir-snapshot-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn package() -> Self {
        let root = Self::new("source");
        fs::create_dir(root.path.join("src")).unwrap();
        fs::create_dir(root.path.join("contracts")).unwrap();
        fs::write(
            root.path.join("Cargo.toml"),
            b"[package]\nname='vector'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        fs::write(
            root.path.join("Cargo.lock"),
            b"version = 4\n\n[[package]]\nname='vector'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(root.path.join("contracts/vector.json"), b"{}\n").unwrap();
        fs::write(root.path.join("src/lib.rs"), ORIGINAL_SOURCE).unwrap();
        fs::write(
            root.path.join("src/value.rs"),
            b"pub const VALUE: u8 = 1;\n",
        )
        .unwrap();
        fs::write(
            root.path.join("src/unrelated.rs"),
            b"must never enter the snapshot\n",
        )
        .unwrap();
        root
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.exists() {
            make_tree_writable(&self.path);
            fs::remove_dir_all(&self.path).unwrap();
        }
    }
}

fn make_tree_writable(root: &Path) {
    if let Ok(metadata) = fs::symlink_metadata(root) {
        if metadata.is_dir() {
            fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
            if let Ok(entries) = fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                        make_tree_writable(&path);
                    }
                }
            }
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

fn capture(root: &TestDirectory) -> ModuleClosure {
    let request: LowerRequest = parse_lower_args(arguments(root.path())).unwrap();
    let validated = manifest::validate(&request, preflight::run(&request).unwrap()).unwrap();
    module_closure::discover(validated).unwrap().0
}

#[test]
fn snapshot_bytes_and_hashes_remain_bound_after_original_mutation() {
    let source = TestDirectory::package();
    let parent = TestDirectory::new("parent");
    let closure = capture(&source);
    let captured_source = closure
        .source_inputs()
        .find(|input| input.normalized_path.as_str() == "src/lib.rs")
        .unwrap();
    let captured_bytes = captured_source.bytes.clone();
    let captured_hash = captured_source.sha256;

    fs::write(
        source.path.join("src/lib.rs"),
        b"pub fn identity(_: u8) -> u8 { 99 }\n",
    )
    .unwrap();
    fs::write(source.path.join("src/unrelated.rs"), b"changed too\n").unwrap();

    let snapshot = Snapshot::create(parent.path(), &closure).unwrap();
    let snapshotted = fs::read(snapshot.input_path(&captured_source.normalized_path)).unwrap();
    assert_eq!(snapshotted, captured_bytes.as_ref());
    assert_eq!(sha256::digest(&snapshotted), captured_hash);
    assert_ne!(
        snapshotted,
        fs::read(source.path.join("src/lib.rs")).unwrap()
    );
    assert!(!snapshot.path().join("src/unrelated.rs").exists());
    let snapshot_path = snapshot.path().to_owned();
    let sibling = parent.path.join("user-owned");
    fs::write(&sibling, b"preserve").unwrap();
    drop(snapshot);
    assert!(
        !snapshot_path.exists(),
        "cleanup left entries: {:?}",
        fs::read_dir(&snapshot_path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>()
    );
    assert_eq!(fs::read(sibling).unwrap(), b"preserve");
}

#[test]
fn snapshot_permissions_are_private_and_read_only_after_sealing() {
    let source = TestDirectory::package();
    let parent = TestDirectory::new("parent");
    let closure = capture(&source);
    let snapshot = Snapshot::create(parent.path(), &closure).unwrap();

    assert_eq!(fs::metadata(snapshot.path()).unwrap().mode() & 0o777, 0o500);
    assert_eq!(
        fs::metadata(snapshot.path().join("src")).unwrap().mode() & 0o777,
        0o500
    );
    assert_eq!(
        fs::metadata(snapshot.path().join("src/lib.rs"))
            .unwrap()
            .mode()
            & 0o777,
        0o400
    );
}

#[test]
fn cleanup_stops_on_replaced_entries_without_following_them() {
    let source = TestDirectory::package();
    let parent = TestDirectory::new("parent");
    let outside = TestDirectory::new("outside");
    let outside_file = outside.path.join("keep");
    fs::write(&outside_file, b"do not remove").unwrap();
    let closure = capture(&source);
    let snapshot = Snapshot::create(parent.path(), &closure).unwrap();
    let snapshot_path = snapshot.path().to_owned();

    fs::set_permissions(&snapshot_path, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(snapshot_path.join("src"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::remove_file(snapshot_path.join("src/lib.rs")).unwrap();
    symlink(&outside_file, snapshot_path.join("src/lib.rs")).unwrap();
    drop(snapshot);

    assert_eq!(fs::read(&outside_file).unwrap(), b"do not remove");
    assert!(fs::symlink_metadata(snapshot_path.join("src/lib.rs"))
        .unwrap()
        .file_type()
        .is_symlink());
    fs::remove_file(snapshot_path.join("src/lib.rs")).unwrap();
    make_tree_writable(&snapshot_path);
    fs::remove_dir_all(snapshot_path).unwrap();
}

#[test]
fn cleanup_never_removes_a_replacement_at_the_temporary_root_name() {
    let source = TestDirectory::package();
    let parent = TestDirectory::new("parent");
    let closure = capture(&source);
    let snapshot = Snapshot::create(parent.path(), &closure).unwrap();
    let named_root = snapshot.path().to_owned();
    let displaced_root = parent.path.join("displaced-snapshot");

    fs::rename(&named_root, &displaced_root).unwrap();
    fs::create_dir(&named_root).unwrap();
    assert_eq!(snapshot.validate(), Err(SnapshotError::FileType));
    drop(snapshot);

    assert!(named_root.is_dir());
    assert!(displaced_root.is_dir());
}

#[test]
fn a_symlinked_temporary_parent_is_rejected_without_creating_a_snapshot() {
    let source = TestDirectory::package();
    let closure = capture(&source);
    let real_parent = TestDirectory::new("real-parent");
    let link_holder = TestDirectory::new("link-holder");
    let link = link_holder.path.join("temporary");
    symlink(real_parent.path(), &link).unwrap();

    assert_eq!(
        Snapshot::create(&link, &closure).unwrap_err(),
        SnapshotError::FileType
    );
    assert_eq!(fs::read_dir(real_parent.path()).unwrap().count(), 0);
}
