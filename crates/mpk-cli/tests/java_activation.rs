use mpk_cli::successor_release_bundle::{
    successor_release_registry_hash, validate_successor_bundle_candidate,
    validate_successor_release_registry, SuccessorReleaseSelectionRequest,
    ACTIVE_RELEASE_REGISTRY_SHA256,
};
use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, ProfileContractField,
    RegistryRevision,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SEMANTIC_REGISTRY_SHA256: &str =
    "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557";
const REFERENCE_CHECKER_SHA256: &str =
    "e20c1e4022245968a56a36214b4251ea06ce48732538d8a5895144a3c0f21728";
const LANGUAGES: [&str; 4] = ["csharp", "go", "java", "rust"];
const CONTRACT_FIELDS: [ProfileContractField; 9] = [
    ProfileContractField::Ai,
    ProfileContractField::Evidence,
    ProfileContractField::Frontend,
    ProfileContractField::Manifest,
    ProfileContractField::Policy,
    ProfileContractField::Release,
    ProfileContractField::SourceMap,
    ProfileContractField::Vc,
    ProfileContractField::Vir,
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(root().join(relative)).expect("read repository input")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative)).expect("parse repository JSON")
}

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("serialize JSON");
    bytes.push(b'\n');
    bytes
}

fn members(value: &Value, field: &str) -> BTreeSet<String> {
    value[field]
        .as_array()
        .expect("release collection")
        .iter()
        .map(|member| serde_json::to_string(member).expect("serialize release member"))
        .collect()
}

