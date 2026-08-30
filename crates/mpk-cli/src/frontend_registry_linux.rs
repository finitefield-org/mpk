use super::{
    bundle_invalid, raw_sha256, BundleSnapshot, FrontendReleaseCode, FrontendReleaseError,
    SnapshotFile, EXPECTED_REGISTRY_ID, EXPECTED_REGISTRY_SHA256,
};
use mpk_vc::{validate_release_registry, BundleInventory, ValidatedReleaseRegistry};
use rustix::fs::{openat2, Mode, OFlags, RawDir, ResolveFlags, CWD};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::mem::MaybeUninit;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::sync::Arc;

pub(super) struct InstalledReleaseRoot {
    directory: File,
    device: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StableMetadata {
    device: u64,
    inode: u64,
    mode: u32,
    links: u64,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[derive(Debug, PartialEq, Eq)]
struct ObservedInventory {
    directories: BTreeMap<String, StableMetadata>,
    files: BTreeMap<String, StableMetadata>,
}

pub(super) fn load_installed_registry(
) -> Result<(InstalledReleaseRoot, ValidatedReleaseRegistry), FrontendReleaseError> {
    if !cfg!(all(target_arch = "x86_64", target_env = "gnu")) {
        return Err(release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing host does not match the registered Linux ABI",
        ));
    }
    let root = installed_release_root()?;
    let registry = open_installed_registry(&root)?;
    Ok((root, registry))
}

pub(super) fn load_successor_descriptors(
) -> Result<(InstalledReleaseRoot, Vec<u8>, Vec<u8>), FrontendReleaseError> {
    if !cfg!(all(target_arch = "x86_64", target_env = "gnu")) {
        return Err(release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing host does not match the successor Linux ABI",
        ));
    }
    let root = installed_release_root()?;
    require_exact_names(
        &root,
        "share",
        &["mpk"],
        FrontendReleaseCode::RegistryMissing,
        FrontendReleaseCode::RegistryInvalid,
    )?;
    require_exact_names(
        &root,
        "share/mpk",
        &["bundle-registry.json", "semantic-profile-registry.json"],
        FrontendReleaseCode::RegistryMissing,
        FrontendReleaseCode::RegistryInvalid,
    )?;
    let registry = read_stable_installed_file(
        &root,
        "share/mpk/bundle-registry.json",
        mpk_vc::REGISTRY_TRANSPORT_BYTES_MAX,
    )?;
    let semantic_registry = read_stable_installed_file(
        &root,
        "share/mpk/semantic-profile-registry.json",
        mpk_vc::REGISTRY_TRANSPORT_BYTES_MAX,
    )?;
    Ok((root, registry, semantic_registry))
}

fn installed_release_root() -> Result<InstalledReleaseRoot, FrontendReleaseError> {
    let image = File::open("/proc/self/exe").map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing image cannot be retained",
        )
    })?;
    let image_metadata = image.metadata().map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing image identity is unavailable",
        )
    })?;
    let executable = fs::read_link("/proc/self/exe").map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing image path is unavailable",
        )
    })?;
    if executable.to_string_lossy().ends_with(" (deleted)")
        || !executable.is_absolute()
        || executable.file_name().and_then(|name| name.to_str()) != Some("mpk")
        || executable
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            != Some("bin")
    {
        return Err(release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the executing image is not an installed bin/mpk",
        ));
    }
    let root_path = executable.parent().and_then(Path::parent).ok_or_else(|| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed release root is unavailable",
        )
    })?;
    let root_fd = openat2(
        CWD,
        root_path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
        ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed release root cannot be retained without links",
        )
    })?;
    let directory = File::from(root_fd);
    let root_metadata = directory.metadata().map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed release root identity is unavailable",
        )
    })?;
    if !root_metadata.is_dir() || stat_mode(&root_metadata) != 0o555 {
        return Err(release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed release root identity is invalid",
        ));
    }
    let root = InstalledReleaseRoot {
        device: root_metadata.dev(),
        directory,
    };
    let named = open_regular_beneath(
        &root,
        "bin/mpk",
        0o555,
        FrontendReleaseCode::SandboxUnavailable,
        "the installed bin/mpk identity is unavailable",
    )?;
    let named_metadata = named.metadata().map_err(|_| {
        release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed bin/mpk identity is unavailable",
        )
    })?;
    if !same_identity(&image_metadata, &named_metadata) {
        return Err(release_error(
            FrontendReleaseCode::SandboxUnavailable,
            "the installed bin/mpk differs from the executing image",
        ));
    }
    require_exact_names(
        &root,
        "bin",
        &["mpk"],
        FrontendReleaseCode::SandboxUnavailable,
        FrontendReleaseCode::SandboxUnavailable,
    )?;
    Ok(root)
}

