//! CSHARP-03-T03-W01: private practical source capture and closure gate.
//!
//! The executable harness is compiled only from the pinned Roslyn/.NET closure.
//! These Rust tests bind its exact inputs, the frozen limit vectors, and the
//! absence of any installed/public practical-profile route.

use mpk_vc::csharp_practical_vir_model::{
    csharp_practical_closed_instance_id, csharp_practical_declaration_id,
    registered_foundation_definitions_transport, registered_foundation_descriptor_transport,
    validate_registered_foundation_bundle,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORK_ITEM: &str = "CSHARP-03-T03-W01";
const OWNER: &str = "crates/mpk-cli/tests/csharp_practical_capture.rs#CSHARP-03-T03-W01";
const MANIFEST_PATH: &str = "develop/migrations/csharp-03/capture/capture-inputs.json";
const PACKAGE_PATH: &str = "develop/specs/vectors/csharp-practical-profile-v1.json";
const TOOLCHAIN_HASH: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn text(relative: &str) -> String {
    String::from_utf8(read(relative)).unwrap_or_else(|error| panic!("UTF-8 {relative}: {error}"))
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative))
        .unwrap_or_else(|error| panic!("decode {relative}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn csharp_03_t03_w01_capture_inputs_and_private_routing_are_exact() {
    let manifest = load(MANIFEST_PATH);
    let mut canonical = serde_json::to_vec(&manifest).expect("canonical manifest");
    canonical.push(b'\n');
    assert_eq!(read(MANIFEST_PATH), canonical, "manifest is canonical JSON");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w01.capture_inputs.v1"
    );
    assert_eq!(manifest["work_item"], WORK_ITEM);
    let files = manifest["files"].as_array().expect("manifest files");
    assert_eq!(files.len(), 2);
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_capture_harness.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
    ];
    for (record, expected_path) in files.iter().zip(expected) {
        assert_eq!(record["path"], expected_path);
        let bytes = read(expected_path);
        assert_eq!(record["size_bytes"].as_u64(), Some(bytes.len() as u64));
        assert_eq!(record["sha256"], sha256(&bytes));
    }

    let project = text("csharp-tools/csharp2vir/csharp2vir.csproj");
    let program = text("csharp-tools/csharp2vir/Program.cs");
    let active_bundle = text("release/bundles/bundle-registry.json");
    let active_semantic = text("release/bundles/semantic-profile-registry.json");
    assert!(project.contains("<EnableDefaultCompileItems>false</EnableDefaultCompileItems>"));
    assert!(!project.contains("PracticalCapture.cs"));
    assert!(!program.contains("CSharpPracticalCapture"));
    assert!(!active_bundle.contains("mpk.csharp_practical"));
    assert!(!active_semantic.contains("mpk.csharp.practical.v1"));

    let wrapper = text("scripts/build-csharp-practical-frontend.sh");
    let script = text("scripts/csharp_practical_build_inputs.py");
    assert!(wrapper.contains("--test-capture"));
    assert!(script.contains("def test_capture()"));
    assert!(script.contains("/main:Mpk.CSharp2Vir.PracticalCaptureHarness"));
    assert_eq!(script.matches("active.validate_project_files").count(), 14);
    assert_eq!(script.matches("copy_bound_file(").count(), 25);
    assert!(script.contains("active.materialize_closure"));
    assert!(script.contains("active.closed_dotnet_environment"));
    assert!(script.contains("active.execute_isolated"));
}

