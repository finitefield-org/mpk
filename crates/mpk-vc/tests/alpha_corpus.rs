use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use mpk_vc::{
    emit_theorem_obligations, generate_branch_vcs, GirBinding, GirBlock, GirContractExpr,
    GirContracts, GirFieldType, GirFunction, GirInstruction, GirInstructionKind, GirIntLiteral,
    GirLoopContract, GirModule, GirPackage, GirRejectedFeature, GirTerminator, GirTerminatorKind,
    GirType, GirTypeKind, GirValue, GIR_SCHEMA_VERSION,
};

const UPDATE_ENV: &str = "MPK_UPDATE_VC_ALPHA";
const MANIFEST_SCHEMA: &str = "mpk.vc_alpha_manifest.v0";
const GO_ALPHA_MANIFEST_PATH: &str = "fixtures/go-alpha/manifest.json";
const GO_ALPHA_FUNCTION_COUNT: usize = 100;
const BRANCH_CASE_COUNT: usize = 33;
const POSTCONDITIONS_PER_CASE: usize = 16;
const EXPECTED_VC_COUNT: usize = BRANCH_CASE_COUNT * POSTCONDITIONS_PER_CASE * 2;

#[test]
fn alpha_vc_corpus_fixture_has_recorded_count_and_hash() {
    assert_go_alpha_dependency();

    let gir = alpha_branch_gir();
    let vc_module = generate_branch_vcs(&gir).expect("generate alpha branch VCs");
    let skeleton =
        emit_theorem_obligations(&vc_module).expect("emit alpha theorem-obligation skeletons");

    assert_eq!(vc_module.source_gir_hash, gir.gir_hash);
    assert_eq!(vc_module.obligations.len(), EXPECTED_VC_COUNT);
    assert_eq!(skeleton.theorem_declarations.len(), EXPECTED_VC_COUNT);

    let vc_json = pretty_json(&vc_module);
    let skeleton_json = pretty_json(&skeleton);
    let manifest = AlphaVcManifest {
        schema_version: MANIFEST_SCHEMA.to_owned(),
        source: AlphaSource {
            generator: "crates/mpk-vc/tests/alpha_corpus.rs".to_owned(),
            go_alpha_manifest: GO_ALPHA_MANIFEST_PATH.to_owned(),
            go_alpha_function_count: GO_ALPHA_FUNCTION_COUNT,
            gir_schema_version: GIR_SCHEMA_VERSION.to_owned(),
            branch_case_count: BRANCH_CASE_COUNT,
            postconditions_per_case: POSTCONDITIONS_PER_CASE,
            source_gir_hash: gir.gir_hash.clone().expect("GIR hash is set"),
        },
        artifacts: AlphaArtifacts {
            vc: AlphaArtifact {
                path: "vc.json".to_owned(),
                sha256: sha256_hex(vc_json.as_bytes()),
                obligation_count: Some(vc_module.obligations.len()),
                theorem_declaration_count: None,
            },
            skeleton: AlphaArtifact {
                path: "vc_skeleton.json".to_owned(),
                sha256: sha256_hex(skeleton_json.as_bytes()),
                obligation_count: None,
                theorem_declaration_count: Some(skeleton.theorem_declarations.len()),
            },
        },
    };
    let manifest_json = pretty_json(&manifest);

    let fixture_dir = vc_alpha_fixture_dir();
    assert_fixture(&fixture_dir.join("vc.json"), &vc_json);
    assert_fixture(&fixture_dir.join("vc_skeleton.json"), &skeleton_json);
    assert_fixture(&fixture_dir.join("manifest.json"), &manifest_json);

    let recorded = read_manifest(&fixture_dir.join("manifest.json"));
    assert_eq!(recorded.schema_version, MANIFEST_SCHEMA);
    assert_eq!(recorded.source.go_alpha_manifest, GO_ALPHA_MANIFEST_PATH);
    assert_eq!(
        recorded.source.go_alpha_function_count,
        GO_ALPHA_FUNCTION_COUNT
    );
    assert_eq!(recorded.source.branch_case_count, BRANCH_CASE_COUNT);
    assert_eq!(
        recorded.source.postconditions_per_case,
        POSTCONDITIONS_PER_CASE
    );
    assert_eq!(
        recorded.artifacts.vc.obligation_count,
        Some(EXPECTED_VC_COUNT)
    );
    assert_eq!(
        recorded.artifacts.skeleton.theorem_declaration_count,
        Some(EXPECTED_VC_COUNT)
    );
}

#[derive(Debug, Deserialize, Serialize)]
struct AlphaVcManifest {
    schema_version: String,
    source: AlphaSource,
    artifacts: AlphaArtifacts,
}

