//! CSHARP-03-T03-W03: private immutable declarations, enums and recursive defaults.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORK_ITEM: &str = "CSHARP-03-T03-W03";
const OWNER: &str = "crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W03";
const MANIFEST: &str = "develop/migrations/csharp-03/types/types-inputs.json";
const PACKAGE: &str = "develop/specs/vectors/csharp-practical-profile-v1.json";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(path: &str) -> Vec<u8> {
    fs::read(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}
fn text(path: &str) -> String {
    String::from_utf8(read(path)).unwrap()
}
fn json(path: &str) -> Value {
    serde_json::from_slice(&read(path)).unwrap()
}

#[test]
fn csharp_03_t03_w03_exact_manifest_and_private_routing() {
    let manifest = json(MANIFEST);
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(MANIFEST));
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w03.types_inputs.v1"
    );
    assert_eq!(manifest["work_item"], WORK_ITEM);
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_types_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ];
    let records = manifest["files"].as_array().unwrap();
    assert_eq!(records.len(), expected.len());
    for (record, path) in records.iter().zip(expected) {
        let bytes = read(path);
        assert_eq!(record["path"], path);
        assert_eq!(record["size_bytes"], bytes.len());
        assert_eq!(record["sha256"], format!("{:x}", Sha256::digest(&bytes)));
    }
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "release/bundles/bundle-registry.json",
        "release/bundles/semantic-profile-registry.json",
    ] {
        let source = text(path);
        assert!(
            !source.contains("PracticalDataTypes"),
            "active route in {path}"
        );
        assert!(!source.contains("mpk.csharp_practical.data_types.v1"));
    }
    let wrapper = text("scripts/build-csharp-practical-frontend.sh");
    let script = text("scripts/csharp_practical_build_inputs.py");
    assert!(wrapper.contains("--test-types"));
    for marker in [
        "def validate_types_inputs_value",
        "def test_types()",
        "/main:Mpk.CSharp2Vir.PracticalTypesHarness",
        "CSHARP_PRACTICAL_TYPES_TEST_BUILD",
        "CSHARP_PRACTICAL_TYPES_TEST_FAILURE",
    ] {
        assert!(script.contains(marker));
    }
}

#[test]
fn csharp_03_t03_w03_frozen_limits_have_exact_executable_owners() {
    let package = json(PACKAGE);
    let owned = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 6);
    for (counter, maximum) in [
        ("fields_properties_per_type", 32),
        ("structural_type_nesting", 16),
    ] {
        for (suffix, value) in [
            ("below", maximum - 1),
            ("at", maximum),
            ("above", maximum + 1),
        ] {
            let id = format!("limit.practical.{counter}.{suffix}");
            let vector = owned.iter().find(|vector| vector["id"] == id).unwrap();
            assert_eq!(vector["production_test_owner"], OWNER);
            assert_eq!(vector["inputs"]["counter"], counter);
            assert_eq!(vector["inputs"]["inclusive_maximum"], maximum);
            assert_eq!(vector["inputs"]["value"], value);
            if suffix == "above" {
                assert_eq!(vector["expected"]["reject"], "limit_exceeded");
            } else {
                assert_eq!(vector["expected"]["accept"], true);
            }
        }
    }
    let owner = package["downstream_work_item_owners"]
        .as_array()
        .unwrap()
        .iter()
        .find(|owner| owner["work_item"] == WORK_ITEM)
        .unwrap();
    assert_eq!(owner["primary_test_owner"], OWNER);
    assert!(owner["owns"]
        .as_str()
        .unwrap()
        .contains("T04-W04 is the sole owner"));
}

#[test]
fn csharp_03_t03_w03_harness_covers_shapes_defaults_limits_and_runtime() {
    let harness = text("crates/mpk-cli/tests/csharp_practical_types_harness.cs");
    for required in [
        "DeclarationAndMemberMatrix",
        "EnumCarrierMatrix",
        "RecursiveDefaults",
        "ForbiddenDeclarations",
        "IdentityMutationAndEnumEscapes",
        "SourceExceptionsAreClassificationOnly",
        "FrozenLimitsAreInclusive",
        "ArtifactsAndPrecedence",
        "SourceRuntimeDifferential",
        "18446744073709551615",
        "-9223372036854775808",
        "System.DayOfWeek",
        "ClassifySourceException",
        "System.IO.IOException",
        "OPAQUE_SIDECAR_IS_NOT_INVARIANT_PROOF",
        "COMPUTED_GETTER_NOT_STORAGE",
        "new[] { 31, 32, 33 }",
        "new[] { 15, 16, 17 }",
        "RUNTIME_CONSTRUCTOR_DISTINCT",
        "RUNTIME_ENUM_CARRIER",
        "default_invariant_pending",
        "default(Data?)",
        "new Data()",
        "return default;",
    ] {
        assert!(
            harness.contains(required),
            "missing executable case {required}"
        );
    }
    let capture = text("csharp-tools/csharp2vir/PracticalCapture.cs");
    let limits = capture.find("validateDataLimits?.Invoke").unwrap();
    assert!(limits < capture.find("ValidateDependencies(roslyn)").unwrap());
    let declaration = capture.find("validateDataDeclarations?.Invoke").unwrap();
    let types = capture.find("validateDataTypes?.Invoke").unwrap();
    assert!(
        capture
            .find("ValidateGlobalDeclarationExclusions(roslyn)")
            .unwrap()
            < declaration
    );
    assert!(declaration < types);
    assert!(types < capture.find("ValidateGenerics(roslyn)").unwrap());
    assert!(
        types
            < capture
                .find("ValidateEffectsAndConcurrency(roslyn)")
                .unwrap()
    );
}

