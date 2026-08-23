use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_vc::{
    canonical_json_bytes, contract_hash, emit_validated_vc_skeleton_v1, generate_program_vcs,
    generate_vc_v1, import_frontend_source_manifest_json, import_source_map_json, import_vir_json,
    parse_strict_json, validate_release_registry, CapturedInput, InputKind, LanguageConfiguration,
    ProgramVcMemberKind, SourceManifestValidationContext, SourceMapValidationContext, SourceOrigin,
    StrictJsonLimits, ValidatedReleaseRegistry, VcGroupKind, VcMemberKind, VirInstruction,
    VirModule,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const UPDATE_ENV: &str = "MPK_UPDATE_RUST_POSITIVE_CORPUS";
const FRONTEND_INDEX: &str = "fixtures/rust-basic/positive/frontend-index.json";
const PUBLIC_ROOT: &str = "fixtures/rust-basic/positive";
const FRONTEND_ROOT: &str = "rust-tools/rust2vir/testdata/positive";
const CATEGORIES: &[&str] = &[
    "boolean_short_circuit",
    "signed_unsigned_max",
    "checked_addition",
    "minimum_literal_negation",
    "division",
    "cross_width_shifts",
    "array_bounds",
    "struct_move",
    "early_return",
    "cross_module_calls",
    "usize_targets",
    "multi_file_closure",
];
const JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrontendIndex {
    accepted_mir: AcceptedMir,
    categories: Vec<String>,
    cases: Vec<FrontendCase>,
    deterministic_runs: u64,
    schema: String,
    update_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptedMir {
    compiler_commit: String,
    dialect_sha256: String,
    dialect_summary: String,
    findings: Vec<Value>,
    profile_id: String,
    query: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrontendCase {
    artifacts: Vec<FrontendArtifact>,
    call_count: usize,
    captured_sources: Vec<String>,
    category: String,
    contracts: Vec<String>,
    diagnostics: Vec<Value>,
    fixture: String,
    frontend_phase: String,
    frontend_status: String,
    function_count: usize,
    id: String,
    instruction_count: usize,
    pointer_width: u32,
    safety_check_count: usize,
    selection: String,
    semantic_profile: String,
    source_map_entries: usize,
    target_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrontendArtifact {
    bytes: usize,
    kind: String,
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DerivedArtifacts {
    vc: Vec<u8>,
    skeleton: Vec<u8>,
    metadata: Vec<u8>,
    source_ir_hash: String,
    input_set_hash: String,
    vc_hash: String,
    function_count: usize,
    member_count: usize,
    property_member_count: usize,
    safety_member_count: usize,
    group_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct CorpusArtifact {
    bytes: usize,
    kind: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct CorpusManifest {
    schema: &'static str,
    status: &'static str,
    deterministic_runs: u64,
    categories: Vec<String>,
    category_count: usize,
    case_count: usize,
    accepted_mir: AcceptedMir,
    certificate_bytes: &'static str,
    generation: GenerationAudit,
    negative_coverage: NegativeCoverageIndex,
    cases: Vec<CorpusCase>,
    findings: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct NegativeCoverageIndex {
    schema: &'static str,
    manifests: [&'static str; 2],
}

#[derive(Debug, Serialize)]
struct GenerationAudit {
    commands: Vec<String>,
    snapshot_compiler_clean_runs: u64,
    downstream_clean_runs: u64,
    byte_identical: bool,
    leakage_scan: &'static str,
}

#[derive(Debug, Serialize)]
struct CorpusCase {
    id: String,
    category: String,
    fixture: String,
    selection: String,
    target_id: String,
    pointer_width: u32,
    captured_source_count: usize,
    function_count: usize,
    member_count: usize,
    property_member_count: usize,
    safety_member_count: usize,
    group_count: usize,
    source_ir_hash: String,
    input_set_hash: String,
    vc_hash: String,
    artifacts: Vec<CorpusArtifact>,
    findings: Vec<Value>,
}

#[test]
fn positive_rust_sources_generate_linked_deterministic_vcs() {
    let root = repo_root();
    if env::var_os(UPDATE_ENV).is_some() {
        sync_tree(&root.join(FRONTEND_ROOT), &root.join(PUBLIC_ROOT));
    }
    let index_bytes = fs::read(root.join(FRONTEND_INDEX)).expect("read Rust frontend index");
    let mirror_index =
        fs::read(root.join(FRONTEND_ROOT).join("frontend-index.json")).expect("mirror index");
    assert_eq!(index_bytes, mirror_index, "frontend index mirror");
    let index: FrontendIndex = serde_json::from_slice(&index_bytes).expect("frontend index JSON");
    assert_eq!(
        index_bytes,
        canonical(&index),
        "frontend index is exact JCS"
    );
    validate_frontend_index(&root, &index);
    let registry = valid_registry(&root);

    let mut corpus_cases = Vec::with_capacity(index.cases.len());
    for corpus_case in &index.cases {
        validate_frontend_artifacts(&root, corpus_case);
        let first = derive_case(&root, corpus_case, &index.accepted_mir, &registry);
        let second = derive_case(&root, corpus_case, &index.accepted_mir, &registry);
        assert_eq!(first, second, "{} downstream clean runs", corpus_case.id);

        let artifact_base = artifact_base(corpus_case);
        let outputs = [
            ("vc_v1", "vc.json", first.vc.as_slice()),
            (
                "certificate_skeleton",
                "vc-skeleton.json",
                first.skeleton.as_slice(),
            ),
            (
                "diagnostic_profile_metadata",
                "metadata.json",
                first.metadata.as_slice(),
            ),
        ];
        let mut artifacts = corpus_case
            .artifacts
            .iter()
            .map(|item| CorpusArtifact {
                bytes: item.bytes,
                kind: item.kind.clone(),
                path: format!("positive/{}", item.path),
                sha256: item.sha256.clone(),
            })
            .collect::<Vec<_>>();
        for (kind, name, bytes) in outputs {
            let relative = format!("{artifact_base}/{name}");
            assert_fixture(&root.join(PUBLIC_ROOT).join(&relative), bytes);
            assert_no_path_leakage(&root, bytes);
            artifacts.push(artifact(kind, &format!("positive/{relative}"), bytes));
        }
        assert_eq!(artifacts.len(), 7);
        corpus_cases.push(CorpusCase {
            id: corpus_case.id.clone(),
            category: corpus_case.category.clone(),
            fixture: corpus_case.fixture.clone(),
            selection: corpus_case.selection.clone(),
            target_id: corpus_case.target_id.clone(),
            pointer_width: corpus_case.pointer_width,
            captured_source_count: corpus_case.captured_sources.len(),
            function_count: first.function_count,
            member_count: first.member_count,
            property_member_count: first.property_member_count,
            safety_member_count: first.safety_member_count,
            group_count: first.group_count,
            source_ir_hash: first.source_ir_hash,
            input_set_hash: first.input_set_hash,
            vc_hash: first.vc_hash,
            artifacts,
            findings: Vec::new(),
        });
    }

    let manifest = canonical(&CorpusManifest {
        schema: "mpk.rust.positive_corpus.v0",
        status: "reviewed_zero_findings",
        deterministic_runs: 2,
        categories: index.categories.clone(),
        category_count: index.categories.len(),
        case_count: corpus_cases.len(),
        accepted_mir: index.accepted_mir.clone(),
        certificate_bytes: "deferred_to_RUST-06",
        generation: GenerationAudit {
            commands: vec![
                index.update_command.clone(),
                "MPK_UPDATE_RUST_POSITIVE_CORPUS=1 cargo test -p mpk-vc --test rust_positive_corpus".to_owned(),
                "./scripts/run-rust2vir-toolchain.sh cargo test --locked --test positive_corpus".to_owned(),
                "cargo test -p mpk-vc --test rust_positive_corpus".to_owned(),
            ],
            snapshot_compiler_clean_runs: 2,
            downstream_clean_runs: 2,
            byte_identical: true,
            leakage_scan: "source_root,toolchain,temp,host,timestamp,unrelated_source",
        },
        negative_coverage: NegativeCoverageIndex {
            schema: "mpk.rust.negative_coverage.index.v0",
            manifests: ["negative/manifest.json", "adversarial/manifest.json"],
        },
        cases: corpus_cases,
        findings: Vec::new(),
    });
    assert_fixture(&root.join("fixtures/rust-basic/manifest.json"), &manifest);
    assert_no_path_leakage(&root, &manifest);
}

fn validate_frontend_index(root: &Path, index: &FrontendIndex) {
    assert_eq!(index.schema, "mpk.rust.positive_frontend_corpus.v0");
    assert_eq!(index.deterministic_runs, 2);
    assert_eq!(index.categories, CATEGORIES);
    assert_eq!(index.cases.len(), 13);
    assert!(index.update_command.contains(UPDATE_ENV));
    assert_eq!(
        index.accepted_mir.compiler_commit,
        "4d08223c054cf5a56d9761ca925fd46ffebe7115"
    );
    assert_eq!(index.accepted_mir.profile_id, "mpk.rust.mir.4d08223c.v0");
    assert_eq!(
        index.accepted_mir.query,
        "mir_drops_elaborated_and_const_checked"
    );
    assert_eq!(
        index.accepted_mir.dialect_sha256,
        "6dd18917a34f886319af0284d9a8a1bd8e9634388c6ea56fbee2c52f05917a80"
    );
    assert!(index
        .accepted_mir
        .dialect_summary
        .contains("statement=Assign"));
    assert!(index
        .accepted_mir
        .dialect_summary
        .contains("rvalue=Use,BinaryOp"));
    assert!(index
        .accepted_mir
        .dialect_summary
        .contains("terminator=Goto"));
    assert!(index.accepted_mir.findings.is_empty());

    let mut ids = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let mut usize_widths = BTreeSet::new();
    for corpus_case in &index.cases {
        assert!(ids.insert(corpus_case.id.as_str()), "duplicate case id");
        categories.insert(corpus_case.category.as_str());
        assert_eq!(corpus_case.frontend_status, "ir-lowered");
        assert_eq!(corpus_case.frontend_phase, "emission");
        assert_eq!(corpus_case.semantic_profile, "mpk.rust.checked.v0");
        assert!(corpus_case.diagnostics.is_empty());
        assert!(!corpus_case.contracts.is_empty());
        assert!(!corpus_case.captured_sources.is_empty());
        assert_eq!(corpus_case.artifacts.len(), 4);
        assert!(corpus_case.function_count > 0);
        assert!(corpus_case.instruction_count > 0);
        assert!(corpus_case.source_map_entries > 0);
        if corpus_case.category == "usize_targets" {
            usize_widths.insert(corpus_case.pointer_width);
        }
        let source_root = root
            .join(PUBLIC_ROOT)
            .join(&corpus_case.fixture)
            .join("source");
        for path in corpus_case
            .contracts
            .iter()
            .chain(corpus_case.captured_sources.iter())
        {
            assert!(
                source_root.join(path).is_file(),
                "missing fixture input {path}"
            );
            assert_eq!(
                fs::read(source_root.join(path)).expect("public fixture input"),
                fs::read(
                    root.join(FRONTEND_ROOT)
                        .join(&corpus_case.fixture)
                        .join("source")
                        .join(path)
                )
                .expect("frontend fixture input"),
                "mirrored fixture input {path}"
            );
        }
    }
    assert_eq!(categories, CATEGORIES.iter().copied().collect());
    assert_eq!(usize_widths, BTreeSet::from([32, 64]));
    assert_eq!(
        fs::read(
            root.join(PUBLIC_ROOT)
                .join("multi-file-closure/source/src/unrelated.rs")
        )
        .expect("public unrelated source"),
        fs::read(
            root.join(FRONTEND_ROOT)
                .join("multi-file-closure/source/src/unrelated.rs")
        )
        .expect("frontend unrelated source")
    );
}

fn validate_frontend_artifacts(root: &Path, corpus_case: &FrontendCase) {
    let mut kinds = BTreeSet::new();
    let base = artifact_base(corpus_case);
    for item in &corpus_case.artifacts {
        assert!(kinds.insert(item.kind.as_str()), "duplicate artifact kind");
        assert_eq!(
            Path::new(&item.path)
                .parent()
                .and_then(Path::to_str)
                .expect("artifact parent"),
            base
        );
        let bytes = fs::read(root.join(PUBLIC_ROOT).join(&item.path))
            .unwrap_or_else(|error| panic!("read {}: {error}", item.path));
        assert_eq!(bytes.len(), item.bytes, "{} byte count", item.path);
        assert_eq!(sha256(&bytes), item.sha256, "{} digest", item.path);
        assert_eq!(
            bytes,
            fs::read(root.join(FRONTEND_ROOT).join(&item.path))
                .unwrap_or_else(|error| panic!("read mirror {}: {error}", item.path)),
            "{} frontend mirror",
            item.path
        );
        assert_no_path_leakage(root, &bytes);
    }
    assert_eq!(
        kinds,
        BTreeSet::from([
            "frontend_envelope",
            "source_manifest_frontend",
            "source_map",
            "vir"
        ])
    );
}

fn derive_case(
    root: &Path,
    corpus_case: &FrontendCase,
    accepted_mir: &AcceptedMir,
    registry: &ValidatedReleaseRegistry,
) -> DerivedArtifacts {
    let envelope_bytes = read_frontend_artifact(root, corpus_case, "frontend_envelope");
    let vir_bytes = read_frontend_artifact(root, corpus_case, "vir");
    let source_map_bytes = read_frontend_artifact(root, corpus_case, "source_map");
    let manifest_bytes = read_frontend_artifact(root, corpus_case, "source_manifest_frontend");
    validate_envelope(
        corpus_case,
        &envelope_bytes,
        &vir_bytes,
        &source_map_bytes,
        &manifest_bytes,
    );

    let vir = import_vir_json(&vir_bytes)
        .unwrap_or_else(|error| panic!("{} VIR import: {error}", corpus_case.id));
    let manifest_value: Value =
        serde_json::from_slice(&manifest_bytes).expect("frontend source manifest JSON");
    let language_configuration: LanguageConfiguration =
        serde_json::from_value(manifest_value["target"]["language_configuration"].clone())
            .expect("Rust language configuration");
    let storage = captured_storage(root, corpus_case, &manifest_value);
    let captures = captured_refs(&storage);
    let source_map = import_source_map_json(
        &source_map_bytes,
        SourceMapValidationContext {
            vir: &vir,
            captured_inputs: &captures,
            synthetic_permissions: &[],
        },
    )
    .unwrap_or_else(|error| panic!("{} source-map import: {error}", corpus_case.id));
    assert_source_map_coverage(corpus_case, &source_map);
    let manifest_context = SourceManifestValidationContext {
        vir: &vir,
        source_map: &source_map,
        captured_inputs: &captures,
        release_registry: registry,
        expected_language_configuration: Some(&language_configuration),
    };
    let frontend_manifest = import_frontend_source_manifest_json(&manifest_bytes, manifest_context)
        .unwrap_or_else(|error| panic!("{} source-manifest import: {error}", corpus_case.id));

    let program = generate_program_vcs(&vir)
        .unwrap_or_else(|error| panic!("{} program VC generation: {error}", corpus_case.id));
    let vc = generate_vc_v1(&vir, &frontend_manifest)
        .unwrap_or_else(|error| panic!("{} VC v1 generation: {error}", corpus_case.id));
    let skeleton = emit_validated_vc_skeleton_v1(&vc)
        .unwrap_or_else(|error| panic!("{} skeleton generation: {error}", corpus_case.id));
    let (property_member_count, safety_member_count) =
        cross_check_program_and_groups(corpus_case, &vir, &program, vc.document(), &skeleton);
    assert!(property_member_count > 0, "{} property VCs", corpus_case.id);
    assert!(safety_member_count > 0, "{} safety VCs", corpus_case.id);

    let document = vc.document();
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
    let metadata = canonical(&json!({
        "accepted_mir": {
            "compiler_commit": accepted_mir.compiler_commit,
            "dialect_sha256": accepted_mir.dialect_sha256,
            "findings": accepted_mir.findings,
            "profile_id": accepted_mir.profile_id,
            "query": accepted_mir.query
        },
        "case": {
            "captured_source_count": corpus_case.captured_sources.len(),
            "category": corpus_case.category,
            "id": corpus_case.id,
            "selection": corpus_case.selection
        },
        "certificate_bytes": "deferred_to_RUST-06",
        "diagnostics": corpus_case.diagnostics,
        "findings": [],
        "frontend": {
            "phase": corpus_case.frontend_phase,
            "status": corpus_case.frontend_status
        },
        "profile": {
            "pointer_width": corpus_case.pointer_width,
            "semantic_parameters": serde_json::to_value(&vir.semantic_parameters).expect("semantic parameters"),
            "semantic_profile": corpus_case.semantic_profile,
            "source_language": "rust",
            "target_id": corpus_case.target_id
        },
        "schema": "mpk.rust.positive_case_metadata.v0",
        "vc": {
            "function_count": document.functions.len(),
            "group_count": group_count,
            "input_set_hash": document.input_set_hash,
            "member_count": member_count,
            "property_member_count": property_member_count,
            "safety_member_count": safety_member_count,
            "source_ir_hash": document.source_ir_hash,
            "vc_hash": document.vc_hash
        }
    }));

    DerivedArtifacts {
        vc: vc.canonical_bytes().to_vec(),
        skeleton: skeleton.canonical_bytes().to_vec(),
        metadata,
        source_ir_hash: document.source_ir_hash.clone(),
        input_set_hash: document.input_set_hash.clone(),
        vc_hash: document.vc_hash.clone(),
        function_count: document.functions.len(),
        member_count,
        property_member_count,
        safety_member_count,
        group_count,
    }
}

fn validate_envelope(
    corpus_case: &FrontendCase,
    envelope_bytes: &[u8],
    vir_bytes: &[u8],
    source_map_bytes: &[u8],
    manifest_bytes: &[u8],
) {
    let envelope: Value = serde_json::from_slice(envelope_bytes).expect("frontend envelope JSON");
    let mut canonical_envelope = canonical(&envelope);
    canonical_envelope.push(b'\n');
    assert_eq!(
        envelope_bytes, canonical_envelope,
        "canonical public envelope"
    );
    assert_eq!(envelope["schema"], "mpk.frontend.cli.v0");
    assert_eq!(envelope["source_language"], "rust");
    assert_eq!(envelope["status"], corpus_case.frontend_status);
    assert_eq!(envelope["phase"], corpus_case.frontend_phase);
    assert_eq!(envelope["semantic_profile"], corpus_case.semantic_profile);
    assert_eq!(
        envelope["semantic_parameters"]["pointer_width"],
        corpus_case.pointer_width
    );
    assert_eq!(envelope["selection"]["function"], corpus_case.selection);
    assert_eq!(
        envelope["diagnostics"],
        Value::Array(corpus_case.diagnostics.clone())
    );
    assert_eq!(envelope["rejected_features"], json!([]));
    assert_eq!(canonical(&envelope["ir"]["value"]), vir_bytes);
    assert_eq!(canonical(&envelope["source_map"]), source_map_bytes);
    assert_eq!(canonical(&envelope["source_manifest"]), manifest_bytes);
    assert_eq!(
        envelope["ir"]["sha256"],
        envelope["ir"]["value"]["vir_hash"]
    );
}

fn captured_storage(
    root: &Path,
    corpus_case: &FrontendCase,
    manifest: &Value,
) -> Vec<(InputKind, String, Vec<u8>)> {
    let storage = manifest["inputs"]
        .as_array()
        .expect("source-manifest inputs")
        .iter()
        .map(|input| {
            let kind: InputKind =
                serde_json::from_value(input["kind"].clone()).expect("known input kind");
            let normalized = input["normalized_path"]
                .as_str()
                .expect("normalized path")
                .to_owned();
            let path = if matches!(normalized.as_str(), "Cargo.toml" | "Cargo.lock") {
                root.join("fixtures/rust-basic").join(&normalized)
            } else {
                root.join(PUBLIC_ROOT)
                    .join(&corpus_case.fixture)
                    .join("source")
                    .join(&normalized)
            };
            let bytes = fs::read(&path)
                .unwrap_or_else(|error| panic!("read captured input {}: {error}", path.display()));
            assert_eq!(
                bytes.len() as u64,
                input["size_bytes"].as_u64().expect("input size")
            );
            assert_eq!(sha256(&bytes), input["sha256"]);
            (kind, normalized, bytes)
        })
        .collect::<Vec<_>>();
    let contracts = storage
        .iter()
        .filter(|(kind, _, _)| *kind == InputKind::Contract)
        .map(|(_, path, _)| path.as_str())
        .collect::<BTreeSet<_>>();
    let sources = storage
        .iter()
        .filter(|(kind, _, _)| *kind == InputKind::Source)
        .map(|(_, path, _)| path.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        contracts,
        corpus_case.contracts.iter().map(String::as_str).collect()
    );
    assert_eq!(
        sources,
        corpus_case
            .captured_sources
            .iter()
            .map(String::as_str)
            .collect()
    );
    assert!(storage
        .iter()
        .all(|(_, path, _)| path != "src/unrelated.rs"));
    storage
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

fn assert_source_map_coverage(corpus_case: &FrontendCase, source_map: &mpk_vc::ValidatedSourceMap) {
    assert_eq!(
        source_map.map().entries.len(),
        corpus_case.source_map_entries
    );
    let mapped_sources = source_map
        .map()
        .entries
        .iter()
        .filter_map(|entry| match &entry.origin {
            SourceOrigin::Source {
                normalized_path, ..
            } => Some(normalized_path.as_str()),
            SourceOrigin::Synthetic { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        mapped_sources,
        corpus_case
            .captured_sources
            .iter()
            .map(String::as_str)
            .collect(),
        "{} source-map captured-source coverage",
        corpus_case.id
    );
    assert!(source_map
        .map()
        .entries
        .iter()
        .all(|entry| matches!(entry.origin, SourceOrigin::Source { .. })));
}

fn cross_check_program_and_groups(
    corpus_case: &FrontendCase,
    vir: &VirModule,
    program: &mpk_vc::ProgramVcModule,
    vc: &mpk_vc::VcDocument,
    skeleton: &mpk_vc::ValidatedVcCertificateSkeleton,
) -> (usize, usize) {
    let vir_functions = vir
        .units
        .iter()
        .flat_map(|unit| &unit.functions)
        .map(|function| (function.id.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let instructions = vir_functions
        .values()
        .flat_map(|function| function.blocks.iter())
        .flat_map(|block| block.instructions.iter())
        .collect::<Vec<_>>();
    let safety_check_count = instructions
        .iter()
        .map(|instruction| instruction_safety_checks(instruction).len())
        .sum::<usize>();
    let call_count = instructions
        .iter()
        .filter(|instruction| matches!(instruction, VirInstruction::CallStatic { .. }))
        .count();
    assert_eq!(vir_functions.len(), corpus_case.function_count);
    assert_eq!(instructions.len(), corpus_case.instruction_count);
    assert_eq!(safety_check_count, corpus_case.safety_check_count);
    assert_eq!(call_count, corpus_case.call_count);

    for function in vir_functions.values() {
        assert_eq!(
            contract_hash(&function.contracts)
                .expect("recompute contract hash")
                .as_str(),
            function.contracts.contract_hash.as_str(),
            "{} contract hash",
            function.id
        );
    }
    for instruction in &instructions {
        if let VirInstruction::CallStatic {
            function,
            contract_hash: repeated,
            ..
        } = instruction
        {
            let callee = vir_functions
                .get(function.as_str())
                .unwrap_or_else(|| panic!("missing static callee {function}"));
            assert_eq!(
                repeated.as_str(),
                callee.contracts.contract_hash.as_str(),
                "{function} repeated callee contract hash"
            );
        }
    }

    assert_eq!(program.functions.len(), vir_functions.len());
    assert_eq!(vc.functions.len(), program.functions.len());
    let program_positions = program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| (function.function_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let program_functions = program
        .functions
        .iter()
        .map(|function| (function.function_id.as_str(), function))
        .collect::<BTreeMap<_, _>>();

    let mut property_member_count = 0;
    let mut safety_member_count = 0;
    for (index, vc_function) in vc.functions.iter().enumerate() {
        let function = vir_functions[vc_function.function_id.as_str()];
        let generated = program_functions[vc_function.function_id.as_str()];
        assert_eq!(generated.function_id, vc_function.function_id);
        assert_eq!(
            program.functions[index].function_id,
            vc_function.function_id
        );
        assert_eq!(
            vc_function.contract_hash,
            function.contracts.contract_hash.as_str()
        );

        let function_instructions = function
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .collect::<Vec<_>>();
        let calls = function_instructions
            .iter()
            .filter_map(|instruction| match instruction {
                VirInstruction::CallStatic { function, .. } => Some(function.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let expected_preconditions = calls
            .iter()
            .map(|callee| vir_functions[callee].contracts.requires.len())
            .sum::<usize>();
        let expected_safety = function_instructions
            .iter()
            .map(|instruction| instruction_safety_checks(instruction).len())
            .sum::<usize>();
        assert!(
            count_program_members(generated, ProgramVcMemberKind::Postcondition)
                >= function.contracts.ensures.len().max(1),
            "{} property postconditions",
            function.id
        );
        assert_eq!(
            count_program_members(generated, ProgramVcMemberKind::OperationSafety),
            expected_safety,
            "{} operation safety members",
            function.id
        );
        assert_eq!(
            count_program_members(generated, ProgramVcMemberKind::CalleePrecondition),
            expected_preconditions,
            "{} callee precondition members",
            function.id
        );
        assert_eq!(
            count_program_members(generated, ProgramVcMemberKind::CalleePanicFree),
            calls.len(),
            "{} callee panic-free members",
            function.id
        );
        let direct_callees = calls.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            generated
                .direct_callees
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            direct_callees
        );
        for callee in &generated.direct_callees {
            assert!(
                program_positions[callee.as_str()] < program_positions[function.id.as_str()],
                "{} must follow callee {callee}",
                function.id
            );
        }

        assert_eq!(vc_function.groups.len(), 2);
        let mut partition = BTreeSet::new();
        for group in &vc_function.groups {
            let expected_members = vc_function
                .members
                .iter()
                .filter(|member| member.group_id == group.id)
                .map(|member| member.id.as_str())
                .collect::<BTreeSet<_>>();
            assert_eq!(
                group
                    .member_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                expected_members,
                "{} {} group membership",
                function.id,
                group.kind.as_str()
            );
            for member_id in &group.member_ids {
                assert!(
                    partition.insert(member_id.as_str()),
                    "duplicate grouped member"
                );
            }
            for member in vc_function
                .members
                .iter()
                .filter(|member| member.group_id == group.id)
            {
                assert_eq!(member.kind.required_group(), group.kind);
            }
            let expected_dependencies = match group.kind {
                VcGroupKind::Contract => &generated.contract_dependencies,
                VcGroupKind::PanicFree => &generated.panic_free_dependencies,
            };
            assert_eq!(&group.dependencies, expected_dependencies);
            let declaration = skeleton
                .skeleton()
                .theorem_declarations
                .iter()
                .find(|declaration| declaration.name == group.declaration_name)
                .expect("grouped theorem declaration");
            assert_eq!(declaration.function_id, vc_function.function_id);
            assert_eq!(declaration.group_id, group.id);
            assert_eq!(declaration.group_kind, group.kind);
            assert_eq!(declaration.member_ids, group.member_ids);
            assert_eq!(declaration.dependencies, group.dependencies);
            match group.kind {
                VcGroupKind::Contract => property_member_count += group.member_ids.len(),
                VcGroupKind::PanicFree => safety_member_count += group.member_ids.len(),
            }
        }
        assert_eq!(partition.len(), vc_function.members.len());
        assert!(vc_function.members.iter().all(|member| {
            partition.contains(member.id.as_str())
                && matches!(
                    member.kind,
                    VcMemberKind::Postcondition
                        | VcMemberKind::OperationSafety
                        | VcMemberKind::CalleePrecondition
                        | VcMemberKind::CalleePanicFree
                )
        }));
    }

    let declaration_names = skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .map(|declaration| declaration.name.as_str())
        .collect::<Vec<_>>();
    let expected_names = vc
        .functions
        .iter()
        .flat_map(|function| &function.groups)
        .map(|group| group.declaration_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        declaration_names, expected_names,
        "canonical declaration order"
    );
    let declaration_positions = declaration_names
        .iter()
        .enumerate()
        .map(|(index, name)| (*name, index))
        .collect::<BTreeMap<_, _>>();
    for (index, declaration) in skeleton.skeleton().theorem_declarations.iter().enumerate() {
        for dependency in &declaration.dependencies {
            assert!(
                declaration_positions[dependency.as_str()] < index,
                "declaration dependency must be topological"
            );
        }
    }
    (property_member_count, safety_member_count)
}

fn count_program_members(function: &mpk_vc::ProgramVcFunction, kind: ProgramVcMemberKind) -> usize {
    function
        .members
        .iter()
        .filter(|member| member.kind == kind)
        .count()
}

fn instruction_safety_checks(instruction: &VirInstruction) -> &[mpk_vc::VirSafetyCheck] {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn read_frontend_artifact(root: &Path, corpus_case: &FrontendCase, kind: &str) -> Vec<u8> {
    let item = corpus_case
        .artifacts
        .iter()
        .find(|item| item.kind == kind)
        .unwrap_or_else(|| panic!("{} missing artifact {kind}", corpus_case.id));
    fs::read(root.join(PUBLIC_ROOT).join(&item.path))
        .unwrap_or_else(|error| panic!("read {}: {error}", item.path))
}

fn artifact_base(corpus_case: &FrontendCase) -> &str {
    let mut parents = corpus_case.artifacts.iter().map(|item| {
        Path::new(&item.path)
            .parent()
            .and_then(Path::to_str)
            .expect("portable artifact parent")
    });
    let first = parents.next().expect("frontend artifact parent");
    assert!(parents.all(|parent| parent == first));
    first
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

fn sync_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create public corpus mirror");
    let mut entries = fs::read_dir(source)
        .expect("read frontend corpus tree")
        .map(|entry| entry.expect("frontend corpus entry"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let target = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("frontend corpus entry type")
            .is_dir()
        {
            sync_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("synchronize public frontend artifact");
        }
    }
}

fn assert_no_path_leakage(root: &Path, bytes: &[u8]) {
    let root_text = root.to_string_lossy();
    for forbidden in [
        root_text.as_bytes(),
        b"/root/",
        b"/tmp/",
        b"/mpk/",
        b"/not-emitted/",
        b"rust2vir-positive-",
        b"src/unrelated.rs",
        b"this file is intentionally outside the module closure",
        br#""timestamp""#,
        br#""generated_at""#,
        br#""hostname""#,
    ] {
        assert!(
            !bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "generated artifact leaks {}",
            String::from_utf8_lossy(forbidden)
        );
    }
    if let Ok(hostname) = env::var("HOSTNAME") {
        if !hostname.is_empty() {
            assert!(!bytes
                .windows(hostname.len())
                .any(|window| window == hostname.as_bytes()));
        }
    }
}

fn artifact(kind: &str, path: &str, bytes: &[u8]) -> CorpusArtifact {
    CorpusArtifact {
        bytes: bytes.len(),
        kind: kind.to_owned(),
        path: path.to_owned(),
        sha256: sha256(bytes),
    }
}

fn valid_registry(root: &Path) -> ValidatedReleaseRegistry {
    validate_release_registry(
        &fs::read(root.join("release/bundles/bundle-registry.json"))
            .expect("read release bundle registry"),
    )
    .expect("release bundle registry validates")
}

fn canonical(value: &impl Serialize) -> Vec<u8> {
    let transport = serde_json::to_vec(value).expect("serialize deterministic JSON");
    let strict = parse_strict_json(&transport, JSON_LIMITS).expect("strict deterministic JSON");
    canonical_json_bytes(&strict).expect("canonical deterministic JSON")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
