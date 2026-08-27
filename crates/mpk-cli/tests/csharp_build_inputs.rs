use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const TOOLCHAIN_HASH: &str = "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f";
const REFERENCE_HASH: &str = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";

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

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn integer(value: &Value) -> u64 {
    value.as_u64().expect("nonnegative JSON integer")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_canonical_transport(relative: &str, value: &Value) {
    let bytes = fs::read(repository_root().join(relative)).expect("read canonical JSON");
    let mut expected = serde_json::to_vec(value).expect("serialize canonical JSON");
    expected.push(b'\n');
    assert_eq!(bytes, expected, "{relative}");
}

#[test]
fn frozen_descriptor_binds_the_profile_project_recipe_and_candidate_inventory() {
    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let descriptor = load("release/build-inputs/csharp/build-inputs.json");
    let inventory = load("release/build-inputs/csharp/candidate-inventory.json");
    assert_canonical_transport("release/build-inputs/csharp/build-inputs.json", &descriptor);
    assert_canonical_transport(
        "release/build-inputs/csharp/candidate-inventory.json",
        &inventory,
    );

    let descriptor = object(&descriptor);
    assert_eq!(
        descriptor.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "build_recipe",
            "candidate_inventory",
            "notice_sources",
            "project_files",
            "project_root",
            "schema",
            "toolchain_inputs_sha256",
            "toolchain_vector",
        ]
    );
    assert_eq!(text(&descriptor["schema"]), "mpk.csharp.build_inputs.v0");
    assert_eq!(text(&descriptor["toolchain_inputs_sha256"]), TOOLCHAIN_HASH);
    assert_eq!(
        text(&profile["toolchain_inputs"]["toolchain_inputs_sha256"]),
        TOOLCHAIN_HASH
    );
    assert_eq!(
        text(&profile["toolchain_inputs"]["reference_projection"]["inventory_sha256"]),
        REFERENCE_HASH
    );

    let expected_project_files = [
        "AssemblyInfo.cs",
        "Capture.cs",
        "Cli.cs",
        "FrontendModel.cs",
        "NOTICE.txt",
        "Program.cs",
        "RoslynAdapters.cs",
        "RoslynSession.cs",
        "Selection.cs",
        "SourceTransport.cs",
        "SubsetModel.cs",
        "SubsetOperations.cs",
        "SubsetSymbols.cs",
        "SubsetValidator.cs",
        "csharp2vir.csproj",
        "csharp2vir.deps.json",
        "csharp2vir.runtimeconfig.json",
    ];
    let project_files = array(&descriptor["project_files"]);
    assert_eq!(project_files.len(), expected_project_files.len());
    for (record, expected_path) in project_files.iter().zip(expected_project_files) {
        let record = object(record);
        assert_eq!(text(&record["path"]), expected_path);
        let bytes = fs::read(root.join("csharp-tools/csharp2vir").join(expected_path))
            .expect("read frozen C# project input");
        assert_eq!(integer(&record["size_bytes"]), bytes.len() as u64);
        assert_eq!(text(&record["sha256"]), sha256(&bytes));
    }

    let recipe = object(&descriptor["build_recipe"]);
    assert_eq!(text(&recipe["id"]), "mpk.csharp.build_recipe.csc_direct.v0");
    assert_eq!(text(&recipe["language_version"]), "14.0");
    assert_eq!(text(&recipe["target_framework"]), "net10.0");
    assert_eq!(text(&recipe["runtime_version"]), "10.0.11");
    assert_eq!(text(&recipe["network_namespace"]), "required");
    assert_eq!(text(&recipe["package_restore"]), "forbidden");
    let arguments = array(&recipe["compiler_arguments"])
        .iter()
        .map(text)
        .collect::<Vec<_>>();
    assert!(arguments.contains(&"/nologo"));
    assert!(arguments.contains(&"/deterministic+"));
    assert!(arguments.contains(&"/langversion:14.0"));
    assert!(arguments.contains(&"/warnaserror+"));
    assert!(!arguments
        .iter()
        .any(|argument| argument.contains("analyzer")));

    let inventory = object(&inventory);
    assert_eq!(
        text(&inventory["schema"]),
        "mpk.csharp.frontend_candidate_inventory.v0"
    );
    assert_eq!(text(&inventory["toolchain_inputs_sha256"]), TOOLCHAIN_HASH);
    let frontend = array(&inventory["frontend_files"]);
    assert_eq!(frontend.len(), 5);
    let frontend_paths = frontend
        .iter()
        .map(|record| text(&record["path"]))
        .collect::<Vec<_>>();
    assert_eq!(
        frontend_paths,
        [
            "frontend/Microsoft.CodeAnalysis.CSharp.dll",
            "frontend/Microsoft.CodeAnalysis.dll",
            "frontend/csharp2vir.deps.json",
            "frontend/csharp2vir.dll",
            "frontend/csharp2vir.runtimeconfig.json",
        ]
    );
    assert!(frontend
        .iter()
        .all(|record| text(&record["mode"]) == "0644"));
    assert_eq!(array(&inventory["notice_files"]).len(), 13);

    let managed = array(&profile["toolchain_inputs"]["managed_projection"]);
    for projection in managed {
        let expected_name = Path::new(text(&projection["runtime_path"]))
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let candidate = frontend
            .iter()
            .find(|record| text(&record["path"]).ends_with(expected_name))
            .expect("managed projection in candidate inventory");
        assert_eq!(candidate["size_bytes"], projection["size_bytes"]);
        assert_eq!(candidate["sha256"], projection["sha256"]);
    }
}

