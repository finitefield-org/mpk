use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SELECTION_DOMAIN: &[u8] = b"MPK-CSHARP-SELECTION-0.1\0";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("JSON object")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn frozen_selection_is_executably_owned_by_the_candidate() {
    let vector = load("develop/specs/vectors/csharp-profile-v0.json");
    let selection = &vector["selection_fixture"];
    let canonical = serde_json::to_vec(selection).expect("canonical fixture value");
    assert_eq!(canonical.len(), 215);
    let mut preimage = SELECTION_DOMAIN.to_vec();
    preimage.extend_from_slice(&canonical);
    assert_eq!(sha256(&preimage), text(&vector["selection_sha256"]));
    assert_eq!(
        text(&vector["selection_sha256"]),
        "d5033138bd8c53eee3901d0d1852ed4c1b1a85686cf2a68f01effb0b8c70dfcd"
    );

    let root = repository_root();
    let selection_source = fs::read_to_string(root.join("csharp-tools/csharp2vir/Selection.cs"))
        .expect("read C# selection implementation");
    let frontend_model = fs::read_to_string(root.join("csharp-tools/csharp2vir/FrontendModel.cs"))
        .expect("read C# frontend model");
    for required in [
        "MPK-CSHARP-SELECTION-0.1\\0",
        "CSHARP_LIMIT_SOURCE_FILES",
        "CSHARP_LIMIT_CONTRACT_FILES",
        "CSHARP_LIMIT_SELECTED_METHODS",
        "CSHARP_LIMIT_NORMALIZED_PATH_BYTES",
        "CSHARP_LIMIT_CANONICAL_METHOD_ID_BYTES",
        "ParseMethodId",
        "IsPortablePath",
    ] {
        assert!(selection_source.contains(required), "missing {required}");
    }
    assert!(selection_source.contains("JavaScriptEncoder.UnsafeRelaxedJsonEscaping"));
    assert!(selection_source.contains("FrontendConstants.SelectionSchema"));
    assert!(frontend_model.contains("mpk.selection.csharp_methods.v0"));
}

#[test]
fn private_cli_and_capture_boundary_are_closed_before_roslyn() {
    let root = repository_root();
    let cli = fs::read_to_string(root.join("csharp-tools/csharp2vir/Cli.cs"))
        .expect("read C# CLI implementation");
    let capture = fs::read_to_string(root.join("csharp-tools/csharp2vir/Capture.cs"))
        .expect("read C# capture implementation");
    let source_transport =
        fs::read_to_string(root.join("csharp-tools/csharp2vir/SourceTransport.cs"))
            .expect("read C# source transport implementation");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive C# program");
    let frontend_model = fs::read_to_string(root.join("csharp-tools/csharp2vir/FrontendModel.cs"))
        .expect("read C# frontend model");

    let ordered_cli_tokens = [
        "--semantic-profile",
        "--target",
        "--compilation",
        "--source",
        "--contract",
        "--method",
        "--profile-registry-id",
        "--profile-registry-revision",
        "--profile-registry-sha256",
        "--profile-entry-sha256",
        "--frontend-bundle-id",
        "--frontend-sha256",
        "--release-registry-id",
        "--release-registry-sha256",
        "--toolchain-bundle-id",
        "--toolchain-root",
        "--toolchain-distribution-sha256",
    ];
    let mut previous = 0;
    for token in ordered_cli_tokens {
        let position = cli.find(token).unwrap_or_else(|| panic!("missing {token}"));
        assert!(position >= previous, "CLI token order differs at {token}");
        previous = position;
    }
    for forbidden in [
        "--compiler",
        "--runtime",
        "--reference",
        "--analyzer",
        "--sdk",
        "--project",
        "--response",
        "--plugin",
    ] {
        assert!(!cli.contains(forbidden), "forbidden option {forbidden}");
    }

    for required in [
        "OpenNoFollow",
        "OpenAt",
        "FStat",
        "FStatAt",
        "ReadDir",
        "LinkCount != 1",
        "CSHARP_CAPTURE_FILE_TYPE",
        "CSHARP_CAPTURE_PATH",
        "CSHARP_CAPTURE_INVENTORY",
        "CSHARP_LIMIT_SOURCE_FILE_BYTES",
        "CSHARP_LIMIT_SOURCE_TOTAL_BYTES",
        "CSHARP_LIMIT_CONTRACT_FILE_BYTES",
        "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES",
        "CSHARP_LIMIT_SNAPSHOT_ENTRIES",
        "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES",
        "AddExpectedNodeWithinLimit",
    ] {
        assert!(capture.contains(required), "missing {required}");
    }
    assert!(!capture.contains("ReadAllBytes"));
    assert!(!capture.contains("Directory.Enumerate"));
    assert!(!source_transport.contains("System.IO"));
    assert!(!source_transport.contains("File."));
    assert!(!source_transport.contains("Directory."));
    assert!(source_transport.contains("new UTF8Encoding(false, true)"));
    assert!(source_transport.contains("CSHARP_SOURCE_ENCODING"));

    let capture_call = program.find("SnapshotCapture.Capture").unwrap();
    let transport_call = program.find("SourceTransport.Validate").unwrap();
    let parse_call = program.find("RoslynSessionFactory.Parse").unwrap();
    assert!(capture_call < transport_call && transport_call < parse_call);
    assert!(!program.contains("ParseText"));
    assert!(!program.contains("CSharpCompilation.Create"));
    assert!(!frontend_model.contains("mpk.frontend.cli.v1"));
    assert!(!frontend_model.contains("FrontendEnvelope"));
}

