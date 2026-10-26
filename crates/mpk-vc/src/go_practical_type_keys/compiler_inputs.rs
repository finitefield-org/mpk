//! Complete original prepared SDK bytes and their source/export correspondence.
//! This private owner does not authenticate installed roots/context, execute a
//! compiler or prove compiler correctness. Bundle and source budgets stay distinct.
use super::*;
use crate::release_bundle::{
    BundleInventory, InventoryFile, InventoryScope, BUNDLE_CONTENT_HASH_DOMAIN,
    BUNDLE_DECLARED_BYTES_MAX, BUNDLE_FILE_BYTES_MAX, BUNDLE_INVENTORY_SCHEMA,
    PORTABLE_PATH_BYTES_MAX, SERIALIZED_INVENTORY_ENTRIES_MAX, UNIQUE_BUNDLE_FILES_MAX,
};
use crate::source_manifest::{ComponentIdentity, ToolchainIdentity};
use std::sync::Arc;
mod build_info;
mod source_roles;
pub use build_info::{
    parse_go_executable_build_info_text, read_go_executable_build_info, GoExecutableBuildInfo,
    GoExecutableModule,
};
pub use source_roles::{derive_go_compiler_source_role, GoCompilerSourceIndex};

const DOMAIN: HashDomain = HashDomain::new("MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0");
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceCompilerInputError {
    Input(GoTypeKeyError),
    Linkage,
    Inventory,
    Bytes,
    Component,
    Version,
    Source,
    Namespace,
    BuildInfo,
    SourceRole,
}
impl std::fmt::Display for GoSourceCompilerInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go original compiler inputs: {self:?}")
    }
}
impl std::error::Error for GoSourceCompilerInputError {}
use GoSourceCompilerInputError as C;
type CompilerResult<T> = std::result::Result<T, C>;
fn need(ok: bool, error: C) -> CompilerResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn path_valid(path: &str) -> bool {
    !path.is_empty()
        && path.len() as u64 <= PORTABLE_PATH_BYTES_MAX
        && path.split('/').all(|part| {
            let base = part.split('.').next().unwrap().to_ascii_uppercase();
            !part.is_empty()
                && part.len() <= 255
                && !part.ends_with('.')
                && !matches!(part, "." | "..")
                && !matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(base.len() == 4
                    && (base.starts_with("COM") || base.starts_with("LPT"))
                    && matches!(base.as_bytes()[3], b'1'..=b'9'))
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        })
}
/// An owned untrusted bundle file. Its identity is derived from the complete
/// bytes; naming/mode claims must still match the complete compiler inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoCompilerFileBuffer {
    entry: InventoryFile,
    data: Arc<[u8]>,
}
impl GoCompilerFileBuffer {
    pub fn import(path: &str, executable: bool, raw: &[u8]) -> CompilerResult<Self> {
        need(path_valid(path), C::Inventory)?;
        need(raw.len() as u64 <= BUNDLE_FILE_BYTES_MAX, C::Bytes)?;
        Ok(Self {
            entry: InventoryFile {
                path: path.into(),
                executable,
                size_bytes: raw.len() as i64,
                sha256: sha256_raw_file_bytes(raw).to_hex(),
            },
            data: Arc::from(raw),
        })
    }
    pub fn entry(&self) -> &InventoryFile {
        &self.entry
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Component {
    identity: ComponentIdentity,
    inventory: BundleInventory,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: String,
    source_export_inputs_sha256: String,
    sdk_capability: String,
    original_root: String,
    host: String,
    toolchain: ToolchainIdentity,
    inventory: BundleInventory,
    components: Vec<Component>,
    content_sha256: String,
}
fn inventory_hash(inventory: &BundleInventory) -> CompilerResult<String> {
    let raw = serde_json::to_vec(inventory).map_err(|_| C::Inventory)?;
    let parsed = parse_strict_json(&raw, limits()).map_err(|_| C::Inventory)?;
    Ok(hash_canonical_json(BUNDLE_CONTENT_HASH_DOMAIN, &parsed)
        .map_err(|_| C::Inventory)?
        .to_hex())
}
fn inventory_valid(inventory: &BundleInventory) -> CompilerResult<()> {
    need(
        inventory.schema == BUNDLE_INVENTORY_SCHEMA
            && !inventory.files.is_empty()
            && inventory.files.len() as u64 <= UNIQUE_BUNDLE_FILES_MAX,
        C::Inventory,
    )?;
    let mut bytes = 0u64;
    let mut folded = BTreeSet::new();
    for (i, file) in inventory.files.iter().enumerate() {
        need(
            path_valid(&file.path)
                && file.size_bytes >= 0
                && file.size_bytes as u64 <= BUNDLE_FILE_BYTES_MAX
                && file.sha256.len() == 64
                && file
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
                && (i == 0 || inventory.files[i - 1].path < file.path)
                && folded.insert(file.path.to_ascii_lowercase()),
            C::Inventory,
        )?;
        bytes = bytes
            .checked_add(file.size_bytes as u64)
            .ok_or(C::Inventory)?;
        need(bytes <= BUNDLE_DECLARED_BYTES_MAX, C::Inventory)?;
    }
    Ok(())
}
fn name(identity: &ComponentIdentity) -> &str {
    match identity {
        ComponentIdentity::Executable { name, .. } | ComponentIdentity::Content { name, .. } => {
            name
        }
    }
}
fn components(wire: &Wire) -> CompilerResult<()> {
    need(
        wire.inventory.scope
            == InventoryScope::ToolchainBundle {
                bundle_id: wire.toolchain.bundle_id.clone(),
            },
        C::Inventory,
    )?;
    inventory_valid(&wire.inventory)?;
    need(
        inventory_hash(&wire.inventory)? == wire.toolchain.distribution_sha256,
        C::Inventory,
    )?;
    let mut host = None;
    for file in &wire.inventory.files {
        need(file.path.starts_with("go/"), C::Inventory)?;
        if let Some(tail) = file.path.strip_prefix("go/pkg/tool/") {
            let (current, _) = tail.split_once('/').ok_or(C::Inventory)?;
            need(
                matches!(current, "linux_amd64" | "darwin_arm64")
                    && host.is_none_or(|old| old == current),
                C::Inventory,
            )?;
            host = Some(current);
        }
    }
    need(host == Some(wire.host.as_str()), C::Inventory)?;
    let mut tools = BTreeMap::from([
        ("go".to_string(), "go/bin/go".to_string()),
        ("gofmt".to_string(), "go/bin/gofmt".to_string()),
    ]);
    for tool in [
        "asm",
        "cgo",
        "compile",
        "cover",
        "link",
        "preprofile",
        "vet",
    ] {
        tools.insert(
            format!("go-tool-{tool}"),
            format!("go/pkg/tool/{}/{tool}", wire.host),
        );
    }
    need(
        wire.components.len() == tools.len() + 1
            && wire.toolchain.components.len() == wire.components.len(),
        C::Component,
    )?;
    let mut partition = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut serialized = wire.inventory.files.len() as u64;
    for component in &wire.components {
        let component_name = name(&component.identity);
        need(seen.insert(component_name), C::Component)?;
        inventory_valid(&component.inventory)?;
        need(
            component.inventory.scope
                == InventoryScope::Component {
                    bundle_id: wire.toolchain.bundle_id.clone(),
                    component_name: component_name.into(),
                },
            C::Component,
        )?;
        serialized += component.inventory.files.len() as u64;
        need(serialized <= SERIALIZED_INVENTORY_ENTRIES_MAX, C::Inventory)?;
        match &component.identity {
            ComponentIdentity::Executable {
                release,
                commit_hash,
                binary_sha256,
                ..
            } => {
                let file = component.inventory.files.first().ok_or(C::Component)?;
                need(
                    release == "go1.25.0"
                        && commit_hash.is_none()
                        && component.inventory.files.len() == 1
                        && file.executable
                        && binary_sha256 == &file.sha256
                        && tools.get(component_name) == Some(&file.path),
                    C::Component,
                )?;
            }
            ComponentIdentity::Content {
                release,
                content_sha256,
                ..
            } => {
                need(
                    component_name == "go-target-linux-amd64"
                        && release == "go1.25.0"
                        && component.inventory.files.iter().all(|f| !f.executable)
                        && inventory_hash(&component.inventory)? == *content_sha256,
                    C::Component,
                )?;
            }
        }
        for file in &component.inventory.files {
            need(partition.insert(&file.path, file).is_none(), C::Component)?;
        }
    }
    need(
        seen.contains("go-target-linux-amd64") && tools.keys().all(|k| seen.contains(k.as_str())),
        C::Component,
    )?;
    need(
        partition.len() == wire.inventory.files.len()
            && wire
                .inventory
                .files
                .iter()
                .all(|f| partition.get(&f.path) == Some(&f)),
        C::Component,
    )?;
    for (i, identity) in wire.toolchain.components.iter().enumerate() {
        need(
            i == 0 || name(&wire.toolchain.components[i - 1]) < name(identity),
            C::Component,
        )?;
        need(
            wire.components.iter().any(|c| c.identity == *identity),
            C::Component,
        )?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceCompilerInputs {
    exports: GoSourceExportInputs,
    wire: Wire,
    canonical: Vec<u8>,
    files: Vec<GoCompilerFileBuffer>,
    source_files: BTreeMap<String, GoTypeSourceObject>,
    executable_build_info: BTreeMap<String, GoExecutableBuildInfo>,
    source_index: GoCompilerSourceIndex,
}
impl GoSourceCompilerInputs {
    pub fn export_inputs(&self) -> &GoSourceExportInputs {
        &self.exports
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn files(&self) -> &[GoCompilerFileBuffer] {
        &self.files
    }
    pub fn inventory(&self) -> &BundleInventory {
        &self.wire.inventory
    }
    pub fn toolchain(&self) -> &ToolchainIdentity {
        &self.wire.toolchain
    }
    pub fn host(&self) -> &str {
        &self.wire.host
    }
    pub fn sdk_source_files(&self) -> &BTreeMap<String, GoTypeSourceObject> {
        &self.source_files
    }
    pub fn executable_build_info(&self) -> &BTreeMap<String, GoExecutableBuildInfo> {
        &self.executable_build_info
    }
    pub fn source_index(&self) -> &GoCompilerSourceIndex {
        &self.source_index
    }
}

/// Independently bind every actual prepared SDK file and mode to its complete
/// inventory/component partition, and bind every captured SDK source to those
/// same original bytes. Installed compiler/runtime/context authority remains due.
pub fn import_go_source_compiler_inputs(
    record: &[u8],
    exports: GoSourceExportInputs,
    files: Vec<GoCompilerFileBuffer>,
) -> CompilerResult<GoSourceCompilerInputs> {
    let wire: Wire = decode(record).map_err(C::Input)?;
    need(
        wire.schema == "mpk.go.practical.compiler_image_inputs.v1",
        C::Input(GoTypeKeyError::Shape),
    )?;
    need(
        identity(&wire, DOMAIN, "content_sha256").map_err(C::Input)? == wire.content_sha256,
        C::Input(GoTypeKeyError::Identity),
    )?;
    let export: serde_json::Value =
        serde_json::from_slice(exports.canonical_bytes()).map_err(|_| C::Linkage)?;
    need(
        export["content_sha256"] == wire.source_export_inputs_sha256
            && export["sdk_capability"] == wire.sdk_capability
            && export["compiler_sha256"] == wire.toolchain.distribution_sha256,
        C::Linkage,
    )?;
    need(
        export["origins"]
            .as_array()
            .ok_or(C::Linkage)?
            .iter()
            .any(|o| o["capability"] == wire.sdk_capability && o["path"] == wire.original_root),
        C::Linkage,
    )?;
    components(&wire)?;
    need(
        files.len() == wire.inventory.files.len()
            && files
                .iter()
                .zip(&wire.inventory.files)
                .all(|(a, b)| a.entry() == b),
        C::Bytes,
    )?;
    let lookup = files
        .iter()
        .map(|f| (f.entry.path.as_str(), f))
        .collect::<BTreeMap<_, _>>();
    let go = lookup.get("go/bin/go").ok_or(C::Component)?;
    need(export["go_binary_sha256"] == go.entry.sha256, C::Linkage)?;
    let mut executable_build_info = BTreeMap::new();
    for file in &files {
        if file.entry.executable {
            // The exact complete executable/component partition has already
            // been checked. Derive identity from bytes, never from a producer
            // build-info record or from an invocation of this executable.
            let info = read_go_executable_build_info(file.bytes())?;
            let tool = file.entry.path.rsplit('/').next().ok_or(C::Component)?;
            info.validate_compiler_tool(tool, &wire.host)?;
            executable_build_info.insert(file.entry.path.clone(), info);
        }
    }
    for (path, size, hash) in [
        (
            "go/VERSION",
            35,
            "5b444481508c9a41ae34393cf2e0e06ddfa930b02c783a7ccad450b6a8eab773",
        ),
        (
            "go/doc/go_spec.html",
            293108,
            "42a4ff1c01efe8dc23eca4378b8fb03d76e9f2f079dab01298ffd74d8b0e3149",
        ),
        (
            "go/src/go/types/version.go",
            2085,
            "6a41744c247a4e22afa59098970b2dee3693f6c0a0556a043236758415755e83",
        ),
        (
            "go/src/go/version/version.go",
            1970,
            "7a910410a2b437e873749b2bb551085ff993617304331784afd83954a74c7f5c",
        ),
    ] {
        let file = lookup.get(path).ok_or(C::Version)?;
        need(
            !file.entry.executable && file.entry.size_bytes == size && file.entry.sha256 == hash,
            C::Version,
        )?;
    }
    for path in [
        "go/go.env",
        "go/src/runtime/runtime2.go",
        "go/src/runtime/asm_amd64.s",
        "go/src/runtime/os_linux.go",
        "go/src/builtin/builtin.go",
        "go/src/unsafe/unsafe.go",
    ] {
        need(
            lookup.get(path).is_some_and(|f| !f.entry.executable),
            C::Version,
        )?;
    }
    let snapshot = exports.capture_inputs().snapshot_inputs();
    let mut source_files = BTreeMap::new();
    for file in &files {
        if let Some(relative) = file.entry.path.strip_prefix("go/src/") {
            let path = format!("src/{relative}");
            let object = snapshot
                .source_objects()
                .iter()
                .find(|o| o.capability == wire.sdk_capability && o.path == path)
                .ok_or(C::Source)?;
            need(
                !file.entry.executable
                    && object.bytes == file.entry.size_bytes as u64
                    && object.raw_sha256 == file.entry.sha256,
                C::Source,
            )?;
            source_files.insert(file.entry.path.clone(), object.clone());
        }
    }
    for object in snapshot
        .source_objects()
        .iter()
        .filter(|o| o.capability == wire.sdk_capability)
    {
        let path = format!("go/{}", object.path);
        let file = lookup.get(path.as_str()).ok_or(C::Source)?;
        let original = snapshot.source(&object.id).ok_or(C::Source)?;
        need(original.bytes() == file.bytes(), C::Source)?;
    }
    let source_index = GoCompilerSourceIndex::derive(snapshot, &source_files)?;
    // Check each available original parent observation. Unvisited namespaces
    // remain unknown; empty directories are not invented from a file inventory.
    for file in &files {
        let path = file.entry.path.strip_prefix("go/").ok_or(C::Inventory)?;
        let parts = path.split('/').collect::<Vec<_>>();
        let mut parent = String::new();
        for (index, part) in parts.iter().enumerate() {
            match snapshot.read_directory(&wire.sdk_capability, &parent) {
                Ok(entries) => {
                    let entry = entries
                        .iter()
                        .find(|e| e.name_bytes() == part.as_bytes())
                        .ok_or(C::Namespace)?;
                    let expected = if index + 1 == parts.len() { 0 } else { 1 << 31 };
                    need(entry.type_bits() == expected, C::Namespace)?;
                }
                Err(GoSourceSnapshotLookupError::NotCaptured) => break,
                Err(_) => return Err(C::Namespace),
            }
            if !parent.is_empty() {
                parent.push('/');
            }
            parent.push_str(part);
        }
    }
    for (_, parent, entries) in snapshot
        .observed_directories()
        .filter(|(capability, _, _)| *capability == wire.sdk_capability)
    {
        for entry in entries {
            if entry.type_bits() == 1 << 31 {
                continue;
            }
            need(entry.type_bits() == 0, C::Namespace)?;
            let name = std::str::from_utf8(entry.name_bytes()).map_err(|_| C::Namespace)?;
            let path = if parent.is_empty() {
                format!("go/{name}")
            } else {
                format!("go/{parent}/{name}")
            };
            need(lookup.contains_key(path.as_str()), C::Namespace)?;
        }
    }
    Ok(GoSourceCompilerInputs {
        exports,
        wire,
        canonical: record.to_vec(),
        files,
        source_files,
        executable_build_info,
        source_index,
    })
}