#[test]
fn csharp_03_t03_w01_gate_owns_every_frozen_capture_and_closure_rule() {
    let source = text("csharp-tools/csharp2vir/PracticalCapture.cs");
    for required in [
        "mpk.selection.csharp_members.v1",
        "PracticalCapturedInputKind",
        "PracticalSidecarFile",
        "CopyBytes",
        "new UTF8Encoding(false, true)",
        "SelectedMethodsMaximum = 32",
        "SourceTotalBytesMaximum = 16_777_216",
        "ContractFilesMaximum = 128",
        "ContractFileBytesMaximum = 1_048_576",
        "ContractTotalBytesMaximum = 8_388_608",
        "SourceDataExceptionTypesMaximum = 128",
        "RetainSourceDataExceptionType",
        "candidate = checked(current + 1)",
        "ValidateCompilerDiagnostics",
        "IgnoredCompilerDiagnostics",
        "DiagnosticSeverity.Warning",
        "ValidateDiagnosticLocation",
        "MetadataReference.CreateFromImage",
        "ValidateDependencies",
        "mpk_namespace",
        "mpk_attribute",
        "generated_source",
        "ValidateGlobalDeclarationExclusions",
        "delegate_dynamic_or_runtime_codegen",
        "ValidateGenerics",
        "user_or_constructed_generic",
        "IsExactNullable",
        "ValidateEffectsAndConcurrency",
        "external_effect_or_concurrency",
        "framework_api",
        "HasExactIntrinsicArguments",
        "IsExactIntrinsicArgument",
        "dead_declaration",
        "call_cycle",
        "type_cycle",
        "ClosedInstanceId",
        "PracticalSourceClosure",
        "Queue<SourceTypeRecord>",
        "Sidecars",
        "ArtifactCount => 0",
    ] {
        assert!(source.contains(required), "missing W01 rule {required}");
    }
    let dependency = source.find("ValidateDependencies(roslyn)").unwrap();
    let diagnostics = source.find("ValidateCompilerDiagnostics(roslyn)").unwrap();
    let declarations = source
        .find("ValidateGlobalDeclarationExclusions(roslyn)")
        .unwrap();
    let generics = source.find("ValidateGenerics(roslyn)").unwrap();
    let effects = source
        .find("ValidateEffectsAndConcurrency(roslyn)")
        .unwrap();
    assert!(dependency < diagnostics);
    assert!(diagnostics < declarations);
    assert!(declarations < generics);
    assert!(generics < effects);
    assert!(!source.contains("Microsoft.CodeAnalysis.CSharp.Scripting"));
    assert!(!source.contains("CSharpScript"));
    for unfrozen_limit in ["\"declarations\"", "\"graph_edges\"", "\"source_types\""] {
        assert!(
            !source.contains(unfrozen_limit),
            "unfrozen W01 limit {unfrozen_limit}"
        );
    }
}

#[test]
fn csharp_03_t03_w01_executable_vectors_cover_the_exit_gate() {
    let harness = text("crates/mpk-cli/tests/csharp_practical_capture_harness.cs");
    for owner in [
        "MultiFileClosureIsCompleteAndDeterministic",
        "DeadAndUnsupportedDeclarationsReject",
        "CompilerSeverityGateIsClosed",
        "SelectionCapturePathAndEncodingAreClosed",
        "MpkAndAmbientDependenciesReject",
        "DelegatesDynamicLinqReflectionAndEffectsReject",
        "GenericsNullableAndIncidentalMetadataAreClosed",
        "ClosedFrameworkTypesAreExact",
        "ConstructorInitializersEnterTheCallClosure",
        "CallAndTypeCyclesReject",
        "MethodClosureLimitIsInclusive",
        "SourceTypeLimitIsInclusive",
        "CompilerSynthesizedMarkersStayOpaque",
        "FailuresAreArtifactFree",
    ] {
        assert!(harness.contains(owner), "missing executable owner {owner}");
    }
    for mutation in [
        "PracticalCapturedInputKind.Project",
        "PracticalCapturedInputKind.Package",
        "PracticalCapturedInputKind.Binary",
        "PracticalCapturedInputKind.GeneratedSource",
        "PracticalCapturedInputKind.AnalyzerConfig",
        "PracticalCapturedInputKind.EditorConfig",
        "System.Func<int>",
        "System.Linq.Expressions.Expression",
        "System.Activator.CreateInstance",
        "System.Console.WriteLine",
        "System.IO.File.Exists",
        "System.Environment.TickCount",
        "System.Random",
        "System.Diagnostics.Stopwatch.GetTimestamp",
        "System.Threading.Tasks.Task pending",
        "System.Collections.Generic.List<int>",
        "System.Nullable<int>",
        "System.DateOnly date",
        "System.StringComparison.Ordinal",
        "public sealed class Item(int value)",
        "System.Uri value",
        "int? value",
        "required int Value",
    ] {
        assert!(harness.contains(mutation), "missing mutation {mutation}");
    }
}

