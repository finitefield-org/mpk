#![allow(dead_code)]

#[cfg(test)]
use mpk_vc::validate_release_registry;
#[cfg(test)]
use mpk_vc::InventoryFile;
use mpk_vc::{
    BundleInventory, FrontendBundle, ReleaseSelectionRequest, ToolchainBundle,
    ValidatedReleaseRegistry,
};
#[cfg(any(test, target_os = "linux"))]
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
#[cfg(target_os = "linux")]
#[path = "frontend_registry_linux.rs"]
mod linux;

include!(concat!(env!("OUT_DIR"), "/frontend_registry_constants.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrontendReleaseCode {
    RegistryMissing,
    RegistryLimit,
    RegistryInvalid,
    RegistryMismatch,
    BundleInvalid,
    BundleUnknown,
    BundleIncompatible,
    RegistryAssertion,
    SandboxUnavailable,
}

impl FrontendReleaseCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryMissing => "FRONTEND_REGISTRY_MISSING",
            Self::RegistryLimit => "FRONTEND_REGISTRY_LIMIT",
            Self::RegistryInvalid => "FRONTEND_REGISTRY_INVALID",
            Self::RegistryMismatch => "FRONTEND_REGISTRY_MISMATCH",
            Self::BundleInvalid => "FRONTEND_BUNDLE_INVALID",
            Self::BundleUnknown => "FRONTEND_BUNDLE_UNKNOWN",
            Self::BundleIncompatible => "FRONTEND_BUNDLE_INCOMPATIBLE",
            Self::RegistryAssertion => "FRONTEND_REGISTRY_ASSERTION",
            Self::SandboxUnavailable => "FRONTEND_SANDBOX_UNAVAILABLE",
        }
    }
}

#[derive(Debug)]
pub(crate) struct FrontendReleaseError {
    code: FrontendReleaseCode,
    detail: &'static str,
}

impl FrontendReleaseError {
    fn new(code: FrontendReleaseCode, detail: &'static str) -> Self {
        Self { code, detail }
    }

    pub(crate) const fn code(&self) -> FrontendReleaseCode {
        self.code
    }
}

impl fmt::Display for FrontendReleaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl Error for FrontendReleaseError {}

#[derive(Clone, Debug)]
pub(crate) struct SnapshotFile {
    pub(crate) bytes: Vec<u8>,
    pub(crate) executable: bool,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug)]
pub(crate) struct BundleSnapshot {
    files: BTreeMap<String, SnapshotFile>,
}

impl BundleSnapshot {
    pub(crate) fn file(&self, path: &str) -> Option<&SnapshotFile> {
        self.files.get(path)
    }