#[test]
fn build_input_self_test_is_closed_against_hostile_ambient_settings() {
    let root = repository_root();
    let script = root.join("scripts/build-csharp-frontend.sh");
    assert_eq!(
        fs::metadata(&script).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let output = Command::new(&script)
        .arg("--self-test")
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("HOME", "/ambient-home-must-not-be-used")
        .env("DOTNET_ROOT", "/ambient-dotnet-must-not-be-used")
        .env("NUGET_PACKAGES", "/ambient-nuget-must-not-be-used")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .output()
        .expect("run C# build-input self-test");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn candidate_remains_inert_unregistered_and_outside_the_active_release() {
    let root = repository_root();
    let project = fs::read_to_string(root.join("csharp-tools/csharp2vir/csharp2vir.csproj"))
        .expect("read C# project");
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive C# program");
    assert!(!project.contains("PackageReference"));
    assert!(!project.contains("Analyzer Include"));
    assert!(!program.contains("ParseText"));
    assert!(!program.contains("CSharpCompilation.Create"));
    assert!(!program.contains("lower"));
    assert!(program.contains("CSHARP_FRONTEND_UNAVAILABLE"));

    let registry = load("release/bundles/bundle-registry.json");
    let registry_bytes = serde_json::to_vec(&registry).unwrap();
    for forbidden in [
        b"csharp2vir".as_slice(),
        b"mpk.csharp.scalar.v0".as_slice(),
        b"mpk.semantic_profile.registry.v1".as_slice(),
    ] {
        assert!(!registry_bytes
            .windows(forbidden.len())
            .any(|window| window == forbidden));
    }

    let rejected = Command::new(root.join("scripts/build-release-bundles.sh"))
        .args(["--check", "csharp"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("active release assembler rejects C#");
    assert_eq!(rejected.status.code(), Some(64));
    assert!(rejected.stdout.is_empty());
    assert_eq!(rejected.stderr, b"BUNDLE_ASSEMBLER_USAGE\n");

    let ignored = Command::new("git")
        .current_dir(&root)
        .args([
            "check-ignore",
            "--no-index",
            "release/build-input-cache/csharp/probe",
        ])
        .output()
        .expect("check C# cache ignore rule");
    assert!(ignored.status.success());
    let tracked = Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "release/build-input-cache/csharp"])
        .output()
        .expect("check tracked C# cache files");
    assert!(tracked.status.success());
    assert!(tracked.stdout.is_empty());
}