#[derive(Debug, Deserialize, Serialize)]
struct AlphaSource {
    generator: String,
    go_alpha_manifest: String,
    go_alpha_function_count: usize,
    gir_schema_version: String,
    branch_case_count: usize,
    postconditions_per_case: usize,
    source_gir_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AlphaArtifacts {
    vc: AlphaArtifact,
    skeleton: AlphaArtifact,
}

#[derive(Debug, Deserialize, Serialize)]
struct AlphaArtifact {
    path: String,
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    obligation_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theorem_declaration_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct GoAlphaManifest {
    function_count: usize,
    positive: Vec<GoAlphaPositiveCase>,
}

#[derive(Debug, Deserialize)]
struct GoAlphaPositiveCase {
    path: String,
    function_count: usize,
}

fn alpha_branch_gir() -> GirModule {
    let functions = (0..BRANCH_CASE_COUNT)
        .map(alpha_branch_function)
        .collect::<Vec<_>>();
    let mut module = GirModule {
        schema_version: GIR_SCHEMA_VERSION.to_owned(),
        packages: vec![GirPackage {
            package_path: "alpha.branch".to_owned(),
            name: "branch".to_owned(),
            functions,
        }],
        gir_hash: None,
    };
    let hash_input = serde_json::to_string(&module).expect("serialize alpha GIR hash input");
    module.gir_hash = Some(sha256_hex(hash_input.as_bytes()));
    module
}

fn alpha_branch_function(index: usize) -> GirFunction {
    let function_name = format!("Case{index:03}");
    let function_id = format!("alpha.branch.{function_name}");
    GirFunction {
        id: function_id,
        package: "alpha.branch".to_owned(),
        name: function_name,
        params: vec![binding("a"), binding("b")],
        results: vec![binding("result0")],
        locals: vec![binding("selected")],
        blocks: vec![
            GirBlock {
                label: "entry".to_owned(),
                parameters: Vec::new(),
                instructions: vec![
                    copy_instruction("t0", "selected", var_value("a")),
                    GirInstruction {
                        id: "t1".to_owned(),
                        kind: GirInstructionKind::BinOp,
                        op: Some("signed_gt".to_owned()),
                        r#type: bool_type(),
                        target: None,
                        value: None,
                        base: None,
                        index: None,
                        field: None,
                        fields: Vec::new(),
                        elements: Vec::new(),
                        lhs: Some(var_value("b")),
                        rhs: Some(var_value("selected")),
                        function: None,
                        args: Vec::new(),
                    },
                ],
                terminator: GirTerminator {
                    kind: GirTerminatorKind::Branch,
                    values: Vec::new(),
                    cond: Some(var_value("t1")),
                    label: None,
                    then_label: Some("if_then_0".to_owned()),
                    else_label: Some("if_after_1".to_owned()),
                    args: Vec::new(),
                    reason: None,
                },
            },
            GirBlock {
                label: "if_then_0".to_owned(),
                parameters: Vec::new(),
                instructions: vec![copy_instruction("t2", "selected", var_value("b"))],
                terminator: GirTerminator {
                    kind: GirTerminatorKind::Jump,
                    values: Vec::new(),
                    cond: None,
                    label: Some("if_after_1".to_owned()),
                    then_label: None,
                    else_label: None,
                    args: Vec::new(),
                    reason: None,
                },
            },
            GirBlock {
                label: "if_after_1".to_owned(),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: GirTerminator {
                    kind: GirTerminatorKind::Return,
                    values: vec![var_value("selected")],
                    cond: None,
                    label: None,
                    then_label: None,
                    else_label: None,
                    args: Vec::new(),
                    reason: None,
                },
            },
        ],
        contracts: GirContracts {
            requires: Vec::new(),
            ensures: alpha_postconditions(),
            modifies: Vec::new(),
            loops: Vec::<GirLoopContract>::new(),
        },
        supported_features: vec![
            "params".to_owned(),
            "locals".to_owned(),
            "blocks".to_owned(),
            "binops".to_owned(),
            "if".to_owned(),
            "return".to_owned(),
        ],
        rejected_features: Vec::<GirRejectedFeature>::new(),
    }
}

fn alpha_postconditions() -> Vec<GirContractExpr> {
    vec![
        binary_expr("signed_ge", result_expr(0), var_expr("a")),
        binary_expr("signed_ge", result_expr(0), var_expr("b")),
        or_expr(vec![
            binary_expr("eq", result_expr(0), var_expr("a")),
            binary_expr("eq", result_expr(0), var_expr("b")),
        ]),
        binary_expr("signed_le", var_expr("a"), result_expr(0)),
        binary_expr("signed_le", var_expr("b"), result_expr(0)),
        binary_expr("eq", result_expr(0), result_expr(0)),
        binary_expr("signed_ge", result_expr(0), result_expr(0)),
        or_expr(vec![
            binary_expr("signed_ge", result_expr(0), var_expr("a")),
            binary_expr("signed_ge", result_expr(0), var_expr("b")),
        ]),
        binary_expr("signed_ge", var_expr("a"), var_expr("a")),
        binary_expr("signed_ge", var_expr("b"), var_expr("b")),
        binary_expr("signed_le", var_expr("a"), var_expr("a")),
        binary_expr("signed_le", var_expr("b"), var_expr("b")),
        binary_expr("eq", var_expr("a"), var_expr("a")),
        binary_expr("eq", var_expr("b"), var_expr("b")),
        or_expr(vec![
            binary_expr("eq", result_expr(0), var_expr("a")),
            binary_expr("signed_ge", result_expr(0), var_expr("b")),
        ]),
        or_expr(vec![
            binary_expr("eq", result_expr(0), var_expr("b")),
            binary_expr("signed_ge", result_expr(0), var_expr("a")),
        ]),
    ]
}

fn binding(name: &str) -> GirBinding {
    GirBinding {
        name: name.to_owned(),
        r#type: int64_type(),
    }
}

fn copy_instruction(id: &str, target: &str, value: GirValue) -> GirInstruction {
    GirInstruction {
        id: id.to_owned(),
        kind: GirInstructionKind::Copy,
        op: None,
        r#type: int64_type(),
        target: Some(target.to_owned()),
        value: Some(value),
        base: None,
        index: None,
        field: None,
        fields: Vec::new(),
        elements: Vec::new(),
        lhs: None,
        rhs: None,
        function: None,
        args: Vec::new(),
    }
}

fn int64_type() -> GirType {
    GirType {
        kind: GirTypeKind::BitVector,
        name: None,
        width: Some(64),
        signed: Some(true),
        length: None,
        element: None,
        fields: Vec::<GirFieldType>::new(),
    }
}

fn bool_type() -> GirType {
    GirType {
        kind: GirTypeKind::Bool,
        name: None,
        width: None,
        signed: None,
        length: None,
        element: None,
        fields: Vec::<GirFieldType>::new(),
    }
}

fn var_value(name: &str) -> GirValue {
    GirValue {
        var: Some(name.to_owned()),
        int: None::<GirIntLiteral>,
        bool: None,
    }
}

fn empty_expr() -> GirContractExpr {
    GirContractExpr {
        op: None,
        args: Vec::new(),
        lhs: None,
        rhs: None,
        value: None,
        r#type: None,
        var: None,
        result: None,
        bool: None,
        int: None,
    }
}

fn var_expr(name: &str) -> GirContractExpr {
    GirContractExpr {
        var: Some(name.to_owned()),
        ..empty_expr()
    }
}

fn result_expr(index: u32) -> GirContractExpr {
    GirContractExpr {
        result: Some(index),
        ..empty_expr()
    }
}

fn binary_expr(op: &str, lhs: GirContractExpr, rhs: GirContractExpr) -> GirContractExpr {
    GirContractExpr {
        op: Some(op.to_owned()),
        lhs: Some(Box::new(lhs)),
        rhs: Some(Box::new(rhs)),
        ..empty_expr()
    }
}

fn or_expr(args: Vec<GirContractExpr>) -> GirContractExpr {
    GirContractExpr {
        op: Some("or".to_owned()),
        args,
        ..empty_expr()
    }
}

fn vc_alpha_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/vc-alpha")
        .components()
        .collect()
}

fn pretty_json(value: &impl Serialize) -> String {
    let mut output = serde_json::to_string_pretty(value).expect("serialize fixture JSON");
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
        fs::write(path, actual).unwrap_or_else(|error| {
            panic!("write updated alpha VC fixture {}: {error}", path.display())
        });
        return;
    }

    let expected = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read alpha VC fixture {}: {error}", path.display()));
    assert_eq!(actual, expected, "fixture mismatch for {}", path.display());
}

fn read_manifest(path: &Path) -> AlphaVcManifest {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read alpha VC manifest {}: {error}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|error| panic!("decode alpha VC manifest {}: {error}", path.display()))
}

fn assert_go_alpha_dependency() {
    let manifest_path = repo_root().join(GO_ALPHA_MANIFEST_PATH);
    let content = fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
        panic!(
            "read ALPHA-001 Go corpus manifest {}: {error}",
            manifest_path.display()
        )
    });
    let manifest = serde_json::from_str::<GoAlphaManifest>(&content).unwrap_or_else(|error| {
        panic!(
            "decode ALPHA-001 Go corpus manifest {}: {error}",
            manifest_path.display()
        )
    });
    assert_eq!(manifest.function_count, GO_ALPHA_FUNCTION_COUNT);
    let branch_case = manifest
        .positive
        .iter()
        .find(|case| case.path == "branch")
        .expect("ALPHA-001 branch corpus entry is present");
    assert_eq!(branch_case.function_count, BRANCH_CASE_COUNT);
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
