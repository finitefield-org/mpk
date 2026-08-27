use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
}

#[test]
fn frozen_reject_before_vir_rows_are_owned_exactly() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let rows = profile["semantic_rows"].as_array().expect("semantic rows");
    let actual = rows
        .iter()
        .filter(|row| row["disposition"] == "reject_before_vir")
        .map(|row| (row["row"].as_str().unwrap(), row["basis"].as_str().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            ("M03", "unsupported"),
            ("M04", "foundation-binary-float"),
            ("M05", "foundation-csharp-decimal"),
            ("M06", "unsupported"),
            ("M15", "unsupported"),
            ("M17", "unsupported"),
            ("M20", "unsupported"),
            ("M22", "successor-operation-gap"),
            ("M23", "unsupported"),
            ("M24", "unsupported"),
            ("M25", "unsupported"),
            ("M26", "unsupported"),
            ("M28", "unsupported"),
            ("M30", "unsupported"),
            ("M31", "unsupported"),
            ("M32", "unsupported"),
        ]
    );
    assert_eq!(rows.len(), 34);
}

#[test]
fn pre_lowering_limits_are_bound_to_the_candidate_gate() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let limits = profile["limit_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|limit| {
            let id = limit["id"].as_str().unwrap();
            matches!(
                id,
                "method_closure"
                    | "syntax_nodes"
                    | "operations_per_method"
                    | "operations_per_closure"
                    | "cfg_blocks_per_method"
                    | "cfg_blocks_per_closure"
            )
            .then(|| {
                (
                    id,
                    (
                        limit["maximum"].as_u64().unwrap(),
                        limit["code"].as_str().unwrap(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        limits,
        BTreeMap::from([
            (
                "cfg_blocks_per_closure",
                (8192, "CSHARP_LIMIT_CFG_BLOCKS_PER_CLOSURE"),
            ),
            (
                "cfg_blocks_per_method",
                (1024, "CSHARP_LIMIT_CFG_BLOCKS_PER_METHOD"),
            ),
            ("method_closure", (128, "CSHARP_LIMIT_METHOD_CLOSURE")),
            (
                "operations_per_closure",
                (250_000, "CSHARP_LIMIT_OPERATIONS_PER_CLOSURE"),
            ),
            (
                "operations_per_method",
                (100_000, "CSHARP_LIMIT_OPERATIONS_PER_METHOD"),
            ),
            ("syntax_nodes", (250_000, "CSHARP_LIMIT_SYNTAX_NODES")),
        ])
    );

    let root = repository_root();
    let model = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetModel.cs"))
        .expect("read subset model");
    let operations = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetOperations.cs"))
        .expect("read subset operations");
    let symbols = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetSymbols.cs"))
        .expect("read subset symbols");
    let validator = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetValidator.cs"))
        .expect("read subset validator");
    let limits = fs::read_to_string(root.join("csharp-tools/csharp2vir/FrontendLimits.cs"))
        .expect("read frontend limits");
    for required in [
        "MethodClosureMaximum = 128",
        "SyntaxNodesMaximum = 250_000",
        "OperationsPerMethodMaximum = 100_000",
        "OperationsPerClosureMaximum = 250_000",
        "CfgBlocksPerMethodMaximum = 1_024",
        "CfgBlocksPerClosureMaximum = 8_192",
        "checked(current + increment)",
    ] {
        assert!(limits.contains(required), "missing limit owner {required}");
    }
    assert!(model.contains("FrontendLimits.MethodClosureMaximum"));
    for required in [
        "DescendantNodesAndSelf(descendIntoTrivia: false)",
        "ReferenceOperationComparer.Instance",
        "block.Operations",
        "block.BranchValue",
        "operation.ChildOperations",
        "graph.Blocks.Length",
    ] {
        assert!(
            operations.contains(required) || symbols.contains(required),
            "missing accounting owner {required}"
        );
    }
    assert!(validator.contains("CSHARP_LIMIT_METHOD_CLOSURE"));

    let add_tree = operations
        .split_once("private static void AddOperationTree")
        .expect("operation-union builder")
        .1;
    let retain = add_tree.find("operations.Add(operation)").unwrap();
    assert!(
        add_tree.find("OperationsPerMethodMaximum").unwrap() < retain,
        "per-method operation limit must precede retention"
    );
    assert!(
        add_tree.find("OperationsPerClosureMaximum").unwrap() < retain,
        "closure operation limit must precede retention"
    );

    let visit = validator
        .split_once("private static void Visit")
        .expect("closure visitor")
        .1;
    assert!(
        visit.find("CSHARP_LIMIT_METHOD_CLOSURE").unwrap()
            < visit.find("states.Add(id, 1)").unwrap(),
        "closure limit must precede analysis retention"
    );
}

#[test]
fn declaration_closure_purity_and_cfg_gates_are_private_and_complete() {
    let root = repository_root();
    let symbols = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetSymbols.cs"))
        .expect("read subset symbols");
    let operations = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetOperations.cs"))
        .expect("read subset operations");
    let validator = fs::read_to_string(root.join("csharp-tools/csharp2vir/SubsetValidator.cs"))
        .expect("read subset validator");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive program");
    let combined = format!("{symbols}\n{operations}\n{validator}");

    for required in [
        "CSHARP_SUBSET_DECLARATION",
        "CSHARP_SUBSET_TYPE",
        "CSHARP_SUBSET_LITERAL",
        "CSHARP_SUBSET_CONTROL_FLOW",
        "CSHARP_SUBSET_OPERATION",
        "CSHARP_SUBSET_OVERFLOW_CONTEXT",
        "CSHARP_SUBSET_CHECKED_CONVERSION",
        "CSHARP_SUBSET_CONVERSION",
        "CSHARP_SUBSET_CALL",
        "CSHARP_SUBSET_INITIALIZATION",
        "CSHARP_SUBSET_PURITY",
        "CSHARP_SUBSET_ABRUPT",
        "SymbolEqualityComparer.Default",
        "ControlFlowRegionKind.LocalLifetime",
        "ValidateDefiniteAssignment",
        "operation.TargetMethod.DeclaringSyntaxReferences",
        "CanonicalOrder",
        "ready.Min",
    ] {
        assert!(
            combined.contains(required),
            "missing subset owner {required}"
        );
    }
    for forbidden in [
        "MSBuildWorkspace",
        "BindingFlags.NonPublic",
        "ToFullString",
        ".Emit(",
        "FrontendEnvelope",
        "NormalizedContract",
        "MPK-CONTRACT-1.0",
    ] {
        assert!(
            !combined.contains(forbidden),
            "later or forbidden surface {forbidden}"
        );
    }
    assert!(program.contains("phase = \"typecheck\""));
    assert!(program.contains("CSharpSubset.Validate(selection, compilationSession)"));
    assert!(program.contains("CSharpContracts.Attach(selection, snapshot, closure)"));
}

#[test]
fn executable_harness_and_build_gate_own_subset_drift() {
    let root = repository_root();
    let harness = fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_subset_harness.cs"))
        .expect("read executable subset harness");
    for owner in [
        "AcceptedClosureIsDeterministic",
        "ExactTypesLiteralsAndControlAreAccepted",
        "DeclarationTypeAndLiteralAdmissionIsClosed",
        "ControlOperationAndConversionAdmissionIsClosed",
        "ClosurePurityAndInitializationAreClosed",
        "DefiniteAssignmentAndCfgAccountingAreOwned",
        "SemanticRowRejectionsAreOwned",
        "LimitsAreInclusiveAndChecked",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }

    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    assert!(script.contains("validate_subset_implementation"));
    assert!(script.contains("run_subset_tests=True"));
    assert!(script.contains("csharp2vir-subset-tests.dll"));

    let manifest = load("develop/specs/vectors/manifest.json");
    let record = manifest["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == "develop/specs/vectors/csharp-profile-v0.json")
        .expect("C# vector manifest record");
    assert!(record["implementation_test_owners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_subset.rs"));
}

#[test]
fn provisioned_offline_closure_executes_the_subset_harness() {
    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let hash = profile["toolchain_inputs"]["toolchain_inputs_sha256"]
        .as_str()
        .unwrap();
    let cache = root
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = profile["toolchain_inputs"]["archives"].as_array().unwrap();
    let present = archives
        .iter()
        .filter(|record| {
            let suffix = match record["kind"].as_str().unwrap() {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!("{}{}", record["id"].as_str().unwrap(), suffix))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial C# archive cache"
    );
    if present == 0 {
        return;
    }

    let output = Command::new(root.join("scripts/build-csharp-frontend.sh"))
        .arg("--test-subset")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# subset harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