fn open_installed_registry(
    root: &InstalledReleaseRoot,
) -> Result<ValidatedReleaseRegistry, FrontendReleaseError> {
    require_exact_names(
        root,
        "share",
        &["mpk"],
        FrontendReleaseCode::RegistryMissing,
        FrontendReleaseCode::RegistryInvalid,
    )?;
    require_exact_names(
        root,
        "share/mpk",
        &["bundle-registry.json"],
        FrontendReleaseCode::RegistryMissing,
        FrontendReleaseCode::RegistryInvalid,
    )?;
    let bytes = read_stable_installed_file(
        root,
        "share/mpk/bundle-registry.json",
        mpk_vc::REGISTRY_TRANSPORT_BYTES_MAX,
    )?;
    let registry = validate_release_registry(&bytes).map_err(|error| {
        let code = if error.code() == mpk_vc::ReleaseRegistryErrorCode::Limit {
            FrontendReleaseCode::RegistryLimit
        } else {
            FrontendReleaseCode::RegistryInvalid
        };
        release_error(code, "installed registry validation failed")
    })?;
    if registry.registry().id != EXPECTED_REGISTRY_ID
        || registry.registry_digest().as_bytes() != &EXPECTED_REGISTRY_SHA256
    {
        return Err(release_error(
            FrontendReleaseCode::RegistryMismatch,
            "installed registry differs from the build-pinned identity",
        ));
    }
    Ok(registry)
}

fn read_stable_installed_file(
    root: &InstalledReleaseRoot,
    path: &str,
    limit: u64,
) -> Result<Vec<u8>, FrontendReleaseError> {
    let mut file = open_regular_beneath(
        root,
        path,
        0o444,
        FrontendReleaseCode::RegistryMissing,
        "the installed descriptor is missing",
    )?;
    let before = file.metadata().map_err(|_| {
        release_error(
            FrontendReleaseCode::RegistryInvalid,
            "installed descriptor metadata is unavailable",
        )
    })?;
    if before.len() > limit {
        return Err(release_error(
            FrontendReleaseCode::RegistryLimit,
            "installed descriptor exceeds its transport limit",
        ));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(before.len()).unwrap_or(0));
    file.by_ref()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            release_error(
                FrontendReleaseCode::RegistryInvalid,
                "installed descriptor could not be read exactly once",
            )
        })?;
    let after = file.metadata().map_err(|_| {
        release_error(
            FrontendReleaseCode::RegistryInvalid,
            "installed descriptor identity disappeared",
        )
    })?;
    if bytes.len() as u64 != before.len() || !same_stable_file(&before, &after) {
        return Err(release_error(
            FrontendReleaseCode::RegistryInvalid,
            "installed descriptor changed during capture",
        ));
    }
    Ok(bytes)
}

pub(super) fn snapshot_selected_bundles(
    root: &InstalledReleaseRoot,
    registry: &ValidatedReleaseRegistry,
    frontend_bundle_id: &str,
    toolchain_bundle_id: &str,
) -> Result<BTreeMap<String, Arc<BundleSnapshot>>, FrontendReleaseError> {
    let mut expected = BTreeMap::new();
    for frontend in &registry.registry().frontend_bundles {
        expected.insert(frontend.bundle_id.clone(), &frontend.inventory);
    }
    for toolchain in &registry.registry().toolchain_bundles {
        expected.insert(toolchain.bundle_id.clone(), &toolchain.inventory);
    }
    snapshot_inventory_set(root, &expected, frontend_bundle_id, toolchain_bundle_id)
}