#[test]
fn csharp_03_t03_w03_pinned_roslyn_harness_passes_when_cache_is_present() {
    run_pinned_harness("--test-types");
}

fn run_pinned_harness(argument: &str) {
    if !cfg!(target_os = "linux") {
        return;
    }
    let package = json(PACKAGE);
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    let cache = root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            let suffix = match archive["kind"].as_str().unwrap() {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("archive kind {kind}"),
            };
            cache
                .join(format!("{}{suffix}", archive["id"].as_str().unwrap()))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned archive cache"
    );
    if count == 0 {
        return;
    }
    let output = Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg(argument)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn csharp_03_t03_w04_constructor_manifest_and_private_route() {
    let path = "develop/migrations/csharp-03/construction/construction-inputs.json";
    let manifest = json(path);
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W04");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w04.construction_inputs.v1"
    );
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_construction_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ];
    assert_eq!(manifest["files"].as_array().unwrap().len(), expected.len());
    for (record, path) in manifest["files"].as_array().unwrap().iter().zip(expected) {
        let bytes = read(path);
        assert_eq!(record["path"], path);
        assert_eq!(record["size_bytes"], bytes.len());
        assert_eq!(record["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!text(path).contains("PracticalConstruction"));
    }
    let capture = text("csharp-tools/csharp2vir/PracticalCapture.cs");
    assert!(
        capture.find("BuildClosure(\n").unwrap()
            < capture.find("validateConstruction?.Invoke").unwrap()
    );
    assert!(
        capture.find("validateConstruction?.Invoke").unwrap()
            < capture
                .find("ValidateEffectsAndConcurrency(roslyn)")
                .unwrap()
    );
}

#[test]
fn csharp_03_t03_w04_frozen_constructor_limits() {
    let package = json(PACKAGE);
    let owned: Vec<_> = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|vector| vector["implementation_owner"] == "CSHARP-03-T03-W04")
        .collect();
    assert_eq!(owned.len(), 3);
    for (suffix, value) in [("below", 7), ("at", 8), ("above", 9)] {
        let id = format!("limit.practical.constructors_per_type.{suffix}");
        let vector = owned.iter().find(|vector| vector["id"] == id).unwrap();
        assert_eq!(
            vector["production_test_owner"],
            "crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W04"
        );
        assert_eq!(vector["inputs"]["inclusive_maximum"], 8);
        assert_eq!(vector["inputs"]["value"], value);
        if value == 9 {
            assert_eq!(vector["expected"]["reject"], "limit_exceeded");
        } else {
            assert_eq!(vector["expected"]["accept"], true);
        }
    }
    for marker in [
        "InvariantAttachment",
        "SynthesizedMutations",
        "RuntimeEquivalence",
        "RejectionMatrix",
        "DELEGATION_STATE",
        "COMPLETE_CLAIM_EMISSION",
        "NULL_CHECK_POINT",
        "new[] {7,8,9}",
    ] {
        assert!(
            text("crates/mpk-cli/tests/csharp_practical_construction_harness.cs").contains(marker),
            "{marker}"
        );
    }
}

#[test]
fn csharp_03_t03_w04_pinned_construction_harness() {
    run_pinned_harness("--test-construction");
}

#[test]
fn csharp_03_t03_w05_initialization_manifest_and_private_route() {
    let path = "develop/migrations/csharp-03/initialization/initialization-inputs.json";
    let manifest = json(path);
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w05.initialization_inputs.v1"
    );
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W05");
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(read(path), canonical);
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_initialization_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
    ];
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), expected.len());
    for (file, path) in files.iter().zip(expected) {
        let bytes = read(path);
        assert_eq!(file["path"], path);
        assert_eq!(file["size_bytes"], bytes.len());
        assert_eq!(file["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!text(path).contains("PracticalInitialization"));
    }
    for marker in [
        "OrderedTransactions",
        "RequiredAndDefaults",
        "DuplicateAndMutationMatrix",
        "InvariantSites",
        "RuntimeOrderAndDiscard",
        "NestedCreationInConstructor",
        "PUBLISH_ONLY_AFTER_CHECK",
        "RUNTIME_EXCEPTION_ORDER",
    ] {
        assert!(
            text("crates/mpk-cli/tests/csharp_practical_initialization_harness.cs")
                .contains(marker)
        );
    }
    let package = json(PACKAGE);
    let owner = package["downstream_work_item_owners"]
        .as_array()
        .unwrap()
        .iter()
        .find(|owner| owner["work_item"] == "CSHARP-03-T03-W05")
        .unwrap();
    assert_eq!(
        owner["primary_test_owner"],
        "crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W05"
    );
}

#[test]
fn csharp_03_t03_w05_pinned_initialization_harness() {
    run_pinned_harness("--test-initialization");
}