#[test]
fn capture_harness_and_candidate_inventory_are_pinned() {
    let root = repository_root();
    let descriptor = load("release/build-inputs/csharp/build-inputs.json");
    let records = descriptor["project_files"]
        .as_array()
        .expect("project file records");
    let expected = [
        "AssemblyInfo.cs",
        "Capture.cs",
        "Cli.cs",
        "ContractAttachment.cs",
        "ContractCanonical.cs",
        "ContractModel.cs",
        "ContractParser.cs",
        "EmissionCanonical.cs",
        "EmissionModel.cs",
        "EmissionProfiles.cs",
        "FrontendDiagnostics.cs",
        "FrontendLimits.cs",
        "FrontendModel.cs",
        "FrontendProtocol.cs",
        "FrontendSuccessEmitter.cs",
        "LoweringBuilder.cs",
        "LoweringModel.cs",
        "LoweringValidation.cs",
        "NOTICE.txt",
        "Program.cs",
        "RoslynAdapters.cs",
        "RoslynSession.cs",
        "Selection.cs",
        "SourceManifestEmitter.cs",
        "SourceMapEmitter.cs",
        "SourceTransport.cs",
        "SubsetModel.cs",
        "SubsetOperations.cs",
        "SubsetSymbols.cs",
        "SubsetValidator.cs",
        "VirEmitter.cs",
        "csharp2vir.csproj",
        "csharp2vir.deps.json",
        "csharp2vir.runtimeconfig.json",
    ];
    assert_eq!(records.len(), expected.len());
    for (record, expected_path) in records.iter().zip(expected) {
        let record = object(record);
        assert_eq!(text(&record["path"]), expected_path);
        let bytes = fs::read(root.join("csharp-tools/csharp2vir").join(expected_path))
            .expect("read candidate input");
        assert_eq!(record["size_bytes"].as_u64(), Some(bytes.len() as u64));
        assert_eq!(text(&record["sha256"]), sha256(&bytes));
    }

    let harness = fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_capture_harness.cs"))
        .expect("read executable C# capture harness");
    for owner in [
        "SelectionAndHashAreExact",
        "SelectionMutationsFailClosed",
        "CliGrammarAndAssertionsAreExact",
        "CaptureIsClosedAndImmutable",
        "CaptureMutationsHaveExactIssues",
        "SourceTransportIsStrict",
        "FileAndSnapshotLimitsAreInclusive",
        "FailuresAreTypedAndArtifactFree",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }

    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    assert!(script.contains("validate_capture_implementation"));
    assert!(script.contains("run_capture_tests=True"));
    assert!(script.contains("csharp2vir-capture-tests.dll"));
}

#[test]
fn provisioned_offline_closure_executes_the_capture_harness() {
    if !cfg!(target_os = "linux") {
        return;
    }

    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let hash = text(&profile["toolchain_inputs"]["toolchain_inputs_sha256"]);
    let cache = root
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = profile["toolchain_inputs"]["archives"]
        .as_array()
        .expect("archive records");
    let present = archives
        .iter()
        .filter(|record| {
            let suffix = match text(&record["kind"]) {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!("{}{}", text(&record["id"]), suffix))
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
        .arg("--test-capture")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# capture harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
