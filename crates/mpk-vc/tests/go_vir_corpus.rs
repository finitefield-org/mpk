use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::decode_canonical_certificate;
use mpk_vc::{
    attach_vc_hash, canonical_json_bytes, emit_validated_vc_skeleton_v1, generate_vc_v1,
    import_frontend_source_manifest_json, import_source_map_json, import_vir_json,
    parse_strict_json, validate_release_registry, CapturedInput, InputKind,
    SourceManifestValidationContext, SourceMapValidationContext, SourceReference, StrictJsonLimits,
    SyntheticPermission, ValidatedVcIdentity,
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
    alpha_function_count: u64,
    positive_source_count: u64,
    cases: Vec<FrontendCase>,
    negative_cases: Vec<NegativeCase>,
    semantic_vector: SemanticVector,
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
struct FrontendArtifact {
    kind: String,
    path: String,
    sha256: String,
    bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NegativeCase {
    id: String,
    source_root: String,
    source_path: String,
    expected_code: String,
    actual_code: String,
    outcome: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SemanticVector {
    path: String,
    accepted_cases: u64,
    rejected_cases: u64,
    runtime_checks: u64,
    loops: u64,
    conversions: u64,
    calls: u64,
    contracts: u64,
    unresolved_cases: u64,
}

#[derive(Clone)]
struct DerivedArtifacts {
    vc: Vec<u8>,
    skeleton: Vec<u8>,
    certificate_manifest: Vec<u8>,
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
    input_set_hash: String,
    vc_hash: String,
    certificate_source_manifest_hash: String,
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

#[derive(Debug, Serialize)]
struct CorpusManifest {
    schema: &'static str,
    status: &'static str,
    generation: GenerationAudit,
    coverage: CoverageAudit,
    checker_audit: CheckerAudit,
    artifacts: Vec<CorpusArtifact>,
    unresolved_dispositions: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct GenerationAudit {
    commands: Vec<&'static str>,
    clean_runs: u64,
    byte_identical: bool,
    leakage_scan: &'static str,
    intentional_hash_migration: bool,
    compatibility_aliases: bool,
    active_release_uses_vir: bool,
}

#[derive(Debug, Serialize)]
struct CoverageAudit {
    alpha_functions: u64,
    positive_frontend_roots: u64,
    vc_fixture_roots: u64,
    frontend_only_aggregate_roots: Vec<&'static str>,
    negative_frontend_roots: u64,
    payment_policies: u64,
    loops: u64,
    conversions: u64,
    runtime_operations: u64,
    calls: u64,
    contracts: u64,
}

#[derive(Debug, Serialize)]
struct CheckerAudit {
    certificate: &'static str,
    source_free: &'static str,
    reference: &'static str,
    hash_agreement: bool,
    axiom_count: u64,
}

#[test]
fn regenerated_go_vir_corpus_is_linked_deterministic_and_active() {
    let root = repo_root();
    let index: FrontendIndex = read_json(&root.join(FRONTEND_INDEX));
    validate_frontend_index(&root, &index);
    let registry = valid_registry(&root);

    let mut derived_entries = Vec::new();
    let mut all_artifacts = indexed_frontend_artifacts(&index);
    for corpus_case in &index.cases {
        if !owns_vc_fixture(corpus_case) {
            continue;
        }
        let first = derive_case(&root, corpus_case, &registry);
        let second = derive_case(&root, corpus_case, &registry);
        assert_derived_equal(&corpus_case.id, &first, &second);

        let base = format!("derived/{}", corpus_case.id);
        let outputs = [
            ("vc_v1", "vc.json", first.vc.as_slice()),
            (
                "grouped_skeleton",
                "vc-skeleton.json",
                first.skeleton.as_slice(),
            ),
            (
                "source_manifest_certificate",
                "source-manifest.certificate.json",
                first.certificate_manifest.as_slice(),
            ),
        ];
        let mut artifacts = Vec::new();
        for (kind, name, bytes) in outputs {
            let path = format!("{base}/{name}");
            assert_corpus_fixture(&root, &path, bytes);
            let artifact = artifact(kind, &path, bytes);
            artifacts.push(artifact.clone());
            all_artifacts.push(artifact);
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
            let alpha_vc: Value = serde_json::from_slice(&first.vc).expect("alpha VC JSON");
            for (name, bytes) in [
                ("vc.json", first.vc.as_slice()),
                ("vc_skeleton.json", first.skeleton.as_slice()),
            ] {
                assert_fixture(&root.join("fixtures/vc-alpha").join(name), bytes);
            }
            let alpha_manifest = canonical(&json!({
                "schema_version": "mpk.vc_alpha_manifest.v1",
                "source": {
                    "frontend_case": corpus_case.id,
                    "frontend_index": FRONTEND_INDEX,
                    "function_count": first.function_count,
                    "source_ir_hash": text(&alpha_vc["source_ir_hash"])
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
            let alpha_artifact = artifact("vc_alpha_manifest", alpha_path, &alpha_manifest);
            artifacts.push(alpha_artifact.clone());
            all_artifacts.push(alpha_artifact);
        }
        let vc: Value = serde_json::from_slice(&first.vc).expect("derived VC JSON");
        let certificate_manifest: Value =
            serde_json::from_slice(&first.certificate_manifest).expect("certificate manifest JSON");
        derived_entries.push(DerivedIndexEntry {
            id: corpus_case.id.clone(),
            source_ir_hash: text(&vc["source_ir_hash"]).to_owned(),
            input_set_hash: text(&vc["input_set_hash"]).to_owned(),
            vc_hash: text(&vc["vc_hash"]).to_owned(),
            certificate_source_manifest_hash: text(&certificate_manifest["source_manifest_hash"])
                .to_owned(),
            function_count: first.function_count,
            member_count: first.member_count,
            group_count: first.group_count,
            artifacts,
        });
    }

    let vc_fixture_roots = derived_entries.len() as u64;
    assert_eq!(vc_fixture_roots, 11);
    let derived_index = canonical(&DerivedIndex {
        schema: "mpk.go_vir_derived_corpus.v0",
        update_command: "MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus",
        deterministic_runs: 2,
        cases: derived_entries,
    });
    assert_corpus_fixture(&root, "derived-index.json", &derived_index);
    all_artifacts.push(artifact(
        "derived_index",
        "derived-index.json",
        &derived_index,
    ));

    let support = generate_support_artifacts(&root);
    for (kind, path, bytes) in &support {
        assert_corpus_fixture(&root, path, bytes);
        all_artifacts.push(artifact(kind, path, bytes));
    }
    all_artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    assert!(all_artifacts
        .windows(2)
        .all(|pair| pair[0].path < pair[1].path));
    let manifest = canonical(&CorpusManifest {
        schema: "mpk.go_vir_corpus.v0",
        status: "reviewed_zero_unexplained_differences",
        generation: GenerationAudit {
            commands: vec![
                "MPK_UPDATE_GO_VIR_CORPUS=1 go test -count=1 -run TestRegenerateGoVIRFrontendCorpus",
                "MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus",
                "python3 scripts/generate-release-report.py --check",
            ],
            clean_runs: 2,
            byte_identical: true,
            leakage_scan: "local_path,temp_path,host,timestamp,obsolete_interface",
            intentional_hash_migration: true,
            compatibility_aliases: false,
            active_release_uses_vir: true,
        },
        coverage: CoverageAudit {
            alpha_functions: index.alpha_function_count,
            positive_frontend_roots: index.positive_source_count,
            vc_fixture_roots,
            frontend_only_aggregate_roots: vec!["alpha-array", "basic-structarray"],
            negative_frontend_roots: index.negative_cases.len() as u64,
            payment_policies: 5,
            loops: index.semantic_vector.loops,
            conversions: index.semantic_vector.conversions,
            runtime_operations: index.semantic_vector.runtime_checks,
            calls: index.semantic_vector.calls,
            contracts: index.semantic_vector.contracts,
        },
        checker_audit: CheckerAudit {
            certificate: "checker/one-theorem.hex",
            source_free: "accepted",
            reference: "accepted",
            hash_agreement: true,
            axiom_count: 0,
        },
        artifacts: all_artifacts,
        unresolved_dispositions: Vec::new(),
    });
    assert_corpus_fixture(&root, "manifest.json", &manifest);
    assert_no_unintended_leakage(&root, &manifest);
}

fn validate_frontend_index(root: &Path, index: &FrontendIndex) {
    assert_eq!(index.schema, "mpk.go_vir_frontend_corpus.v0");
    assert_eq!(index.deterministic_runs, 2);
    assert_eq!(index.alpha_function_count, 100);
    assert_eq!(index.positive_source_count as usize, index.cases.len());
    assert!(index.update_command.contains("MPK_UPDATE_GO_VIR_CORPUS=1"));
    assert_eq!(
        index.semantic_vector.path,
        "develop/specs/vectors/go-vir-profile-v0.json"
    );
    assert!(index.semantic_vector.accepted_cases > 0);
    assert!(index.semantic_vector.rejected_cases > 0);
    assert!(index.semantic_vector.runtime_checks > 0);
    assert!(index.semantic_vector.loops > 0);
    assert!(index.semantic_vector.conversions > 0);
    assert!(index.semantic_vector.calls > 0);
    assert!(index.semantic_vector.contracts > 0);
    assert_eq!(index.semantic_vector.unresolved_cases, 0);

    let mut ids = BTreeSet::new();
    let mut alpha_functions = 0;
    let mut payment_policies = 0;
    for corpus_case in &index.cases {
        assert!(ids.insert(corpus_case.id.as_str()));
        assert_eq!(corpus_case.frontend_status, "ir-lowered");
        assert!(corpus_case.selection.is_object());
        assert!(root
            .join(&corpus_case.source_root)
            .join(&corpus_case.source_path)
            .is_file());
        if corpus_case.id.starts_with("alpha-") {
            alpha_functions += corpus_case.function_count;
        }
        if corpus_case.id.starts_with("payment-") {
            payment_policies += 1;
        }
        assert_eq!(corpus_case.artifacts.len(), 4);
        for artifact in &corpus_case.artifacts {
            assert!(matches!(
                artifact.kind.as_str(),
                "frontend_envelope" | "vir" | "source_map" | "source_manifest_frontend"
            ));
            let bytes = fs::read(root.join(SHARED_ROOT).join(&artifact.path))
                .unwrap_or_else(|error| panic!("read {}: {error}", artifact.path));
            assert_eq!(bytes.len() as u64, artifact.bytes);
            assert_eq!(sha256(&bytes), artifact.sha256);
            assert_no_unintended_leakage(root, &bytes);
        }
    }
    assert_eq!(alpha_functions, 100);
    assert_eq!(payment_policies, 5);
    assert_eq!(index.negative_cases.len(), 8);
    for negative in &index.negative_cases {
        assert!(!negative.id.is_empty());
        assert!(root
            .join(&negative.source_root)
            .join(&negative.source_path)
            .is_file());
        assert_eq!(negative.outcome, "rejected");
        assert_eq!(negative.actual_code, negative.expected_code);
    }
}

fn owns_vc_fixture(corpus_case: &FrontendCase) -> bool {
    !matches!(corpus_case.id.as_str(), "alpha-array" | "basic-structarray")
}

fn derive_case(
    root: &Path,
    corpus_case: &FrontendCase,
    registry: &mpk_vc::ValidatedReleaseRegistry,
) -> DerivedArtifacts {
    let frontend_root = root
        .join(SHARED_ROOT)
        .join("frontend")
        .join(&corpus_case.id);
    let vir_bytes = fs::read(frontend_root.join("vir.json")).expect("frontend VIR");
    let source_map_bytes = fs::read(frontend_root.join("source-map.json")).expect("source map");
    let manifest_bytes =
        fs::read(frontend_root.join("source-manifest.frontend.json")).expect("frontend manifest");
    let vir = import_vir_json(&vir_bytes).expect("VIR imports");
    let manifest_value: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    let storage = captured_storage(root, corpus_case, &manifest_value);
    let captures = captured_refs(&storage);
    let source_map_value: Value =
        serde_json::from_slice(&source_map_bytes).expect("source-map JSON");
    let synthetic_permissions = source_map_value["entries"]
        .as_array()
        .expect("source-map entries")
        .iter()
        .filter_map(|entry| {
            if entry["origin"]["kind"] != "synthetic" {
                return None;
            }
            let reason = text(&entry["origin"]["reason"]);
            assert_eq!(reason, "go.control_flow_join");
            let reference = serde_json::from_value::<SourceReference>(entry["reference"].clone())
                .expect("synthetic source-map reference");
            assert!(matches!(reference, SourceReference::Terminator { .. }));
            Some(SyntheticPermission {
                reference,
                reason: reason.to_owned(),
            })
        })
        .collect::<Vec<_>>();
    let source_map = import_source_map_json(
        &source_map_bytes,
        SourceMapValidationContext {
            vir: &vir,
            captured_inputs: &captures,
            synthetic_permissions: &synthetic_permissions,
        },
    )
    .expect("source map imports");
    let context = SourceManifestValidationContext {
        vir: &vir,
        source_map: &source_map,
        captured_inputs: &captures,
        release_registry: registry,
        expected_language_configuration: None,
    };
    let frontend_manifest = import_frontend_source_manifest_json(&manifest_bytes, context)
        .expect("frontend manifest imports");
    let vc = generate_vc_v1(&vir, &frontend_manifest).expect("VIR generates VC v1");
    let skeleton = emit_validated_vc_skeleton_v1(&vc).expect("VC emits grouped skeleton");
    let document = vc.document();
    let identity = ValidatedVcIdentity::new(
        document.input_set_hash.clone(),
        document.source_ir_schema.clone(),
        document.source_ir_hash.clone(),
        document.semantic_profile,
        document.semantic_parameters.clone(),
        document.vc_hash.clone(),
    )
    .expect("VC identity validates");
    let certificate_manifest = attach_vc_hash(&manifest_bytes, context, &identity)
        .expect("certificate-stage manifest attaches exact VC identity");
    let function_count = document.functions.len();
    let member_count = document
        .functions
        .iter()
        .map(|function| function.members.len())
        .sum();
    let group_count = document
        .functions
        .iter()
        .map(|function| function.groups.len())
        .sum();
    DerivedArtifacts {
        vc: vc.canonical_bytes().to_vec(),
        skeleton: skeleton.canonical_bytes().to_vec(),
        certificate_manifest: certificate_manifest.canonical_bytes().to_vec(),
        function_count,
        member_count,
        group_count,
    }
}

fn captured_storage(
    root: &Path,
    corpus_case: &FrontendCase,
    manifest: &Value,
) -> Vec<(InputKind, String, Vec<u8>)> {
    manifest["inputs"]
        .as_array()
        .expect("manifest inputs")
        .iter()
        .map(|input| {
            let kind: InputKind =
                serde_json::from_value(input["kind"].clone()).expect("known input kind");
            let normalized = text(&input["normalized_path"]).to_owned();
            let path = root.join(&corpus_case.source_root).join(&normalized);
            let bytes = if normalized == "go.sum" && !path.exists() {
                Vec::new()
            } else {
                fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
            };
            assert_eq!(bytes.len() as u64, input["size_bytes"]);
            assert_eq!(sha256(&bytes), input["sha256"]);
            (kind, normalized, bytes)
        })
        .collect()
}

fn captured_refs(storage: &[(InputKind, String, Vec<u8>)]) -> Vec<CapturedInput<'_>> {
    storage
        .iter()
        .map(|(kind, path, bytes)| CapturedInput {
            kind: *kind,
            normalized_path: path,
            bytes,
        })
        .collect()
}

fn generate_support_artifacts(root: &Path) -> Vec<(&'static str, &'static str, Vec<u8>)> {
    let release: Value = read_json(&root.join("release-report.json"));
    let certificate_hex =
        fs::read(root.join("fixtures/cert-basic/one-theorem.hex")).expect("certificate fixture");
    let certificate_bytes = decode_hex(&certificate_hex);
    let certificate = decode_canonical_certificate(&certificate_bytes)
        .expect("support certificate is canonical and source-free checkable");
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    let checker = canonical(&json!({
        "schema": "mpk.go_vir_checker_audit.v0",
        "certificate_sha256": release["certificates"][0]["expected_hashes"]["certificate"],
        "source_free_checker": release["certificates"][0]["source_free_checker"],
        "reference_checker": release["certificates"][0]["reference_checker"],
        "hash_agreement": release["certificates"][0]["hash_agreement"],
        "unresolved": []
    }));
    let axiom = canonical(&release["certificates"][0]["axiom_report"]);

    let scans: Value = read_json(&root.join("develop/specs/vectors/policy-scan-v1.json"));
    let evidence: Value = read_json(&root.join("develop/specs/vectors/policy-evidence-v1.json"));
    let ai: Value = read_json(&root.join("develop/specs/vectors/ai-explain-v1.json"));
    let api: Value = read_json(&root.join("develop/specs/vectors/ai-api-v1.json"));
    vec![
        ("certificate", "checker/one-theorem.hex", certificate_hex),
        ("axiom_report", "checker/axiom-report.json", axiom),
        ("checker_audit", "checker/verdicts.json", checker),
        (
            "policy_scan_v1",
            "policy/scan.json",
            canonical_transport(&fixture(&scans, "fixtures", "scan.go_identity_ready")["input"]),
        ),
        (
            "policy_evidence_v1",
            "policy/evidence.json",
            canonical_transport(
                &fixture(&evidence, "fixtures", "evidence.go_identity_pending")["input"],
            ),
        ),
        (
            "ai_v1_dry_run",
            "ai/dry-run.json",
            stable_json(&fixture(&ai, "request_fixtures", "request.go_pending.en")),
        ),
        (
            "ai_v1_output",
            "ai/output.json",
            pretty_transport(
                &fixture(&ai, "explanation_fixtures", "explanation.rust_verified.v1")["input"],
            ),
        ),
        (
            "ai_api_v1",
            "ai/api-v1-response.json",
            canonical_transport(
                &fixture(&api, "generate_fixtures", "generate.go_identity")["expected_response"],
            ),
        ),
    ]
}

fn indexed_frontend_artifacts(index: &FrontendIndex) -> Vec<CorpusArtifact> {
    let mut artifacts = index
        .cases
        .iter()
        .flat_map(|corpus_case| &corpus_case.artifacts)
        .map(|value| CorpusArtifact {
            kind: value.kind.clone(),
            path: value.path.clone(),
            sha256: value.sha256.clone(),
            bytes: value.bytes as usize,
        })
        .collect::<Vec<_>>();
    for (kind, path) in [
        ("frontend_index", "frontend-index.json"),
        ("negative_audit", "negative-results.json"),
    ] {
        let bytes = fs::read(repo_root().join(SHARED_ROOT).join(path)).expect("frontend audit");
        artifacts.push(artifact(kind, path, &bytes));
    }
    artifacts
}

fn assert_derived_equal(id: &str, left: &DerivedArtifacts, right: &DerivedArtifacts) {
    assert_eq!(left.vc, right.vc, "{id} VC changed between clean runs");
    assert_eq!(
        left.skeleton, right.skeleton,
        "{id} skeleton changed between clean runs"
    );
    assert_eq!(
        left.certificate_manifest, right.certificate_manifest,
        "{id} certificate manifest changed between clean runs"
    );
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
    for forbidden in [
        root_text.as_bytes(),
        env::temp_dir().to_string_lossy().as_bytes(),
    ] {
        assert!(
            !bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "generated artifact leaks a local or temporary path"
        );
    }
    let mut forbidden_values = vec![
        br#""timestamp""#.as_slice(),
        br#""generated_at""#.as_slice(),
        br#""generatedAt""#.as_slice(),
        br#""hostname""#.as_slice(),
    ];
    let hostname = env::var("HOSTNAME").ok();
    if let Some(hostname) = hostname.as_deref().filter(|value| !value.is_empty()) {
        forbidden_values.push(hostname.as_bytes());
    }
    for forbidden in forbidden_values {
        assert!(
            !bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "generated artifact leaks host, timestamp, or obsolete interface text"
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

fn valid_registry(root: &Path) -> mpk_vc::ValidatedReleaseRegistry {
    let vectors: Value = read_json(&root.join("develop/specs/vectors/release-bundles-v0.json"));
    let mut bytes = canonical(&vectors["fixtures"]["valid_registry"]);
    bytes.push(b'\n');
    validate_release_registry(&bytes).expect("release registry validates")
}

fn fixture<'a>(vectors: &'a Value, group: &str, id: &str) -> &'a Value {
    vectors[group]
        .as_array()
        .expect("fixture group")
        .iter()
        .find(|value| value["id"] == id)
        .unwrap_or_else(|| panic!("missing {group} fixture {id}"))
}

fn canonical(value: &impl Serialize) -> Vec<u8> {
    let transport = serde_json::to_vec(value).expect("serialize JSON fixture");
    let strict = parse_strict_json(&transport, JSON_LIMITS).expect("strict JSON fixture");
    canonical_json_bytes(&strict).expect("canonical JSON fixture")
}

fn canonical_transport(value: &impl Serialize) -> Vec<u8> {
    let mut bytes = canonical(value);
    bytes.push(b'\n');
    bytes
}

fn pretty_transport(value: &impl Serialize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize pretty JSON fixture");
    bytes.push(b'\n');
    bytes
}

fn stable_json(value: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialize deterministic JSON fixture")
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

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
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
