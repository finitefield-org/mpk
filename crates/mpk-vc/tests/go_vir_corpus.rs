use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::decode_canonical_certificate;
use mpk_vc::semantic_profile_registry::{
    validate_compiled_profile_envelope, validate_semantic_profile_registry, ProfileContractField,
    RegistryRevision, ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, SuccessorSourceManifestStage,
    SuccessorSourceManifestValidationContext, SuccessorSourceMapValidationContext,
};
use mpk_vc::successor_vc::{emit_successor_vc_skeleton, generate_successor_vc, SuccessorVcSource};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, sha256_raw_file_bytes, CapturedInput, InputKind,
    ReleaseRegistryIdentity, SourceReference, StrictJsonLimits, SyntheticPermission,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const UPDATE_ENV: &str = "MPK_UPDATE_GO_VIR_CORPUS";
const FRONTEND_INDEX: &str = "fixtures/vir-go/frontend-index.json";
const SHARED_ROOT: &str = "fixtures/vir-go";
const JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontendIndex {
    schema: String,
    update_command: String,
    deterministic_runs: u64,
    semantic_context: Value,
    release_registry: ReleaseRegistryIdentity,
    positive_source_count: u64,
    negative_source_count: u64,
    cases: Vec<FrontendCase>,
    negative_cases: Vec<NegativeCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontendCase {
    id: String,
    source_root: String,
    source_path: String,
    selection: Value,
    function_count: u64,
    frontend_status: String,
    artifacts: Vec<FrontendArtifact>,
    example_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NegativeCase {
    id: String,
    source_root: String,
    source_path: String,
    outcome: String,
    phase: String,
    code: String,
    message: String,
    artifact: FrontendArtifact,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrontendArtifact {
    kind: String,
    path: String,
    sha256: String,
    bytes: u64,
}

#[derive(Clone)]
struct DerivedArtifacts {
    vc: Vec<u8>,
    skeleton: Vec<u8>,
    source_ir_hash: String,
    source_manifest_hash: String,
    input_set_hash: String,
    vc_hash: String,
    function_count: usize,
    member_count: usize,
    group_count: usize,
}

#[derive(Debug, Serialize)]
struct DerivedIndex {
    schema: &'static str,
    update_command: &'static str,
    deterministic_runs: u64,
    cases: Vec<DerivedIndexEntry>,
}

#[derive(Debug, Serialize)]
struct DerivedIndexEntry {
    id: String,
    source_ir_hash: String,
    source_manifest_hash: String,
    input_set_hash: String,
    vc_hash: String,
    function_count: usize,
    member_count: usize,
    group_count: usize,
    artifacts: Vec<CorpusArtifact>,
}

#[derive(Clone, Debug, Serialize)]
struct CorpusArtifact {
    kind: String,
    path: String,
    sha256: String,
    bytes: usize,
}

struct OwnedCapturedInput {
    kind: InputKind,
    normalized_path: String,
    bytes: Vec<u8>,
}

#[test]
fn regenerated_go_successor_corpus_is_linked_deterministic_and_active() {
    let root = repo_root();
    let index: FrontendIndex = read_json(&root.join(FRONTEND_INDEX));
    validate_frontend_index(&root, &index);
    let registry = semantic_registry(&root);

    let mut derived_entries = Vec::new();
    let mut all_artifacts = indexed_frontend_artifacts(&root, &index);
    for corpus_case in &index.cases {
        if !owns_vc_fixture(corpus_case) {
            continue;
        }
        let first = derive_case(&root, corpus_case, &registry);
        let second = derive_case(&root, corpus_case, &registry);
        assert_derived_equal(&corpus_case.id, &first, &second);

        let base = format!("derived/{}", corpus_case.id);
        let outputs = [
            ("vc_v2", "vc.json", first.vc.as_slice()),
            (
                "grouped_skeleton_v2",
                "vc-skeleton.json",
                first.skeleton.as_slice(),
            ),
        ];
        let mut artifacts = Vec::new();
        for (kind, name, bytes) in outputs {
            let path = format!("{base}/{name}");
            assert_corpus_fixture(&root, &path, bytes);
            let descriptor = artifact(kind, &path, bytes);
            artifacts.push(descriptor.clone());
            all_artifacts.push(descriptor);
            if let Some(example) = &corpus_case.example_path {
                let example_name = if name == "vc-skeleton.json" {
                    "vc_skeleton.json"
                } else {
                    name
                };
                assert_fixture(&root.join(example).join(example_name), bytes);
            }
        }

        if corpus_case.id == "alpha-branch" {
            for (name, bytes) in [
                ("vc.json", first.vc.as_slice()),
                ("vc_skeleton.json", first.skeleton.as_slice()),
            ] {
                assert_fixture(&root.join("fixtures/vc-alpha").join(name), bytes);
            }
            let alpha_manifest = canonical(&json!({
                "schema_version": "mpk.vc_alpha_manifest.v2",
                "source": {
                    "frontend_case": corpus_case.id,
                    "frontend_index": FRONTEND_INDEX,
                    "function_count": first.function_count,
                    "source_ir_hash": first.source_ir_hash
                },
                "artifacts": {
                    "vc": {
                        "path": "vc.json",
                        "sha256": sha256(&first.vc),
                        "member_count": first.member_count
                    },
                    "skeleton": {
                        "path": "vc_skeleton.json",
                        "sha256": sha256(&first.skeleton),
                        "group_count": first.group_count
                    }
                }
            }));
            let alpha_path = "derived/alpha-branch/vc-alpha-manifest.json";
            assert_corpus_fixture(&root, alpha_path, &alpha_manifest);
            assert_fixture(
                &root.join("fixtures/vc-alpha/manifest.json"),
                &alpha_manifest,
            );
            let descriptor = artifact("vc_alpha_manifest_v2", alpha_path, &alpha_manifest);
            artifacts.push(descriptor.clone());
            all_artifacts.push(descriptor);
        }

        derived_entries.push(DerivedIndexEntry {
            id: corpus_case.id.clone(),
            source_ir_hash: first.source_ir_hash,
            source_manifest_hash: first.source_manifest_hash,
            input_set_hash: first.input_set_hash,
            vc_hash: first.vc_hash,
            function_count: first.function_count,
            member_count: first.member_count,
            group_count: first.group_count,
            artifacts,
        });
    }

    assert_eq!(derived_entries.len(), 11);
    let derived_index = canonical(&DerivedIndex {
        schema: "mpk.go_vir_derived_corpus.v1",
        update_command: "MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus",
        deterministic_runs: 2,
        cases: derived_entries,
    });
    assert_corpus_fixture(&root, "derived-index.json", &derived_index);
    all_artifacts.push(artifact(
        "derived_index_v1",
        "derived-index.json",
        &derived_index,
    ));

    for (kind, path, bytes) in generate_checker_artifacts(&root) {
        assert_corpus_fixture(&root, path, &bytes);
        all_artifacts.push(artifact(kind, path, &bytes));
    }
    all_artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    assert!(all_artifacts
        .windows(2)
        .all(|pair| pair[0].path < pair[1].path));

    let manifest = canonical(&json!({
        "schema": "mpk.go_vir_corpus.v1",
        "status": "active_successor_reviewed_zero_unexplained_differences",
        "generation": {
            "commands": [
                "MPK_UPDATE_GO_CORPUS=1 go test -count=1 -run TestActiveGoCorpus",
                "MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus",
                "python3 scripts/generate-release-report.py --check"
            ],
            "clean_runs": 2,
            "byte_identical": true,
            "compatibility_aliases": false,
            "active_release_uses_successor_vir": true
        },
        "coverage": {
            "positive_frontend_roots": index.positive_source_count,
            "negative_frontend_roots": index.negative_source_count,
            "vc_fixture_roots": 11,
            "frontend_only_aggregate_roots": ["alpha-array", "basic-structarray"],
            "payment_policies": 5
        },
        "checker_audit": {
            "certificate": "checker/one-theorem.hex",
            "source_free": "accepted",
            "reference": "accepted",
            "hash_agreement": true,
            "axiom_count": 0
        },
        "artifacts": all_artifacts,
        "unresolved_dispositions": []
    }));
    assert_corpus_fixture(&root, "manifest.json", &manifest);
    assert_no_unintended_leakage(&root, &manifest);

    for retired in ["policy", "ai"] {
        let path = root.join(SHARED_ROOT).join(retired);
        assert!(
            !path.exists()
                || fs::read_dir(&path)
                    .expect("inspect retired fixture subtree")
                    .next()
                    .is_none(),
            "predecessor helper fixture subtree remains active: {retired}"
        );
    }
}

fn validate_frontend_index(root: &Path, index: &FrontendIndex) {
    assert_eq!(index.schema, "mpk.go_vir_frontend_corpus.v1");
    assert_eq!(index.deterministic_runs, 2);
    assert_eq!(index.positive_source_count as usize, index.cases.len());
    assert_eq!(
        index.negative_source_count as usize,
        index.negative_cases.len()
    );
    assert!(index.update_command.contains("MPK_UPDATE_GO_CORPUS=1"));
    assert_eq!(index.cases.len(), 13);
    assert_eq!(index.negative_cases.len(), 8);
    assert_eq!(index.semantic_context["source_language"], "go");
    assert_eq!(
        index.semantic_context["semantic_profile"],
        "mpk.go.fixed.v0"
    );
    assert_eq!(index.release_registry.id, "mpk.release.registry.v1");

    let mut ids = BTreeSet::new();
    let mut examples = 0;
    for corpus_case in &index.cases {
        assert!(ids.insert(corpus_case.id.as_str()));
        assert_eq!(corpus_case.frontend_status, "ir-lowered");
        assert_eq!(
            corpus_case.selection["schema"],
            "mpk.selection.go_function.v0"
        );
        assert!(corpus_case.function_count > 0);
        assert!(root
            .join(&corpus_case.source_root)
            .join(&corpus_case.source_path)
            .is_file());
        assert_eq!(corpus_case.artifacts.len(), 4);
        for descriptor in &corpus_case.artifacts {
            checked_artifact(root, descriptor);
        }
        if let Some(example) = &corpus_case.example_path {
            examples += 1;
            assert!(example.starts_with("examples/"));
            assert_eq!(
                read_json::<Value>(&root.join(example).join("mpk-semantic-context.json")),
                index.semantic_context
            );
            assert_eq!(
                read_json::<Value>(&root.join(example).join("mpk-selection.json")),
                corpus_case.selection
            );
        }
    }
    assert_eq!(examples, 7);

    for negative in &index.negative_cases {
        assert!(ids.insert(negative.id.as_str()));
        assert_eq!(negative.outcome, "rejected");
        assert_eq!(negative.phase, "subset");
        assert!(!negative.code.is_empty());
        assert!(!negative.message.is_empty());
        assert!(root
            .join(&negative.source_root)
            .join(&negative.source_path)
            .is_file());
        checked_artifact(root, &negative.artifact);
    }
}

fn checked_artifact(root: &Path, descriptor: &FrontendArtifact) -> Vec<u8> {
    let bytes = fs::read(root.join(SHARED_ROOT).join(&descriptor.path))
        .unwrap_or_else(|error| panic!("read {}: {error}", descriptor.path));
    assert_eq!(bytes.len() as u64, descriptor.bytes);
    assert_eq!(sha256(&bytes), descriptor.sha256);
    assert_no_unintended_leakage(root, &bytes);
    bytes
}

fn owns_vc_fixture(corpus_case: &FrontendCase) -> bool {
    !matches!(corpus_case.id.as_str(), "alpha-array" | "basic-structarray")
}

fn derive_case(
    root: &Path,
    corpus_case: &FrontendCase,
    registry: &ValidatedSemanticProfileRegistry,
) -> DerivedArtifacts {
    let frontend_root = root
        .join(SHARED_ROOT)
        .join("frontend")
        .join(&corpus_case.id);
    let vir_bytes = fs::read(frontend_root.join("vir.json")).expect("successor frontend VIR");
    let map_bytes = fs::read(frontend_root.join("source-map.json")).expect("successor source map");
    let manifest_bytes = fs::read(frontend_root.join("source-manifest.frontend.json"))
        .expect("successor frontend manifest");
    let manifest_value: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    let map_value: Value = serde_json::from_slice(&map_bytes).expect("source-map JSON");
    let storage = captured_storage(root, corpus_case, &manifest_value);
    let captured = captured_refs(&storage);
    let permissions = synthetic_permissions(&map_value);
    let vir = import_successor_vir_json(&vir_bytes, registry).expect("successor VIR imports");
    let source_map = import_successor_source_map_json(
        &map_bytes,
        SuccessorSourceMapValidationContext {
            registry,
            vir: &vir,
            captured_inputs: &captured,
            synthetic_permissions: &permissions,
        },
    )
    .expect("successor source map imports");
    let release_registry = serde_json::from_value::<ReleaseRegistryIdentity>(
        manifest_value["release_registry"].clone(),
    )
    .expect("release-registry identity");
    let manifest = import_successor_source_manifest_json(
        &manifest_bytes,
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry,
            vir: &vir,
            source_map: &source_map,
            captured_inputs: &captured,
            expected_release_registry: &release_registry,
        },
    )
    .expect("successor source manifest imports");
    let contract = go_vc_contract();
    validate_compiled_profile_envelope(registry, &contract, ProfileContractField::Vc)
        .expect("compiled Go VC contract");
    let source = SuccessorVcSource {
        registry,
        vir: &vir,
        manifest: &manifest,
        profile_contract: &contract,
    };
    let vc = generate_successor_vc(source).expect("successor VIR generates successor VC");
    let skeleton = emit_successor_vc_skeleton(&vc, source).expect("successor skeleton emits");
    let document = vc.document();
    let function_count = document.functions().len();
    let member_count = document
        .functions()
        .iter()
        .map(|function| function.members.len())
        .sum();
    let group_count = document
        .functions()
        .iter()
        .map(|function| function.groups.len())
        .sum();
    DerivedArtifacts {
        vc: vc.canonical_bytes().to_vec(),
        skeleton: skeleton.canonical_bytes().to_vec(),
        source_ir_hash: document.source_ir_hash().as_str().to_owned(),
        source_manifest_hash: document.source_manifest_hash().as_str().to_owned(),
        input_set_hash: document.input_set_hash().as_str().to_owned(),
        vc_hash: document.vc_hash().as_str().to_owned(),
        function_count,
        member_count,
        group_count,
    }
}

fn go_vc_contract() -> Value {
    json!({
        "profile_entry_sha256":"b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96",
        "contract_id":"mpk.profile.vc.go_fixed.v0",
        "value":{
            "contract_profile_id":"mpk.go.contract.v0",
            "required_check_profile_id":"mpk.go.fixed.v0",
            "verification_limit_profile_id":"mpk.verify.limits.v0"
        }
    })
}

fn captured_storage(
    root: &Path,
    corpus_case: &FrontendCase,
    manifest: &Value,
) -> Vec<OwnedCapturedInput> {
    manifest["inputs"]
        .as_array()
        .expect("manifest inputs")
        .iter()
        .map(|input| {
            let kind: InputKind =
                serde_json::from_value(input["kind"].clone()).expect("known input kind");
            let normalized_path = input["normalized_path"]
                .as_str()
                .expect("normalized path")
                .to_owned();
            let path = root.join(&corpus_case.source_root).join(&normalized_path);
            let bytes = if kind == InputKind::Lockfile && !path.exists() {
                Vec::new()
            } else {
                fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
            };
            assert_eq!(bytes.len() as u64, input["size_bytes"]);
            assert_eq!(sha256_raw_file_bytes(&bytes).to_hex(), input["sha256"]);
            OwnedCapturedInput {
                kind,
                normalized_path,
                bytes,
            }
        })
        .collect()
}

fn captured_refs(storage: &[OwnedCapturedInput]) -> Vec<CapturedInput<'_>> {
    storage
        .iter()
        .map(|input| CapturedInput {
            kind: input.kind,
            normalized_path: &input.normalized_path,
            bytes: &input.bytes,
        })
        .collect()
}

fn synthetic_permissions(source_map: &Value) -> Vec<SyntheticPermission> {
    source_map["entries"]
        .as_array()
        .expect("source-map entries")
        .iter()
        .filter(|entry| entry["origin"]["kind"] == "synthetic")
        .map(|entry| {
            let reason = entry["origin"]["reason"]
                .as_str()
                .expect("synthetic reason");
            assert!(matches!(
                reason,
                "go.control_flow_join" | "go.loop_backedge" | "go.implicit_return"
            ));
            SyntheticPermission {
                reference: serde_json::from_value::<SourceReference>(entry["reference"].clone())
                    .expect("synthetic source reference"),
                reason: reason.to_owned(),
            }
        })
        .collect()
}

fn generate_checker_artifacts(root: &Path) -> Vec<(&'static str, &'static str, Vec<u8>)> {
    let release: Value = read_json(&root.join("release-report.json"));
    let certificate_hex =
        fs::read(root.join("fixtures/cert-basic/one-theorem.hex")).expect("certificate fixture");
    let certificate = decode_canonical_certificate(&decode_hex(&certificate_hex))
        .expect("support certificate is canonical");
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    let checker = canonical(&json!({
        "schema": "mpk.go_vir_checker_audit.v1",
        "certificate_sha256": release["certificates"][0]["expected_hashes"]["certificate"],
        "source_free_checker": release["certificates"][0]["source_free_checker"],
        "reference_checker": release["certificates"][0]["reference_checker"],
        "hash_agreement": release["certificates"][0]["hash_agreement"],
        "unresolved": []
    }));
    let axiom = canonical(&release["certificates"][0]["axiom_report"]);
    vec![
        ("certificate", "checker/one-theorem.hex", certificate_hex),
        ("axiom_report", "checker/axiom-report.json", axiom),
        ("checker_audit_v1", "checker/verdicts.json", checker),
    ]
}

fn indexed_frontend_artifacts(root: &Path, index: &FrontendIndex) -> Vec<CorpusArtifact> {
    let mut artifacts = index
        .cases
        .iter()
        .flat_map(|corpus_case| &corpus_case.artifacts)
        .chain(index.negative_cases.iter().map(|case| &case.artifact))
        .map(|descriptor| CorpusArtifact {
            kind: descriptor.kind.clone(),
            path: descriptor.path.clone(),
            sha256: descriptor.sha256.clone(),
            bytes: descriptor.bytes as usize,
        })
        .collect::<Vec<_>>();
    let bytes = fs::read(root.join(FRONTEND_INDEX)).expect("frontend index");
    artifacts.push(artifact("frontend_index_v1", "frontend-index.json", &bytes));
    artifacts
}

fn assert_derived_equal(id: &str, left: &DerivedArtifacts, right: &DerivedArtifacts) {
    assert_eq!(left.vc, right.vc, "{id} VC changed between clean runs");
    assert_eq!(
        left.skeleton, right.skeleton,
        "{id} skeleton changed between clean runs"
    );
    assert_eq!(left.source_ir_hash, right.source_ir_hash);
    assert_eq!(left.source_manifest_hash, right.source_manifest_hash);
    assert_eq!(left.input_set_hash, right.input_set_hash);
    assert_eq!(left.vc_hash, right.vc_hash);
    assert_eq!(left.function_count, right.function_count);
    assert_eq!(left.member_count, right.member_count);
    assert_eq!(left.group_count, right.group_count);
}

fn assert_corpus_fixture(root: &Path, relative: &str, bytes: &[u8]) {
    assert_fixture(&root.join(SHARED_ROOT).join(relative), bytes);
    assert_no_unintended_leakage(root, bytes);
}

fn assert_fixture(path: &Path, bytes: &[u8]) {
    if env::var_os(UPDATE_ENV).is_some() {
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
        fs::write(path, bytes).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    } else {
        assert_eq!(
            fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
            bytes,
            "fixture {} is stale; rerun the explicit update command",
            path.display()
        );
    }
}

fn assert_no_unintended_leakage(root: &Path, bytes: &[u8]) {
    let root_text = root.to_string_lossy();
    let temporary = env::temp_dir();
    let temporary_text = temporary.to_string_lossy();
    for forbidden in [
        root_text.as_bytes(),
        temporary_text.as_bytes(),
        br#"\"timestamp\""#,
        br#"\"generated_at\""#,
        br#"\"hostname\""#,
    ] {
        assert!(
            !bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "generated artifact leaks a local, host, or temporal value"
        );
    }
}

fn artifact(kind: &str, path: &str, bytes: &[u8]) -> CorpusArtifact {
    CorpusArtifact {
        kind: kind.to_owned(),
        path: path.to_owned(),
        sha256: sha256(bytes),
        bytes: bytes.len(),
    }
}

fn semantic_registry(root: &Path) -> ValidatedSemanticProfileRegistry {
    validate_semantic_profile_registry(
        &fs::read(root.join("release/bundles/semantic-profile-registry.json"))
            .expect("semantic registry"),
        RegistryRevision::Revision2,
    )
    .expect("active revision-2 semantic registry")
}

fn canonical(value: &impl Serialize) -> Vec<u8> {
    let transport = serde_json::to_vec(value).expect("serialize JSON fixture");
    let strict = parse_strict_json(&transport, JSON_LIMITS).expect("strict JSON fixture");
    canonical_json_bytes(&strict).expect("canonical JSON fixture")
}

fn decode_hex(input: &[u8]) -> Vec<u8> {
    let compact = input
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert!(compact.iter().all(u8::is_ascii_hexdigit));
    assert_eq!(compact.len() % 2, 0);
    compact
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII pair"), 16)
                .expect("valid hex")
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
