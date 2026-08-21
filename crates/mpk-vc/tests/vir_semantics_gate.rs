use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::decode_canonical_certificate;
use mpk_vc::{
    evaluate_total_bitvector_operation, required_safety_checks, sha256_raw_file_bytes,
    BitVectorWidth, SemanticProfile, TotalBitVectorResult, VirBinaryOperator, VirSafetyOperation,
    VirType,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Gate {
    schema: String,
    status: String,
    unchecked_axioms: u64,
    proof_pending_assignments: Vec<Value>,
    safety_routes: String,
    reviewed_vectors: Vec<ReviewedVector>,
    go_semantic_baseline: Baseline,
    checked_foundation_fixtures: Vec<String>,
    parser_fuzz_targets: Vec<String>,
    total_operation_cases: Vec<TotalOperationCase>,
    safety_check_cases: Vec<SafetyCheckCase>,
    findings: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedVector {
    path: String,
    schema: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    path: String,
    schema: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TotalOperationCase {
    id: String,
    profiles: Vec<String>,
    operation: String,
    width: u32,
    lhs: u64,
    rhs_width: u32,
    rhs: u64,
    result_kind: String,
    result: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SafetyCheckCase {
    id: String,
    profile: SemanticProfile,
    operation: String,
    lhs_width: u32,
    lhs_signed: bool,
    rhs_width: u32,
    rhs_signed: bool,
    checks: Vec<Value>,
}

#[test]
fn shared_foundation_gate_is_closed_and_executable() {
    let root = repo_root();
    let gate: Gate = serde_json::from_slice(
        &fs::read(root.join("fixtures/vir-semantics/expected.json")).expect("gate fixture"),
    )
    .expect("closed gate schema");
    assert_eq!(gate.schema, "mpk.vir_semantics_gate.v0");
    assert_eq!(gate.status, "closed");
    assert_eq!(gate.unchecked_axioms, 0);
    assert!(gate.proof_pending_assignments.is_empty());
    assert!(gate.findings.is_empty());
    assert_eq!(gate.safety_routes, "fixtures/program-safety/expected.json");

    let mut paths = BTreeSet::new();
    for reviewed in &gate.reviewed_vectors {
        assert!(
            paths.insert(reviewed.path.as_str()),
            "duplicate vector path"
        );
        let value: Value = serde_json::from_slice(
            &fs::read(root.join(&reviewed.path)).expect("reviewed vector exists"),
        )
        .expect("reviewed vector JSON");
        assert_eq!(value["schema"], reviewed.schema, "{}", reviewed.path);
    }
    assert_eq!(gate.reviewed_vectors.len(), 8);

    let baseline_path = root.join(&gate.go_semantic_baseline.path);
    let baseline: Value =
        serde_json::from_slice(&fs::read(&baseline_path).expect("Go baseline exists"))
            .expect("Go baseline JSON");
    assert_eq!(baseline["schema"], gate.go_semantic_baseline.schema);
    assert_eq!(
        sha256_raw_file_bytes(&fs::read(baseline_path).expect("Go baseline bytes")).to_hex(),
        gate.go_semantic_baseline.sha256
    );

    for fixture in &gate.checked_foundation_fixtures {
        let certificate = decode_canonical_certificate(&decode_hex(&root.join(fixture)))
            .unwrap_or_else(|error| panic!("{fixture} must be canonical: {error:?}"));
        assert_eq!(
            certificate.axiom_report.summary.total_axiom_count, 0,
            "{fixture}"
        );
    }

    assert_eq!(
        gate.parser_fuzz_targets,
        [
            "fuzz/fuzz_targets/vir_parser.rs",
            "fuzz/fuzz_targets/source_map_parser.rs"
        ]
    );
    for target in &gate.parser_fuzz_targets {
        assert!(root.join(target).is_file(), "missing fuzz target {target}");
    }

    for case in &gate.total_operation_cases {
        assert_eq!(
            case.profiles,
            ["mpk.go.fixed.v0", "mpk.rust.checked.v0"],
            "{}",
            case.id
        );
        let actual = evaluate_total_bitvector_operation(
            operation(&case.operation),
            BitVectorWidth::try_from(case.width).expect("supported width"),
            case.lhs,
            BitVectorWidth::try_from(case.rhs_width).expect("supported RHS width"),
            case.rhs,
        )
        .unwrap_or_else(|error| panic!("{}: {error}", case.id));
        match (case.result_kind.as_str(), actual) {
            ("bitvector", TotalBitVectorResult::BitVector(actual)) => {
                assert_eq!(case.result.as_u64(), Some(actual), "{}", case.id);
            }
            ("boolean", TotalBitVectorResult::Boolean(actual)) => {
                assert_eq!(case.result.as_bool(), Some(actual), "{}", case.id);
            }
            _ => panic!("{} has a mismatched result kind", case.id),
        }
    }

    for case in &gate.safety_check_cases {
        let operand_types = [
            VirType::Bv {
                width: BitVectorWidth::try_from(case.lhs_width).expect("supported LHS width"),
                signed: case.lhs_signed,
            },
            VirType::Bv {
                width: BitVectorWidth::try_from(case.rhs_width).expect("supported RHS width"),
                signed: case.rhs_signed,
            },
        ];
        let actual = required_safety_checks(
            case.profile,
            VirSafetyOperation::Binary(operation(&case.operation)),
            &operand_types,
        )
        .unwrap_or_else(|error| panic!("{}: {error}", case.id));
        assert_eq!(
            serde_json::to_value(actual).expect("safety checks serialize"),
            Value::Array(case.checks.clone()),
            "{}",
            case.id
        );
    }
}

fn operation(name: &str) -> VirBinaryOperator {
    match name {
        "bv_udiv" => VirBinaryOperator::BvUdiv,
        "bv_sdiv" => VirBinaryOperator::BvSdiv,
        "bv_add" => VirBinaryOperator::BvAdd,
        "bv_shl" => VirBinaryOperator::BvShl,
        "signed_lt" => VirBinaryOperator::SignedLt,
        name => panic!("unsupported fixture operation {name:?}"),
    }
}

fn decode_hex(path: &Path) -> Vec<u8> {
    let input = fs::read_to_string(path).expect("hex fixture");
    let input = input.trim();
    assert_eq!(input.len() % 2, 0);
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16)
                .expect("valid hex")
        })
        .collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