pub(super) fn snapshot_inventory_set(
    root: &InstalledReleaseRoot,
    expected: &BTreeMap<String, &BundleInventory>,
    frontend_bundle_id: &str,
    toolchain_bundle_id: &str,
) -> Result<BTreeMap<String, Arc<BundleSnapshot>>, FrontendReleaseError> {
    require_exact_names(
        root,
        "libexec",
        &["mpk"],
        FrontendReleaseCode::BundleInvalid,
        FrontendReleaseCode::BundleInvalid,
    )?;
    require_exact_names(
        root,
        "libexec/mpk",
        &["bundles"],
        FrontendReleaseCode::BundleInvalid,
        FrontendReleaseCode::BundleInvalid,
    )?;
    let expected_ids = expected.keys().cloned().collect::<BTreeSet<_>>();
    let observed = directory_names_beneath(
        root,
        "libexec/mpk/bundles",
        FrontendReleaseCode::BundleInvalid,
    )?;
    if observed != expected_ids {
        return Err(bundle_invalid(
            "installed bundle directory set is not exact",
        ));
    }
    let mut directory_identities = BTreeSet::new();
    let mut file_identities = BTreeSet::new();
    let mut observed_inventories = BTreeMap::new();
    for (bundle_id, inventory) in expected {
        let bundle_root = format!("libexec/mpk/bundles/{bundle_id}");
        let observed = enumerate_bundle(root, &bundle_root)?;
        validate_inventory_metadata(
            &observed,
            inventory,
            &mut directory_identities,
            &mut file_identities,
        )?;
        observed_inventories.insert(bundle_id.clone(), observed);
    }
    let selected_ids = BTreeSet::from([
        frontend_bundle_id.to_owned(),
        toolchain_bundle_id.to_owned(),
    ]);
    let mut snapshots = BTreeMap::new();
    for bundle_id in selected_ids {
        let inventory = expected
            .get(&bundle_id)
            .ok_or_else(|| bundle_invalid("selected bundle inventory is absent"))?;
        let observed = observed_inventories
            .get(&bundle_id)
            .ok_or_else(|| bundle_invalid("selected bundle metadata is absent"))?;
        let bundle_root = format!("libexec/mpk/bundles/{bundle_id}");
        let snapshot = snapshot_inventory(root, &bundle_root, inventory, observed)?;
        snapshots.insert(bundle_id, Arc::new(snapshot));
    }
    for bundle_id in expected.keys() {
        let bundle_root = format!("libexec/mpk/bundles/{bundle_id}");
        if observed_inventories.get(bundle_id) != Some(&enumerate_bundle(root, &bundle_root)?) {
            return Err(bundle_invalid(
                "installed bundle namespace changed during snapshot",
            ));
        }
    }
    let observed_after = directory_names_beneath(
        root,
        "libexec/mpk/bundles",
        FrontendReleaseCode::BundleInvalid,
    )?;
    if observed_after != expected_ids {
        return Err(bundle_invalid(
            "installed bundle directory set changed during snapshot",
        ));
    }
    require_exact_names(
        root,
        ".",
        &["bin", "libexec", "share"],
        FrontendReleaseCode::BundleInvalid,
        FrontendReleaseCode::BundleInvalid,
    )?;
    Ok(snapshots)
}

fn snapshot_inventory(
    release_root: &InstalledReleaseRoot,
    bundle_root: &str,
    inventory: &BundleInventory,
    observed_metadata: &ObservedInventory,
) -> Result<BundleSnapshot, FrontendReleaseError> {
    let expected: BTreeMap<_, _> = inventory
        .files
        .iter()
        .map(|file| (file.path.clone(), file))
        .collect();
    let observed_before = enumerate_bundle(release_root, bundle_root)?;
    if &observed_before != observed_metadata {
        return Err(bundle_invalid("bundle metadata changed before snapshot"));
    }
    let mut files = BTreeMap::new();
    for (relative, descriptor) in expected {
        let installed_path = format!("{bundle_root}/{relative}");
        let mut file = open_regular_beneath(
            release_root,
            &installed_path,
            if descriptor.executable { 0o555 } else { 0o444 },
            FrontendReleaseCode::BundleInvalid,
            "installed bundle file is missing",
        )?;
        let before = file
            .metadata()
            .map_err(|_| bundle_invalid("bundle file metadata is unavailable"))?;
        if before.len() != u64::try_from(descriptor.size_bytes).unwrap_or(u64::MAX)
            || observed_before.files.get(&relative) != Some(&stable_metadata(&before))
        {
            return Err(bundle_invalid("bundle file size or identity is invalid"));
        }
        let mut bytes = Vec::with_capacity(usize::try_from(before.len()).unwrap_or(0));
        file.by_ref()
            .take(mpk_vc::BUNDLE_FILE_BYTES_MAX + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| bundle_invalid("bundle file cannot be snapshotted"))?;
        let after = file
            .metadata()
            .map_err(|_| bundle_invalid("bundle file identity disappeared"))?;
        let digest = raw_sha256(&bytes);
        if bytes.len() as u64 != before.len()
            || !same_stable_file(&before, &after)
            || digest != descriptor.sha256
        {
            return Err(bundle_invalid("bundle file changed or failed its digest"));
        }
        files.insert(
            relative,
            SnapshotFile {
                bytes,
                executable: descriptor.executable,
                sha256: digest,
            },
        );
    }
    let observed_after = enumerate_bundle(release_root, bundle_root)?;
    if observed_before != observed_after {
        return Err(bundle_invalid("bundle namespace changed during snapshot"));
    }
    Ok(BundleSnapshot { files })
}