#[test]
fn activation_vector_is_owned_and_revision_3_is_the_only_installed_registry() {
    let vectors = load("develop/specs/vectors/semantic-profile-registry-v3.json");
    let cases = vectors["activation_cases"]
        .as_array()
        .expect("activation cases")
        .iter()
        .map(|case| {
            (
                case["id"].as_str().expect("activation case id"),
                case["result"].as_str().expect("activation result"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cases,
        [
            ("activation.membership_is_inactive", "inactive"),
            ("activation.no_java_tuple", "reject"),
            ("activation.old_root", "reject"),
            ("activation.no_partial_release", "reject"),
            ("activation.no_dual_registry", "reject"),
            ("activation.certificate_unchanged", "reject"),
            ("activation.no_dynamic_contract", "reject"),
            ("activation.atomic_release", "eligible"),
        ]
    );

    let active_bytes = read("release/bundles/semantic-profile-registry.json");
    let active = validate_semantic_profile_registry(&active_bytes, RegistryRevision::Revision3)
        .expect("active Revision 3 registry");
    assert_eq!(active.revision(), RegistryRevision::Revision3);
    assert_eq!(
        active.identity().registry_sha256(),
        SEMANTIC_REGISTRY_SHA256
    );
    assert_eq!(active.entries().len(), 4);
    assert!(
        validate_semantic_profile_registry(&active_bytes, RegistryRevision::Revision2).is_err()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&active_bytes).expect("active registry JSON"),
        vectors["registry"]
    );
    assert!(!root()
        .join("release/build-inputs/java/bundle-registry.json")
        .exists());
}

#[test]
fn active_release_is_the_exact_union_of_four_candidates_and_five_tuples() {
    let semantic = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let registry_bytes = read("release/bundles/bundle-registry.json");
    let registry = validate_successor_release_registry(&registry_bytes, &semantic)
        .expect("active four-language release");
    assert_eq!(registry.registry_sha256(), ACTIVE_RELEASE_REGISTRY_SHA256);
    let registry_value: Value = serde_json::from_slice(&registry_bytes).expect("release JSON");
    assert_eq!(
        registry_value["execution_host_profiles"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        registry_value["native_runtime_layout_profiles"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        registry_value["frontend_bundles"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        registry_value["toolchain_bundles"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(registry_value["tuples"].as_array().unwrap().len(), 5);

    for field in [
        "execution_host_profiles",
        "native_runtime_layout_profiles",
        "frontend_bundles",
        "toolchain_bundles",
        "tuples",
    ] {
        let mut union = BTreeSet::new();
        for language in LANGUAGES {
            let path = format!("release/bundles/candidates/{language}.json");
            let bytes = read(&path);
            validate_successor_bundle_candidate(&bytes, &semantic)
                .unwrap_or_else(|error| panic!("{language} candidate: {error}"));
            let candidate: Value = serde_json::from_slice(&bytes).expect("candidate JSON");
            union.extend(members(&candidate, field));
        }
        assert_eq!(union, members(&registry_value, field), "{field}");
    }

    for tuple in registry_value["tuples"].as_array().unwrap() {
        registry
            .resolve(
                &semantic,
                SuccessorReleaseSelectionRequest {
                    semantic_context: &tuple["semantic_context"],
                    frontend_bundle_id: tuple["frontend_bundle_id"].as_str().unwrap(),
                    toolchain_bundle_id: tuple["toolchain_bundle_id"].as_str().unwrap(),
                },
            )
            .expect("every installed tuple resolves");
    }
}

#[test]
fn old_crossed_and_partial_release_contexts_reject_atomically() {
    let semantic = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let mut registry_value = load("release/bundles/bundle-registry.json");
    let java_index = registry_value["tuples"]
        .as_array()
        .unwrap()
        .iter()
        .position(|tuple| tuple["semantic_context"]["source_language"] == "java")
        .expect("Java tuple");
    let mut old_context = registry_value["tuples"][java_index]["semantic_context"].clone();
    old_context["profile_registry"] =
        serde_json::to_value(RegistryRevision::Revision2.identity()).unwrap();
    assert!(validate_registry_semantic_context(&semantic, &old_context).is_err());

    registry_value["tuples"][java_index]["semantic_context"] = old_context;
    registry_value["registry_sha256"] =
        json!(successor_release_registry_hash(&registry_value).expect("repair release hash"));
    assert!(
        validate_successor_release_registry(&canonical_line(&registry_value), &semantic).is_err()
    );

    let v2 = load("develop/specs/vectors/semantic-profile-registry-v2.json");
    let v2_bytes = canonical_line(&v2["registry"]);
    assert!(validate_semantic_profile_registry(&v2_bytes, RegistryRevision::Revision3).is_err());
}

#[test]
fn java_uses_all_nine_compiled_contracts_and_certificate_v0_is_unchanged() {
    let semantic = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let value = load("release/bundles/semantic-profile-registry.json");
    for entry in value["profiles"].as_array().expect("semantic profiles") {
        assert_eq!(entry["contracts"].as_object().unwrap().len(), 9);
    }
    let java = load("develop/specs/vectors/java-profile-v0.json");
    let contracts = java["profile_contracts"]
        .as_array()
        .expect("Java compiled contract envelopes");
    assert_eq!(contracts.len(), CONTRACT_FIELDS.len());
    for (envelope, field) in contracts.iter().zip(CONTRACT_FIELDS) {
        assert_eq!(envelope["field"], field.as_str());
        validate_compiled_profile_envelope(&semantic, &envelope["envelope"], field)
            .unwrap_or_else(|error| panic!("Java {}: {error}", field.as_str()));
    }

    assert_eq!(mpk_cert::encode::CERT_MAGIC, b"MPKCERT");
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(read("release/checkers/mpk-checker-ref-linux-amd64"))
        ),
        REFERENCE_CHECKER_SHA256
    );
}

#[test]
fn checked_in_java_example_is_a_revision_3_selection_owned_request() {
    let registry = validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active Revision 3 registry");
    let context_value = load("examples/java-payment-policy/mpk-semantic-context.json");
    let selection_value = load("examples/java-payment-policy/mpk-selection.json");
    let context = validate_registry_semantic_context(&registry, &context_value)
        .expect("Java example semantic context");
    validate_registry_selection_envelope(&registry, &context, &selection_value)
        .expect("Java example selection");

    assert_eq!(context.source_language(), "java");
    assert_eq!(context.semantic_profile(), "mpk.java.scalar.v0");
    assert_eq!(
        canonical_line(&context_value),
        read("examples/java-payment-policy/mpk-semantic-context.json")
    );
    assert_eq!(
        canonical_line(&selection_value),
        read("examples/java-payment-policy/mpk-selection.json")
    );
    assert_eq!(
        selection_value["value"]["contracts"],
        json!(["contracts/approved-reserve.json"])
    );
    assert_eq!(
        selection_value["value"]["sources"],
        json!(["src/payment/Policy.java"])
    );
    assert_eq!(
        selection_value["value"]["methods"],
        json!(["payment.Policy::approvedReserve(boolean,int,int)->int"])
    );

    let contract = load("examples/java-payment-policy/contracts/approved-reserve.json");
    assert_eq!(
        canonical_line(&contract),
        read("examples/java-payment-policy/contracts/approved-reserve.json")
    );
    assert_eq!(contract["schema"], "mpk.java.contract.v0");
    assert_eq!(contract["semantic_profile"], "mpk.java.scalar.v0");
    assert_eq!(contract["method"], selection_value["value"]["methods"][0]);

    let source = String::from_utf8(read("examples/java-payment-policy/src/payment/Policy.java"))
        .expect("Java example source UTF-8");
    assert!(source.contains("package payment;"));
    assert!(source.contains(
        "public static int approvedReserve(boolean approved, int requested, int fallback)"
    ));
}
