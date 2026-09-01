//! Private T07 installed-candidate runner. Included only by its owning test
//! executable; no public CLI/API, feature flag or environment toggle selects it.

use crate::frontend_protocol::FrontendProcessFacts;
use crate::frontend_registry::{BundleSnapshot, InstalledSuccessorRelease};
use crate::frontend_sandbox::{
    launch_java_frontend, launch_java_trace_probe, prepare_release_sandbox, PreparedSandbox,
};
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, AcceptedSuccessorFrontendEnvelope,
    SuccessorFrontendProtocolRequest,
};
use mpk_vc::java_release;
use mpk_vc::release_bundle_v1::{
    validate_successor_release_registry, SuccessorReleaseSelectionRequest,
    ValidatedSuccessorReleaseRegistry,
};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_semantic_profile_registry, RegistryRevision,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::{CapturedInput, ComponentIdentity, InputKind, ToolchainComponent};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JavaRunError {
    Release,
    Sandbox,
    Selection,
    Process,
    Protocol,
    Identity,
}

pub(crate) struct PreparedJavaRun {
    semantic: ValidatedSemanticProfileRegistry,
    release: ValidatedSuccessorReleaseRegistry,
    snapshots: BTreeMap<String, Arc<BundleSnapshot>>,
    sandbox: PreparedSandbox,
}

