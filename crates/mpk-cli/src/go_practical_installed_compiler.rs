//! Private installed SDK authority. This authenticates compiler bytes, not a
//! practical semantic context, foundation descriptor or source/program theorem.
use crate::frontend_registry::{
    BundleSnapshot, InstalledSuccessorRelease, EXPECTED_REGISTRY_SHA256_HEX,
    EXPECTED_SEMANTIC_REGISTRY_SHA256,
};
use mpk_vc::go_practical_type_keys::{
    import_go_source_compiler_inputs, read_go_executable_build_info, GoCompilerFileBuffer,
    GoSourceCompilerInputs, GoSourceExportInputs,
};
use mpk_vc::release_bundle_v1::{
    validate_successor_release_registry, ValidatedSuccessorReleaseRegistry, GO_FRONTEND_BUNDLE_ID,
    GO_TOOLCHAIN_BUNDLE_ID,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{
    BundleInventory, ComponentIdentity, ReleaseRegistryIdentity, ToolchainComponent,
    ToolchainIdentity,
};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstalledGoCompilerError {
    Release,
    Identity,
    Inputs,
}

/// No Deserialize, caller path, expected digest or file-list constructor.
pub(crate) struct InstalledGoCompiler {
    registry: ReleaseRegistryIdentity,
    identity: ToolchainIdentity,
    inventory: BundleInventory,
    snapshot: Arc<BundleSnapshot>,
}

/// Retains the installed authority alongside the independently checked original
/// source/export/compiler owner. It grants no practical-profile proof verdict.
pub(crate) struct InstalledGoSourceCompilerInputs {
    authority: InstalledGoCompiler,
    inputs: GoSourceCompilerInputs,
}

impl InstalledGoSourceCompilerInputs {
    pub(crate) fn compiler_inputs(&self) -> &GoSourceCompilerInputs {
        &self.inputs
    }
    pub(crate) fn release_registry(&self) -> &ReleaseRegistryIdentity {
        &self.authority.registry
    }
}

impl InstalledGoCompiler {
    pub(crate) fn open() -> Result<Self, InstalledGoCompilerError> {
        let installed =
            InstalledSuccessorRelease::open().map_err(|_| InstalledGoCompilerError::Release)?;
        let semantic = validate_semantic_profile_registry(
            &installed.semantic_registry_bytes,
            RegistryRevision::Revision4,
        )
        .map_err(|_| InstalledGoCompilerError::Release)?;
        let registry = validate_successor_release_registry(&installed.registry_bytes, &semantic)
            .map_err(|_| InstalledGoCompilerError::Release)?;
        if semantic.identity().registry_sha256() != EXPECTED_SEMANTIC_REGISTRY_SHA256
            || registry.registry_sha256() != EXPECTED_REGISTRY_SHA256_HEX
        {
            return Err(InstalledGoCompilerError::Release);
        }
        let expected = registry
            .registry()
            .frontend_bundles
            .iter()
            .map(|b| (b.bundle_id.clone(), &b.inventory))
            .chain(
                registry
                    .registry()
                    .toolchain_bundles
                    .iter()
                    .map(|b| (b.bundle_id.clone(), &b.inventory)),
            )
            .collect::<BTreeMap<_, _>>();
        let snapshots = installed
            .snapshot_selected_bundles(&expected, GO_FRONTEND_BUNDLE_ID, GO_TOOLCHAIN_BUNDLE_ID)
            .map_err(|_| InstalledGoCompilerError::Release)?;
        let frontend = registry
            .registry()
            .frontend_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == GO_FRONTEND_BUNDLE_ID)
            .ok_or(InstalledGoCompilerError::Release)?;
        let executable = snapshots
            .get(GO_FRONTEND_BUNDLE_ID)
            .and_then(|snapshot| snapshot.file(&frontend.main.path))
            .ok_or(InstalledGoCompilerError::Identity)?;
        // Build metadata identifies compiler and linked analysis modules only;
        // it does not establish which source generation produced these bytes.
        read_go_executable_build_info(&executable.bytes)
            .and_then(|info| info.validate_analysis_frontend("linux_amd64"))
            .map_err(|_| InstalledGoCompilerError::Identity)?;
        let snapshot = snapshots
            .get(GO_TOOLCHAIN_BUNDLE_ID)
            .cloned()
            .ok_or(InstalledGoCompilerError::Release)?;
        Self::from_verified_snapshot(&registry, snapshot)
    }

    // The production caller above obtains this snapshot only from the installed
    // root. The test-only caller supplies a complete independently sealed copy
    // of the same registered SDK, and does not establish installed-root authority.
    fn from_verified_snapshot(
        registry: &ValidatedSuccessorReleaseRegistry,
        snapshot: Arc<BundleSnapshot>,
    ) -> Result<Self, InstalledGoCompilerError> {
        if registry.registry_sha256() != EXPECTED_REGISTRY_SHA256_HEX {
            return Err(InstalledGoCompilerError::Release);
        }
        let toolchain = registry
            .registry()
            .toolchain_bundles
            .iter()
            .find(|t| t.bundle_id == GO_TOOLCHAIN_BUNDLE_ID)
            .ok_or(InstalledGoCompilerError::Release)?;
        let identity = ToolchainIdentity {
            bundle_id: toolchain.bundle_id.clone(),
            distribution_sha256: toolchain.distribution_sha256.clone(),
            components: toolchain
                .components
                .iter()
                .map(|component| match component {
                    ToolchainComponent::Executable {
                        name,
                        release,
                        binary_sha256,
                        ..
                    } => ComponentIdentity::Executable {
                        name: name.clone(),
                        release: release.clone(),
                        commit_hash: None,
                        binary_sha256: binary_sha256.clone(),
                    },
                    ToolchainComponent::Content {
                        name,
                        release,
                        content_sha256,
                        ..
                    } => ComponentIdentity::Content {
                        name: name.clone(),
                        release: release.clone(),
                        content_sha256: content_sha256.clone(),
                    },
                })
                .collect(),
        };
        Ok(Self {
            registry: registry.release_identity(),
            identity,
            inventory: toolchain.inventory.clone(),
            snapshot,
        })
    }

    #[cfg(test)]
    pub(crate) fn test_original_snapshot(
        registry: &ValidatedSuccessorReleaseRegistry,
        snapshot: Arc<BundleSnapshot>,
    ) -> Result<Self, InstalledGoCompilerError> {
        Self::from_verified_snapshot(registry, snapshot)
    }

    pub(crate) fn identity(&self) -> &ToolchainIdentity {
        &self.identity
    }
    pub(crate) fn inventory(&self) -> &BundleInventory {
        &self.inventory
    }

    /// Consume only the descriptor-relative installed snapshot. No caller SDK
    /// buffers, component list, namespace substitution or compiler invocation.
    pub(crate) fn bind_original_inputs(
        self,
        record: &[u8],
        exports: GoSourceExportInputs,
    ) -> Result<InstalledGoSourceCompilerInputs, InstalledGoCompilerError> {
        let mut files = Vec::with_capacity(self.inventory.files.len());
        for expected in &self.inventory.files {
            let actual = self
                .snapshot
                .file(&expected.path)
                .ok_or(InstalledGoCompilerError::Identity)?;
            let file =
                GoCompilerFileBuffer::import(&expected.path, actual.executable, &actual.bytes)
                    .map_err(|_| InstalledGoCompilerError::Identity)?;
            if file.entry() != expected {
                return Err(InstalledGoCompilerError::Identity);
            }
            files.push(file);
        }
        let inputs = import_go_source_compiler_inputs(record, exports, files)
            .map_err(|_| InstalledGoCompilerError::Inputs)?;
        if inputs.host() != "linux_amd64"
            || inputs.toolchain() != &self.identity
            || inputs.inventory() != &self.inventory
        {
            return Err(InstalledGoCompilerError::Identity);
        }
        Ok(InstalledGoSourceCompilerInputs {
            authority: self,
            inputs,
        })
    }
}