    pub(crate) fn files(&self) -> &BTreeMap<String, SnapshotFile> {
        &self.files
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SelectedFrontendRelease {
    pub(crate) registry: ValidatedReleaseRegistry,
    pub(crate) registry_id: String,
    pub(crate) registry_sha256: String,
    pub(crate) frontend: FrontendBundle,
    pub(crate) toolchain: ToolchainBundle,
    pub(crate) pointer_width: i64,
    pub(crate) limit_profile_id: String,
    pub(crate) frontend_snapshot: Arc<BundleSnapshot>,
    pub(crate) toolchain_snapshot: Arc<BundleSnapshot>,
}

pub(crate) struct InstalledReleaseResolver {
    registry: ValidatedReleaseRegistry,
    #[cfg(target_os = "linux")]
    root: linux::InstalledReleaseRoot,
}

/// Descriptor-relative handle for the sole installed successor image.
///
/// It retains the executing image's root descriptor, captures both registry
/// transports exactly once, and offers no path, environment, or executable
/// override.
pub(crate) struct InstalledSuccessorRelease {
    pub(crate) registry_bytes: Vec<u8>,
    pub(crate) semantic_registry_bytes: Vec<u8>,
    #[cfg(target_os = "linux")]
    root: linux::InstalledReleaseRoot,
}

impl InstalledSuccessorRelease {
    pub(crate) fn open() -> Result<Self, FrontendReleaseError> {
        #[cfg(not(target_os = "linux"))]
        {
            Err(FrontendReleaseError::new(
                FrontendReleaseCode::SandboxUnavailable,
                "the successor installed release root is Linux-only",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            let (root, registry_bytes, semantic_registry_bytes) =
                linux::load_successor_descriptors()?;
            Ok(Self {
                registry_bytes,
                semantic_registry_bytes,
                root,
            })
        }
    }

    pub(crate) fn snapshot_selected_bundles(
        &self,
        expected: &BTreeMap<String, &BundleInventory>,
        frontend_bundle_id: &str,
        toolchain_bundle_id: &str,
    ) -> Result<BTreeMap<String, Arc<BundleSnapshot>>, FrontendReleaseError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (expected, frontend_bundle_id, toolchain_bundle_id);
            Err(FrontendReleaseError::new(
                FrontendReleaseCode::SandboxUnavailable,
                "the staged installed release root is Linux-only",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            linux::snapshot_inventory_set(
                &self.root,
                expected,
                frontend_bundle_id,
                toolchain_bundle_id,
            )
        }
    }
}

impl InstalledReleaseResolver {
    /// Opens only the registry installed beside the already-running `bin/mpk`.
    /// There is deliberately no path, environment, or feature-based constructor.
    pub(crate) fn open() -> Result<Self, FrontendReleaseError> {
        #[cfg(not(target_os = "linux"))]
        {
            Err(FrontendReleaseError::new(
                FrontendReleaseCode::SandboxUnavailable,
                "the v0 installed release root is Linux-only",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            let (root, registry) = linux::load_installed_registry()?;
            Ok(Self { registry, root })
        }
    }

    pub(crate) fn resolve(
        &self,
        request: &ReleaseSelectionRequest,
    ) -> Result<SelectedFrontendRelease, FrontendReleaseError> {
        let resolved = self.registry.resolve(request).map_err(selection_error)?;
        #[cfg(not(target_os = "linux"))]
        {
            let _ = resolved;
            Err(FrontendReleaseError::new(
                FrontendReleaseCode::SandboxUnavailable,
                "the v0 installed release root is Linux-only",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            let snapshots = linux::snapshot_selected_bundles(
                &self.root,
                &self.registry,
                &resolved.frontend.bundle_id,
                &resolved.toolchain.bundle_id,
            )?;
            selected_from_snapshots(&self.registry, resolved, &snapshots)
        }
    }
}

pub(crate) fn assert_embedded_registry(
    request: &ReleaseSelectionRequest,
) -> Result<(), FrontendReleaseError> {
    if request.registry_id != EXPECTED_REGISTRY_ID
        || !matches_embedded_registry_sha256(&request.registry_sha256)
    {
        return Err(FrontendReleaseError::new(
            FrontendReleaseCode::RegistryAssertion,
            "caller registry assertions differ from the build-pinned identity",
        ));
    }
    Ok(())
}

fn matches_embedded_registry_sha256(value: &str) -> bool {
    value.len() == EXPECTED_REGISTRY_SHA256.len() * 2
        && value
            .as_bytes()
            .chunks_exact(2)
            .zip(EXPECTED_REGISTRY_SHA256)
            .all(|(pair, expected)| decode_hex_pair(pair) == Some(expected))
}

fn decode_hex_pair(pair: &[u8]) -> Option<u8> {
    let [high, low] = pair else {
        return None;
    };
    Some(hex_nibble(*high)? << 4 | hex_nibble(*low)?)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn selection_error(error: mpk_vc::ReleaseSelectionError) -> FrontendReleaseError {
    let code = match error {
        mpk_vc::ReleaseSelectionError::RegistryAssertion => FrontendReleaseCode::RegistryAssertion,
        mpk_vc::ReleaseSelectionError::BundleUnknown => FrontendReleaseCode::BundleUnknown,
        mpk_vc::ReleaseSelectionError::BundleIncompatible => {
            FrontendReleaseCode::BundleIncompatible
        }
    };
    FrontendReleaseError::new(code, "registered tuple selection failed")
}

fn selected_from_snapshots(
    registry: &ValidatedReleaseRegistry,
    resolved: mpk_vc::ResolvedRelease<'_>,
    snapshots: &BTreeMap<String, Arc<BundleSnapshot>>,
) -> Result<SelectedFrontendRelease, FrontendReleaseError> {
    let frontend_snapshot = snapshots
        .get(&resolved.frontend.bundle_id)
        .cloned()
        .ok_or_else(|| {
            FrontendReleaseError::new(
                FrontendReleaseCode::BundleInvalid,
                "selected frontend snapshot is absent",
            )
        })?;
    let toolchain_snapshot = snapshots
        .get(&resolved.toolchain.bundle_id)
        .cloned()
        .ok_or_else(|| {
            FrontendReleaseError::new(
                FrontendReleaseCode::BundleInvalid,
                "selected toolchain snapshot is absent",
            )
        })?;
    Ok(SelectedFrontendRelease {
        registry: registry.clone(),
        registry_id: registry.registry().id.clone(),
        registry_sha256: registry.registry_digest().to_hex(),
        frontend: resolved.frontend.clone(),
        toolchain: resolved.toolchain.clone(),
        pointer_width: resolved.release_tuple.pointer_width,
        limit_profile_id: resolved.release_tuple.limit_profile_id.clone(),
        frontend_snapshot,
        toolchain_snapshot,
    })
}

#[cfg(any(test, target_os = "linux"))]
fn raw_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bundle_invalid(detail: &'static str) -> FrontendReleaseError {
    FrontendReleaseError::new(FrontendReleaseCode::BundleInvalid, detail)
}

#[cfg(test)]
pub(crate) struct TestBundleResolver {
    registry: ValidatedReleaseRegistry,
    snapshots: BTreeMap<String, Arc<BundleSnapshot>>,
}

#[cfg(test)]
impl TestBundleResolver {
    pub(crate) fn new(
        registry: ValidatedReleaseRegistry,
        snapshots: BTreeMap<String, BundleSnapshot>,
    ) -> Result<Self, FrontendReleaseError> {
        let expected: BTreeSet<_> = registry
            .registry()
            .frontend_bundles
            .iter()
            .map(|bundle| bundle.bundle_id.clone())
            .chain(
                registry
                    .registry()
                    .toolchain_bundles
                    .iter()
                    .map(|bundle| bundle.bundle_id.clone()),
            )
            .collect();
        if snapshots.keys().cloned().collect::<BTreeSet<_>>() != expected {
            return Err(bundle_invalid("test snapshot set is not exact"));
        }
        for frontend in &registry.registry().frontend_bundles {
            validate_snapshot(&frontend.inventory, &snapshots[&frontend.bundle_id])?;
        }
        for toolchain in &registry.registry().toolchain_bundles {
            validate_snapshot(&toolchain.inventory, &snapshots[&toolchain.bundle_id])?;
        }
        let snapshots = snapshots
            .into_iter()
            .map(|(bundle_id, snapshot)| (bundle_id, Arc::new(snapshot)))
            .collect();
        Ok(Self {
            registry,
            snapshots,
        })
    }

    pub(crate) fn resolve(
        &self,
        request: &ReleaseSelectionRequest,
    ) -> Result<SelectedFrontendRelease, FrontendReleaseError> {
        let resolved = self.registry.resolve(request).map_err(selection_error)?;
        selected_from_snapshots(&self.registry, resolved, &self.snapshots)
    }
}

#[cfg(test)]
pub(crate) fn test_snapshot(
    inventory: &BundleInventory,
    objects: Vec<(String, Vec<u8>, bool)>,
) -> Result<BundleSnapshot, FrontendReleaseError> {
    let files = objects
        .into_iter()
        .map(|(path, bytes, executable)| {
            let sha256 = raw_sha256(&bytes);
            (
                path,
                SnapshotFile {
                    bytes,
                    executable,
                    sha256,
                },
            )
        })
        .collect();
    let snapshot = BundleSnapshot { files };
    validate_snapshot(inventory, &snapshot)?;
    Ok(snapshot)
}

#[cfg(test)]
fn validate_snapshot(
    inventory: &BundleInventory,
    snapshot: &BundleSnapshot,
) -> Result<(), FrontendReleaseError> {
    if inventory.files.len() != snapshot.files.len() {
        return Err(bundle_invalid("snapshot file count differs from inventory"));
    }
    for InventoryFile {
        path,
        executable,
        size_bytes,
        sha256,
    } in &inventory.files
    {
        let file = snapshot
            .file(path)
            .ok_or_else(|| bundle_invalid("snapshot file is missing"))?;
        if file.executable != *executable
            || file.bytes.len() as i64 != *size_bytes
            || file.sha256 != *sha256
            || raw_sha256(&file.bytes) != *sha256
        {
            return Err(bundle_invalid(
                "snapshot file identity differs from inventory",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn embedded_registry_constants_match_the_reviewed_registry() {
        let bytes = include_bytes!("../../../release/bundles/bundle-registry.json");
        let semantic = mpk_vc::semantic_profile_registry::validate_semantic_profile_registry(
            include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
            mpk_vc::semantic_profile_registry::RegistryRevision::Revision2,
        )
        .expect("reviewed semantic registry validates");
        let registry =
            mpk_vc::release_bundle_v1::validate_successor_release_registry(bytes, &semantic)
                .expect("reviewed successor registry validates");
        assert_eq!(registry.registry().id, EXPECTED_REGISTRY_ID);
        assert_eq!(registry.registry_sha256(), EXPECTED_REGISTRY_SHA256_HEX);
        assert_eq!(
            decode_sha256(EXPECTED_REGISTRY_SHA256_HEX),
            EXPECTED_REGISTRY_SHA256
        );
    }

    #[test]
    fn test_resolver_accepts_only_complete_sealed_objects() {
        let vectors: Value = serde_json::from_slice(include_bytes!(
            "../../../develop/specs/vectors/release-bundles-v0.json"
        ))
        .expect("release vectors parse");
        let registry_bytes = canonical_transport(&vectors["fixtures"]["valid_registry"]);
        let registry = validate_release_registry(&registry_bytes).expect("fixture registry");
        let mut snapshots = BTreeMap::new();
        for bundle in vectors["fixtures"]["bundle_bytes"]
            .as_array()
            .expect("bundle bytes")
        {
            let bundle_id = bundle["bundle_id"].as_str().expect("bundle ID");
            let inventory = registry
                .frontend_bundle(bundle_id)
                .map(|descriptor| &descriptor.inventory)
                .or_else(|| {
                    registry
                        .toolchain_bundle(bundle_id)
                        .map(|descriptor| &descriptor.inventory)
                })
                .expect("fixture descriptor");
            let objects = bundle["files"]
                .as_array()
                .expect("bundle files")
                .iter()
                .map(|file| {
                    (
                        file["path"].as_str().expect("path").to_owned(),
                        decode_base64(file["base64"].as_str().expect("base64")),
                        file["mode"] == "0555",
                    )
                })
                .collect();
            snapshots.insert(
                bundle_id.to_owned(),
                test_snapshot(inventory, objects).expect("sealed fixture snapshot"),
            );
        }
        let resolver = TestBundleResolver::new(registry, snapshots).expect("test resolver");
        let selected = resolver
            .resolve(&ReleaseSelectionRequest {
                registry_id: "mpk.release.registry.v0".to_owned(),
                registry_sha256: "47f80ab09e8cde24af73ddc198aef254ff1dbd18c1423a2e7e0ebb69f8c787a7"
                    .to_owned(),
                source_language: "go".to_owned(),
                semantic_profile: "mpk.go.fixed.v0".to_owned(),
                target_id: "linux/amd64".to_owned(),
                frontend_bundle_id: Some("frontend.go.synthetic.v0".to_owned()),
                toolchain_bundle_id: Some("toolchain.go.synthetic.v0".to_owned()),
            })
            .expect("exact synthetic tuple resolves");
        assert_eq!(selected.frontend.main.path, "bin/go2vir");
        assert!(selected.toolchain_snapshot.file("bin/go").is_some());
    }

    fn canonical_transport(value: &Value) -> Vec<u8> {
        let raw = serde_json::to_vec(value).expect("serialize");
        let strict = mpk_vc::parse_strict_json(
            &raw,
            mpk_vc::StrictJsonLimits::new(68 * 1024 * 1024, 68 * 1024 * 1024, 256, 2 * 1024 * 1024),
        )
        .expect("strict");
        let mut bytes = mpk_vc::canonical_json_bytes(&strict).expect("canonical");
        bytes.push(b'\n');
        bytes
    }

    fn decode_base64(input: &str) -> Vec<u8> {
        let mut output = Vec::new();
        let mut quartet = [0u8; 4];
        let mut count = 0usize;
        for byte in input.bytes() {
            quartet[count] = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => 64,
                _ => panic!("invalid base64 fixture"),
            };
            count += 1;
            if count == 4 {
                output.push((quartet[0] << 2) | (quartet[1] >> 4));
                if quartet[2] != 64 {
                    output.push((quartet[1] << 4) | (quartet[2] >> 2));
                }
                if quartet[3] != 64 {
                    output.push((quartet[2] << 6) | quartet[3]);
                }
                count = 0;
            }
        }
        assert_eq!(count, 0);
        output
    }

    fn decode_sha256(value: &str) -> [u8; 32] {
        let mut output = [0_u8; 32];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            output[index] = (hex(pair[0]) << 4) | hex(pair[1]);
        }
        output
    }

    fn hex(value: u8) -> u8 {
        match value {
            b'0'..=b'9' => value - b'0',
            b'a'..=b'f' => value - b'a' + 10,
            _ => panic!("non-hexadecimal registry digest"),
        }
    }
}