impl PreparedJavaRun {
    // There is deliberately no caller-selected registry, executable or root.
    // All release I/O and host preparation precede access to source inputs.
    pub(crate) fn open() -> Result<Self, JavaRunError> {
        let installed = InstalledSuccessorRelease::open().map_err(|_| JavaRunError::Release)?;
        let semantic = validate_semantic_profile_registry(
            &installed.semantic_registry_bytes,
            RegistryRevision::Revision3,
        )
        .map_err(|_| JavaRunError::Release)?;
        let release = validate_successor_release_registry(&installed.registry_bytes, &semantic)
            .map_err(|_| JavaRunError::Release)?;
        if release.registry() != java_release::registry() {
            return Err(JavaRunError::Release);
        }
        let expected = release
            .registry()
            .frontend_bundles
            .iter()
            .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory))
            .chain(
                release
                    .registry()
                    .toolchain_bundles
                    .iter()
                    .map(|bundle| (bundle.bundle_id.clone(), &bundle.inventory)),
            )
            .collect();
        let snapshots = installed
            .snapshot_selected_bundles(
                &expected,
                java_release::FRONTEND_ID,
                java_release::TOOLCHAIN_ID,
            )
            .map_err(|_| JavaRunError::Release)?;
        let sandbox =
            prepare_release_sandbox(java_release::HOST_ID).map_err(|_| JavaRunError::Sandbox)?;
        Ok(Self {
            semantic,
            release,
            snapshots,
            sandbox,
        })
    }

    pub(crate) fn run(
        self,
        selection: &Value,
        captured: &[CapturedInput<'_>],
    ) -> Result<AcceptedSuccessorFrontendEnvelope, JavaRunError> {
        let resolved = self
            .release
            .resolve(
                &self.semantic,
                SuccessorReleaseSelectionRequest {
                    semantic_context: &java_release::candidate().tuples[0].semantic_context,
                    frontend_bundle_id: java_release::FRONTEND_ID,
                    toolchain_bundle_id: java_release::TOOLCHAIN_ID,
                },
            )
            .map_err(|_| JavaRunError::Release)?;
        let selection = validate_registry_selection_envelope(
            &self.semantic,
            &resolved.semantic_context,
            selection,
        )
        .map_err(|_| JavaRunError::Selection)?;
        let mut expected = BTreeMap::new();
        for (field, kind) in [
            ("sources", InputKind::Source),
            ("contracts", InputKind::Contract),
        ] {
            for path in selection.value()[field]
                .as_array()
                .ok_or(JavaRunError::Selection)?
            {
                if expected
                    .insert(path.as_str().ok_or(JavaRunError::Selection)?, kind)
                    .is_some()
                {
                    return Err(JavaRunError::Selection);
                }
            }
        }
        let mut observed = BTreeSet::new();
        if expected.len() != captured.len()
            || captured.iter().any(|input| {
                expected.get(input.normalized_path) != Some(&input.kind)
                    || !observed.insert(input.normalized_path)
            })
        {
            return Err(JavaRunError::Selection);
        }
        // Bound parent-side materialization before copying any captured bytes.
        let (mut source_total, mut contract_total) = (0_usize, 0_usize);
        for input in captured {
            let (total, maximum) = if input.kind == InputKind::Source {
                (&mut source_total, 16_777_216)
            } else {
                (&mut contract_total, 8_388_608)
            };
            *total = total
                .checked_add(input.bytes.len())
                .ok_or(JavaRunError::Selection)?;
            if input.bytes.len() > 1_048_576 || *total > maximum {
                return Err(JavaRunError::Selection);
            }
        }
        let plan = java_release::launcher_plan(&self.release, &resolved, &selection)
            .map_err(|_| JavaRunError::Selection)?;
        let frontend = self
            .snapshots
            .get(java_release::FRONTEND_ID)
            .ok_or(JavaRunError::Release)?;
        let toolchain = self
            .snapshots
            .get(java_release::TOOLCHAIN_ID)
            .ok_or(JavaRunError::Release)?;
        let output = launch_java_frontend(self.sandbox, frontend, toolchain, &plan, captured)
            .map_err(|_| JavaRunError::Process)?;
        if output.stream_limit_exceeded {
            return Err(JavaRunError::Protocol);
        }
        let identity = self.release.release_identity();
        let envelope = validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &self.semantic,
                semantic_context: &resolved.semantic_context,
                selection: &selection,
                release_registry: &identity,
                captured_inputs: captured,
                synthetic_permissions: &[],
            },
            FrontendProcessFacts {
                exit_code: output.exit_code,
                signaled: output.signaled,
                stdout: &output.stdout,
                stderr_observed_bytes: output.stderr_observed_bytes,
            },
        )
        .map_err(|_| JavaRunError::Protocol)?;
        // Failure envelopes remain artifact-free. Success must bind exactly the
        // bytes actually selected by the installed candidate registry.
        if let Some(artifacts) = envelope.artifacts() {
            let manifest = artifacts.source_manifest().manifest();
            let expected_components = resolved
                .toolchain
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
                        binary_sha256: binary_sha256.clone(),
                        commit_hash: None,
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
                .collect::<Vec<_>>();
            if manifest.frontend().bundle_id != java_release::FRONTEND_ID
                || manifest.frontend().name != resolved.frontend.name
                || manifest.frontend().version != resolved.frontend.version
                || manifest.frontend().binary_sha256 != resolved.frontend.main.binary_sha256
                || !manifest.frontend().subordinate_binaries.is_empty()
                || manifest.toolchain().bundle_id != java_release::TOOLCHAIN_ID
                || manifest.toolchain().distribution_sha256
                    != resolved.toolchain.distribution_sha256
                || manifest.toolchain().components != expected_components
                || manifest.release_registry() != &identity
            {
                return Err(JavaRunError::Identity);
            }
        }
        Ok(envelope)
    }

    /// Measures the registered JVM's native thread/syscall behavior without
    /// charging ptrace overhead to a source request. The full native cases
    /// separately prove that the same registered launcher lowers source within
    /// its frozen request timeout.
    pub(crate) fn trace_probe(self) -> Result<(), JavaRunError> {
        let frontend = self
            .snapshots
            .get(java_release::FRONTEND_ID)
            .ok_or(JavaRunError::Release)?;
        let toolchain = self
            .snapshots
            .get(java_release::TOOLCHAIN_ID)
            .ok_or(JavaRunError::Release)?;
        let output = launch_java_trace_probe(self.sandbox, frontend, toolchain)
            .map_err(|_| JavaRunError::Process)?;
        if output.exit_code != Some(0)
            || output.signaled
            || output.stdout != b"java2vir 0.1.0 (Temurin 25.0.4.1+1; inactive)\n"
            || output.stderr_observed_bytes != 0
            || output.stream_limit_exceeded
        {
            return Err(JavaRunError::Process);
        }
        Ok(())
    }
}