#[test]
fn csharp_03_t03_w01_frozen_limit_vectors_route_only_here() {
    let package = load(PACKAGE_PATH);
    let mut owned = package["vectors"]
        .as_array()
        .expect("vectors")
        .iter()
        .filter(|vector| vector["implementation_owner"] == WORK_ITEM)
        .collect::<Vec<_>>();
    owned.sort_by_key(|vector| vector["inputs"]["value"].as_u64().unwrap());
    assert_eq!(owned.len(), 3);
    for (vector, expected_value) in owned.iter().zip([127_u64, 128, 129]) {
        assert_eq!(vector["production_test_owner"], OWNER);
        assert_eq!(vector["family"], "limit");
        assert_eq!(vector["inputs"]["counter"], "source_data_exception_types");
        assert_eq!(vector["inputs"]["inclusive_maximum"], 128);
        assert_eq!(vector["inputs"]["value"], expected_value);
    }
    assert_eq!(owned[0]["expected"]["accept"], true);
    assert_eq!(owned[1]["expected"]["accept"], true);
    assert_eq!(owned[2]["expected"]["reject"], "limit_exceeded");
}

#[test]
fn csharp_03_t03_w01_closed_instance_ids_match_the_registered_foundation() {
    let source_type_id = csharp_practical_declaration_id(&serde_json::json!({
        "kind": "type",
        "name": "Entry",
        "namespace": "Business",
        "owner": "",
        "parameter_type_ids": [],
        "result_type_id": "",
    }))
    .expect("source type ID");
    assert_eq!(
        source_type_id,
        "mpk.csharp.source.66026d6206a6760e0afdc05dd225167b5346a52361be7cf3b35b9bf904a1b657"
    );
    assert_eq!(
        csharp_practical_declaration_id(&serde_json::json!({
            "kind": "method",
            "name": "Run",
            "namespace": "Business",
            "owner": source_type_id,
            "parameter_type_ids": ["mpk.csharp.value.i32.v1"],
            "result_type_id": "mpk.csharp.value.i32.v1",
        }))
        .expect("source method ID"),
        "mpk.csharp.source.78e51d3041e153c1b3760806931b8d1ec19c38c3ef981ce2a50473edc0ad829c"
    );

    let foundation = validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .expect("registered foundation");
    let expected = [
        (
            "bounded_sequence",
            "mpk.csharp.instance.ed1833d7de995f851050cda920877e4f114c8ceedd7ad25010205e5238bc76c3",
        ),
        (
            "option",
            "mpk.csharp.instance.9f7acdcf062807a2fd9542fe16184fa682ff33a6e64bd43974b14953447f7338",
        ),
    ];
    for (template, expected_id) in expected {
        let instance = serde_json::json!({
            "kind": "instance",
            "template": template,
            "arguments": [{"kind": "primitive", "id": "i32"}],
        });
        assert_eq!(
            csharp_practical_closed_instance_id(&foundation, &instance)
                .expect("closed instance ID"),
            expected_id
        );
    }
}

#[test]
fn csharp_03_t03_w01_pinned_roslyn_harness_passes_when_cache_is_present() {
    let package = load("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"]
        .as_array()
        .expect("archives");
    // Validate the manifest shape on every host before the Linux-only runner.
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache = repository_root()
        .join("release/build-input-cache/csharp")
        .join(TOOLCHAIN_HASH)
        .join("archives");
    let present = archives
        .iter()
        .filter(|archive| {
            let suffix = match archive["kind"].as_str().expect("archive kind") {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!(
                    "{}{}",
                    archive["id"].as_str().expect("archive id"),
                    suffix
                ))
                .is_file()
        })
        .count();
    assert!(
        present == 0 || present == archives.len(),
        "partial pinned C# archive cache"
    );
    if present == 0 {
        return;
    }

    let output = Command::new(repository_root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg("--test-capture")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("run practical capture harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