fn validate_inventory_metadata(
    observed: &ObservedInventory,
    inventory: &BundleInventory,
    directory_identities: &mut BTreeSet<(u64, u64)>,
    file_identities: &mut BTreeSet<(u64, u64)>,
) -> Result<(), FrontendReleaseError> {
    let expected: BTreeMap<_, _> = inventory
        .files
        .iter()
        .map(|file| (file.path.clone(), file))
        .collect();
    if observed.files.keys().cloned().collect::<BTreeSet<_>>() != expected.keys().cloned().collect()
        || observed
            .directories
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != implied_directories(expected.keys().map(String::as_str))
    {
        return Err(bundle_invalid(
            "installed bundle inventory is incomplete or contains extras",
        ));
    }
    for identity in observed.directories.values() {
        if !directory_identities.insert((identity.device, identity.inode)) {
            return Err(bundle_invalid("bundle directory identity is aliased"));
        }
    }
    for (relative, descriptor) in expected {
        let metadata = observed
            .files
            .get(&relative)
            .ok_or_else(|| bundle_invalid("installed bundle file metadata is absent"))?;
        let expected_mode = if descriptor.executable { 0o555 } else { 0o444 };
        if metadata.mode != expected_mode
            || metadata.size != u64::try_from(descriptor.size_bytes).unwrap_or(u64::MAX)
            || !file_identities.insert((metadata.device, metadata.inode))
        {
            return Err(bundle_invalid("bundle file size or identity is invalid"));
        }
    }
    Ok(())
}

fn implied_directories<'a>(paths: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
    let mut directories = BTreeSet::from([String::new()]);
    for path in paths {
        let mut components: Vec<_> = path.split('/').collect();
        components.pop();
        while !components.is_empty() {
            directories.insert(components.join("/"));
            components.pop();
        }
    }
    directories
}

fn enumerate_bundle(
    release_root: &InstalledReleaseRoot,
    bundle_root: &str,
) -> Result<ObservedInventory, FrontendReleaseError> {
    let mut pending = vec![(bundle_root.to_owned(), String::new())];
    let mut directories = BTreeMap::new();
    let mut files = BTreeMap::new();
    while let Some((installed_directory, relative_directory)) = pending.pop() {
        let directory = open_directory_beneath(
            release_root,
            &installed_directory,
            FrontendReleaseCode::BundleInvalid,
        )?;
        let metadata = directory
            .metadata()
            .map_err(|_| bundle_invalid("bundle directory metadata is unavailable"))?;
        directories.insert(relative_directory.clone(), stable_metadata(&metadata));
        for name in read_directory_names(&directory, FrontendReleaseCode::BundleInvalid)? {
            let installed_path = format!("{installed_directory}/{name}");
            let relative = if relative_directory.is_empty() {
                name
            } else {
                format!("{relative_directory}/{name}")
            };
            let opened = open_entry_beneath(
                release_root,
                &installed_path,
                FrontendReleaseCode::BundleInvalid,
            )?;
            let metadata = opened
                .metadata()
                .map_err(|_| bundle_invalid("bundle entry metadata is unavailable"))?;
            if metadata.is_dir() {
                if stat_mode(&metadata) != 0o555 || metadata.dev() != release_root.device {
                    return Err(bundle_invalid("bundle directory identity is invalid"));
                }
                pending.push((installed_path, relative));
            } else if metadata.is_file() {
                if metadata.dev() != release_root.device
                    || metadata.nlink() != 1
                    || !matches!(stat_mode(&metadata), 0o444 | 0o555)
                {
                    return Err(bundle_invalid("bundle file identity is invalid"));
                }
                files.insert(relative, stable_metadata(&metadata));
            } else {
                return Err(bundle_invalid("bundle entry is not regular"));
            }
        }
    }
    Ok(ObservedInventory { directories, files })
}

fn require_exact_names(
    root: &InstalledReleaseRoot,
    path: &str,
    expected: &[&str],
    open_code: FrontendReleaseCode,
    mismatch_code: FrontendReleaseCode,
) -> Result<(), FrontendReleaseError> {
    let observed = directory_names_beneath(root, path, open_code)?;
    let expected: BTreeSet<_> = expected.iter().map(|name| (*name).to_owned()).collect();
    if observed != expected {
        return Err(release_error(
            mismatch_code,
            "installed directory entries are not exact",
        ));
    }
    Ok(())
}

