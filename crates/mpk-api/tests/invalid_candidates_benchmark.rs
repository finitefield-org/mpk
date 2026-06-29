use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_api::{
    export_batch_candidates_jsonl, ApiErrorCode, ApiProofId, ApiService, BatchCandidate,
    BatchCheckMode, BatchCheckRequest, BatchCheckSummary, JsonlImportRequest, StartSessionRequest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const UPDATE_ENV: &str = "MPK_UPDATE_INVALID_CANDIDATES";
const MANIFEST_SCHEMA: &str = "mpk.invalid_candidates_benchmark.v0";
const CANDIDATE_COUNT: usize = 10_000;
const FIRST_PROOF_ID: u32 = 1_000_000;

#[test]
fn invalid_candidate_benchmark_imports_and_rejects_deterministically() {
    let candidates = invalid_candidates();
    let jsonl = export_batch_candidates_jsonl(&candidates).expect("benchmark JSONL exports");
    let manifest = InvalidCandidatesManifest {
        schema_version: MANIFEST_SCHEMA.to_owned(),
        format: "mpk-api BatchCandidate JSONL".to_owned(),
        mode: BatchCheckMode::FailFastPerCandidate,
        candidate_count: CANDIDATE_COUNT,
        invalid_by_design: InvalidByDesign {
            expected_error_code: ApiErrorCode::UnknownProof,
            first_proof_id: FIRST_PROOF_ID,
            last_proof_id: FIRST_PROOF_ID + u32::try_from(CANDIDATE_COUNT).unwrap() - 1,
        },
        artifacts: InvalidCandidateArtifacts {
            candidates: InvalidCandidateArtifact {
                path: "candidates.jsonl".to_owned(),
                sha256: sha256_hex(jsonl.as_bytes()),
                records: candidates.len(),
            },
        },
        expected_summary: BatchCheckSummary {
            total: CANDIDATE_COUNT,
            accepted: 0,
            rejected: CANDIDATE_COUNT,
        },
    };
    let manifest_json = pretty_json(&manifest);

    let fixture_dir = invalid_candidates_fixture_dir();
    assert_fixture(&fixture_dir.join("candidates.jsonl"), &jsonl);
    assert_fixture(&fixture_dir.join("manifest.json"), &manifest_json);

    let recorded = read_manifest(&fixture_dir.join("manifest.json"));
    let recorded_jsonl = fs::read_to_string(fixture_dir.join("candidates.jsonl"))
        .expect("read invalid candidate JSONL");
    assert_eq!(recorded.schema_version, MANIFEST_SCHEMA);
    assert_eq!(recorded.candidate_count, CANDIDATE_COUNT);
    assert_eq!(recorded.artifacts.candidates.records, CANDIDATE_COUNT);
    assert_eq!(
        recorded.artifacts.candidates.sha256,
        sha256_hex(recorded_jsonl.as_bytes())
    );

    let mut api = ApiService::new();
    let session_id = api
        .start_session(StartSessionRequest::new("Example.Api.InvalidBenchmark"))
        .expect("session starts")
        .session_id;
    let imported = api
        .vc_import_candidates_jsonl(JsonlImportRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            jsonl: recorded_jsonl,
        })
        .expect("benchmark JSONL imports");
    assert_eq!(imported.records, CANDIDATE_COUNT);
    assert_eq!(imported.batch_request.candidates[0], candidates[0]);
    assert_eq!(
        imported.batch_request.candidates[CANDIDATE_COUNT - 1],
        candidates[CANDIDATE_COUNT - 1]
    );

    let response = api
        .vc_check_candidates(BatchCheckRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates: imported.batch_request.candidates,
        })
        .expect("benchmark candidates check");
    assert_eq!(response.summary, recorded.expected_summary);
    assert_eq!(response.verdicts.len(), CANDIDATE_COUNT);
    assert!(response.verdicts.iter().all(|verdict| !verdict.ok));
    assert!(response.verdicts.iter().all(|verdict| {
        verdict
            .error
            .as_ref()
            .is_some_and(|error| error.code == ApiErrorCode::UnknownProof)
    }));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        0
    );
}

#[derive(Debug, Deserialize, Serialize)]
struct InvalidCandidatesManifest {
    schema_version: String,
    format: String,
    mode: BatchCheckMode,
    candidate_count: usize,
    invalid_by_design: InvalidByDesign,
    artifacts: InvalidCandidateArtifacts,
    expected_summary: BatchCheckSummary,
}

#[derive(Debug, Deserialize, Serialize)]
struct InvalidByDesign {
    expected_error_code: ApiErrorCode,
    first_proof_id: u32,
    last_proof_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct InvalidCandidateArtifacts {
    candidates: InvalidCandidateArtifact,
}

#[derive(Debug, Deserialize, Serialize)]
struct InvalidCandidateArtifact {
    path: String,
    sha256: String,
    records: usize,
}

fn invalid_candidates() -> Vec<BatchCandidate> {
    (0..CANDIDATE_COUNT)
        .map(|index| BatchCandidate {
            candidate_id: format!("invalid-{index:05}"),
            proof_id: ApiProofId(FIRST_PROOF_ID + u32::try_from(index).unwrap()),
        })
        .collect()
}

fn invalid_candidates_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/invalid-candidates")
        .components()
        .collect()
}

fn pretty_json(value: &impl Serialize) -> String {
    let mut output = serde_json::to_string_pretty(value).expect("serialize manifest JSON");
    output.push('\n');
    output
}

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn assert_fixture(path: &Path, actual: &str) {
    if env::var_os(UPDATE_ENV).is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!(
                    "create invalid candidate fixture directory {}: {error}",
                    parent.display()
                )
            });
        }
        fs::write(path, actual).unwrap_or_else(|error| {
            panic!(
                "write updated invalid candidate fixture {}: {error}",
                path.display()
            )
        });
        return;
    }

    let expected = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("read invalid candidate fixture {}: {error}", path.display())
    });
    assert_eq!(actual, expected, "fixture mismatch for {}", path.display());
}

fn read_manifest(path: &Path) -> InvalidCandidatesManifest {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "read invalid candidate manifest {}: {error}",
            path.display()
        )
    });
    serde_json::from_str(&content).unwrap_or_else(|error| {
        panic!(
            "decode invalid candidate manifest {}: {error}",
            path.display()
        )
    })
}
