#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mpk_cli::policy_schema::{
    import_policy_evidence_v1_for_consumer, import_policy_scan_v1_json, PolicyScanLinkageContext,
    PolicyScanV1, PolicyValidationError, ValidatedPolicyEvidenceV1, ValidatedPolicyScanV1,
};

const MAX_FUZZ_INPUT: usize = 1_048_576;
const SCAN_VECTOR: &[u8] = include_bytes!("../../develop/specs/vectors/policy-scan-v1.json");

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    let first_scan = import_policy_scan_v1_json(data, scan_context());
    let second_scan = import_policy_scan_v1_json(data, scan_context());
    assert_eq!(scan_signature(&first_scan), scan_signature(&second_scan));

    let first_evidence = import_policy_evidence_v1_for_consumer(data);
    let second_evidence = import_policy_evidence_v1_for_consumer(data);
    assert_eq!(
        evidence_signature(&first_evidence),
        evidence_signature(&second_evidence)
    );
});

fn scan_context() -> &'static PolicyScanLinkageContext {
    static CONTEXT: OnceLock<PolicyScanLinkageContext> = OnceLock::new();
    CONTEXT.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(SCAN_VECTOR).expect("tracked policy scan vector parses");
        let fixture: PolicyScanV1 = serde_json::from_value(vector["fixtures"][0]["input"].clone())
            .expect("tracked policy scan fixture parses");
        PolicyScanLinkageContext {
            frontend_status: fixture.frontend_status,
            frontend_phase: fixture.frontend_phase,
            source_language: fixture.source_language,
            semantic_profile: fixture.semantic_profile,
            semantic_parameters: fixture.semantic_parameters,
            selection: fixture.selection,
            release_registry: fixture.release_registry,
            frontend: fixture.frontend,
            toolchain: fixture.toolchain,
            rejected_features: fixture.rejected_features,
            diagnostics: fixture.diagnostics,
            limit_profile: fixture.limit_profile,
            frontend_source_manifest_hash: fixture.frontend_source_manifest_hash,
            input_set_hash: fixture.input_set_hash,
            source_map_hash: fixture.source_map_hash,
            source_ir_schema: fixture.source_ir_schema,
            source_ir_hash: fixture.source_ir_hash,
            helper_artifacts: fixture.helper_artifacts,
        }
    })
}

fn scan_signature(
    result: &Result<ValidatedPolicyScanV1, PolicyValidationError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(value) => (
            true,
            value.document().readiness.clone(),
            String::new(),
            value.canonical_bytes().len(),
        ),
        Err(error) => (
            false,
            error.phase().as_str().to_owned(),
            error.code().to_owned(),
            0,
        ),
    }
}

fn evidence_signature(
    result: &Result<ValidatedPolicyEvidenceV1, PolicyValidationError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(value) => (
            true,
            value.document().vc_hash.clone(),
            String::new(),
            value.canonical_bytes().len(),
        ),
        Err(error) => (
            false,
            error.phase().as_str().to_owned(),
            error.code().to_owned(),
            0,
        ),
    }
}