fn directory_names_beneath(
    root: &InstalledReleaseRoot,
    path: &str,
    code: FrontendReleaseCode,
) -> Result<BTreeSet<String>, FrontendReleaseError> {
    let directory = open_directory_beneath(root, path, code)?;
    read_directory_names(&directory, code).map(|names| names.into_iter().collect())
}

fn read_directory_names(
    directory: &File,
    code: FrontendReleaseCode,
) -> Result<Vec<String>, FrontendReleaseError> {
    let mut storage = [MaybeUninit::uninit(); 64 * 1024];
    let mut reader = RawDir::new(directory, &mut storage);
    let mut names = Vec::new();
    while let Some(entry) = reader.next() {
        let entry =
            entry.map_err(|_| release_error(code, "installed directory enumeration failed"))?;
        let bytes = entry.file_name().to_bytes();
        if matches!(bytes, b"." | b"..") {
            continue;
        }
        let name = std::str::from_utf8(bytes)
            .map_err(|_| release_error(code, "installed directory name is not UTF-8"))?;
        if name.is_empty() || name.contains('/') {
            return Err(release_error(code, "installed directory name is invalid"));
        }
        names.push(name.to_owned());
    }
    names.sort();
    if names.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(release_error(
            code,
            "installed directory contains duplicate names",
        ));
    }
    Ok(names)
}

fn open_directory_beneath(
    root: &InstalledReleaseRoot,
    path: &str,
    code: FrontendReleaseCode,
) -> Result<File, FrontendReleaseError> {
    let file = open_beneath(root, path, true, code, "installed directory is unavailable")?;
    let metadata = file
        .metadata()
        .map_err(|_| release_error(code, "installed directory metadata is unavailable"))?;
    if !metadata.is_dir() || metadata.dev() != root.device || stat_mode(&metadata) != 0o555 {
        return Err(release_error(
            code,
            "installed directory metadata is invalid",
        ));
    }
    Ok(file)
}

fn open_entry_beneath(
    root: &InstalledReleaseRoot,
    path: &str,
    code: FrontendReleaseCode,
) -> Result<File, FrontendReleaseError> {
    open_beneath(root, path, false, code, "installed entry is unavailable")
}

fn open_beneath(
    root: &InstalledReleaseRoot,
    path: &str,
    directory: bool,
    code: FrontendReleaseCode,
    detail: &'static str,
) -> Result<File, FrontendReleaseError> {
    if !safe_installed_relative_path(path) {
        return Err(release_error(code, "installed relative path is invalid"));
    }
    let mut flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
    if directory {
        flags |= OFlags::DIRECTORY;
    }
    let descriptor = openat2(
        &root.directory,
        path,
        flags,
        Mode::empty(),
        ResolveFlags::BENEATH
            | ResolveFlags::NO_MAGICLINKS
            | ResolveFlags::NO_SYMLINKS
            | ResolveFlags::NO_XDEV,
    )
    .map_err(|_| release_error(code, detail))?;
    Ok(File::from(descriptor))
}

fn safe_installed_relative_path(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && !path.starts_with('/')
            && !path.contains('\\')
            && path
                .split('/')
                .all(|component| !component.is_empty() && !matches!(component, "." | "..")))
}

fn open_regular_beneath(
    root: &InstalledReleaseRoot,
    path: &str,
    mode: u32,
    code: FrontendReleaseCode,
    missing_detail: &'static str,
) -> Result<File, FrontendReleaseError> {
    let file = open_beneath(root, path, false, code, missing_detail)?;
    let opened = file
        .metadata()
        .map_err(|_| release_error(code, "opened file metadata is unavailable"))?;
    if !opened.is_file()
        || opened.nlink() != 1
        || stat_mode(&opened) != mode
        || opened.dev() != root.device
    {
        return Err(release_error(
            code,
            "opened installed file identity is invalid",
        ));
    }
    Ok(file)
}

fn stable_metadata(metadata: &Metadata) -> StableMetadata {
    StableMetadata {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: stat_mode(metadata),
        links: metadata.nlink(),
        size: metadata.len(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    }
}

fn stat_mode(metadata: &Metadata) -> u32 {
    metadata.permissions().mode() & 0o777
}

fn same_identity(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev() && left.ino() == right.ino()
}

fn same_stable_file(left: &Metadata, right: &Metadata) -> bool {
    same_identity(left, right)
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

fn release_error(code: FrontendReleaseCode, detail: &'static str) -> FrontendReleaseError {
    FrontendReleaseError::new(code, detail)
}
