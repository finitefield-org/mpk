use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const TOOLCHAIN_HASH: &str = "a75175ba0cce86d97a8e056d4dda7a0826bb6676ba551c454bd65e5d44d23fc4";
const JAR_HASH: &str = "125ef66b3de047ca5ff8c659c1d38e8c225f1cf2975db5fb4d4b4e9c8d67c2ff";
const CLASS_INVENTORY_HASH: &str =
    "33667ae00eb8b166df7a07b32a48b6b93cdeb0d832a630a1218ed8fe6cda7397";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(relative)).unwrap()).unwrap()
}

fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_file(relative: &str) -> Value {
    let value = load(relative);
    let mut expected = canonical(&value);
    expected.push(b'\n');
    assert_eq!(fs::read(root().join(relative)).unwrap(), expected);
    value
}

fn hostile_command() -> Command {
    let mut command = Command::new(root().join("scripts/build-java-frontend.sh"));
    command
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("HOME", "/unselected/home")
        .env("TMPDIR", "/unselected/tmp")
        .env("JAVA_HOME", "/unselected/jdk")
        .env("CLASSPATH", "/unselected/injected.jar")
        .env("JAVA_TOOL_OPTIONS", "-javaagent:/unselected/agent.jar")
        .env("JDK_JAVA_OPTIONS", "--patch-module=java.base=/unselected")
        .env("JDK_JAVAC_OPTIONS", "-processor unselected.Processor")
        .env("_JAVA_OPTIONS", "-Xmx1m")
        .env("DOCKER_HOST", "tcp://127.0.0.1:1")
        .env("DOCKER_CONTEXT", "unselected")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1");
    command
}

#[test]
fn java_build_descriptor_binds_sources_recipe_and_measured_candidate() {
    let descriptor = canonical_file("release/build-inputs/java/build-inputs.json");
    let inventory = canonical_file("release/build-inputs/java/candidate-inventory.json");
    let profile = load("develop/specs/vectors/java-profile-v0.json");
    assert_eq!(descriptor["schema"], "mpk.java.build_inputs.v0");
    assert_eq!(descriptor["project_root"], "java-tools/java2vir");
    assert_eq!(descriptor["toolchain_inputs_sha256"], TOOLCHAIN_HASH);
    assert_eq!(
        profile["toolchain_inputs"]["toolchain_inputs_sha256"],
        TOOLCHAIN_HASH
    );
    let records = descriptor["project_files"].as_array().unwrap();
    let paths = [
        "META-INF/MANIFEST.MF",
        "NOTICE.txt",
        "src/mpk/java2vir/BuildIdentity.java",
        "src/mpk/java2vir/CanonicalJson.java",
        "src/mpk/java2vir/CapturedSnapshot.java",
        "src/mpk/java2vir/ClosedFileManager.java",
        "src/mpk/java2vir/CompilerDiagnostics.java",
        "src/mpk/java2vir/CompilerSession.java",
        "src/mpk/java2vir/DiagnosticRegistry.java",
        "src/mpk/java2vir/FrontendFailure.java",
        "src/mpk/java2vir/FrontendLimits.java",
        "src/mpk/java2vir/JavaAdmission.java",
        "src/mpk/java2vir/JavaContracts.java",
        "src/mpk/java2vir/JavaEmission.java",
        "src/mpk/java2vir/JavaFrontend.java",
        "src/mpk/java2vir/JavaIr.java",
        "src/mpk/java2vir/JavaLowering.java",
        "src/mpk/java2vir/JavaLoweringValidation.java",
        "src/mpk/java2vir/JavaSourceMaps.java",
        "src/mpk/java2vir/JavaSubset.java",
        "src/mpk/java2vir/Main.java",
        "src/mpk/java2vir/Protocol.java",
        "src/mpk/java2vir/ScalarType.java",
        "src/mpk/java2vir/Selection.java",
        "src/mpk/java2vir/SourceText.java",
        "src/mpk/java2vir/SourceTokens.java",
        "src/mpk/java2vir/StrictJson.java",
        "src/mpk/java2vir/TreeInventory.java",
    ];
    assert_eq!(records.len(), paths.len());
    for (record, path) in records.iter().zip(paths) {
        assert_eq!(record["path"], path);
        let file = root().join("java-tools/java2vir").join(path);
        let bytes = fs::read(&file).unwrap();
        assert_eq!(record["size_bytes"], bytes.len() as u64);
        assert_eq!(record["sha256"], digest(&bytes));
        assert_eq!(record["mode"], "0644");
        assert_eq!(
            fs::metadata(file).unwrap().permissions().mode() & 0o777,
            0o644
        );
    }
    let recipe = &descriptor["build_recipe"];
    assert_eq!(recipe["id"], "mpk.java.build_recipe.javac_direct.v0");
    assert_eq!(recipe["image"], profile["toolchain_inputs"]["native_image"]);
    assert_eq!(recipe["platform"], "linux/amd64");
    assert_eq!(recipe["network"], "none");
    assert_eq!(recipe["package_restore"], "forbidden");
    assert_eq!(
        recipe["compiler_arguments"],
        json!([
            "--release",
            "25",
            "-encoding",
            "UTF-8",
            "-g:none",
            "-proc:none",
            "-implicit:none",
            "-Xlint:all",
            "-Werror",
            "--class-path",
            "/work/empty",
            "--source-path",
            "/work/empty",
            "--processor-path",
            "/work/empty",
            "--module-path",
            "/work/empty",
            "-d",
            "/work/classes"
        ])
    );
    assert_eq!(recipe["jar"]["timestamp"], json!([1980, 1, 1, 0, 0, 0]));
    assert_eq!(recipe["jar"]["compression"], "stored");
    assert_eq!(recipe["jar"]["class_path"], "forbidden");
    assert_eq!(recipe["jar"]["service_providers"], "forbidden");
    assert_eq!(
        inventory["schema"],
        "mpk.java.frontend_candidate_inventory.v0"
    );
    assert_eq!(inventory["toolchain_inputs_sha256"], TOOLCHAIN_HASH);
    assert_eq!(
        inventory["project_files_sha256"],
        digest(&canonical(&descriptor["project_files"]))
    );
    assert_eq!(inventory["build_recipe_sha256"], digest(&canonical(recipe)));
    let classes = inventory["class_files"].as_array().unwrap();
    assert_eq!(
        digest(&canonical(&inventory["class_files"])),
        CLASS_INVENTORY_HASH
    );
    assert_eq!(
        classes
            .iter()
            .map(|record| record["path"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "mpk/java2vir/BuildIdentity.class",
            "mpk/java2vir/CanonicalJson$ByteCounter.class",
            "mpk/java2vir/CanonicalJson$Sink.class",
            "mpk/java2vir/CanonicalJson$Writer.class",
            "mpk/java2vir/CanonicalJson.class",
            "mpk/java2vir/CapturedSnapshot$Input.class",
            "mpk/java2vir/CapturedSnapshot$Pass.class",
            "mpk/java2vir/CapturedSnapshot$Stat.class",
            "mpk/java2vir/CapturedSnapshot.class",
            "mpk/java2vir/ClosedFileManager$1.class",
            "mpk/java2vir/ClosedFileManager.class",
            "mpk/java2vir/CompilerDiagnostics$Raw.class",
            "mpk/java2vir/CompilerDiagnostics.class",
            "mpk/java2vir/CompilerSession$RejectingWriter.class",
            "mpk/java2vir/CompilerSession.class",
            "mpk/java2vir/DiagnosticRegistry$Definition.class",
            "mpk/java2vir/DiagnosticRegistry.class",
            "mpk/java2vir/FrontendFailure$Issue.class",
            "mpk/java2vir/FrontendFailure$Span.class",
            "mpk/java2vir/FrontendFailure.class",
            "mpk/java2vir/FrontendLimits$Limit.class",
            "mpk/java2vir/FrontendLimits.class",
            "mpk/java2vir/JavaAdmission$Program.class",
            "mpk/java2vir/JavaAdmission.class",
            "mpk/java2vir/JavaContracts$Attached.class",
            "mpk/java2vir/JavaContracts$ClosureCounter.class",
            "mpk/java2vir/JavaContracts$ContractSet.class",
            "mpk/java2vir/JavaContracts$Expr.class",
            "mpk/java2vir/JavaContracts$MethodCounter.class",
            "mpk/java2vir/JavaContracts$Parsed.class",
            "mpk/java2vir/JavaContracts$Pending.class",
            "mpk/java2vir/JavaContracts$Typed.class",
            "mpk/java2vir/JavaContracts$Validated.class",
            "mpk/java2vir/JavaContracts.class",
            "mpk/java2vir/JavaEmission$Identity.class",
            "mpk/java2vir/JavaEmission.class",
            "mpk/java2vir/JavaFrontend$Result.class",
            "mpk/java2vir/JavaFrontend.class",
            "mpk/java2vir/JavaIr$1.class",
            "mpk/java2vir/JavaIr$Block.class",
            "mpk/java2vir/JavaIr$ClosureCounter.class",
            "mpk/java2vir/JavaIr$Edge.class",
            "mpk/java2vir/JavaIr$Function.class",
            "mpk/java2vir/JavaIr$Instruction.class",
            "mpk/java2vir/JavaIr$MethodCounter.class",
            "mpk/java2vir/JavaIr$Origin.class",
            "mpk/java2vir/JavaIr$Program.class",
            "mpk/java2vir/JavaIr$Terminator.class",
            "mpk/java2vir/JavaIr$Type.class",
            "mpk/java2vir/JavaIr$Value.class",
            "mpk/java2vir/JavaIr.class",
            "mpk/java2vir/JavaLowering$1.class",
            "mpk/java2vir/JavaLowering$Builder.class",
            "mpk/java2vir/JavaLowering$Demand.class",
            "mpk/java2vir/JavaLowering$Draft.class",
            "mpk/java2vir/JavaLowering$Expression.class",
            "mpk/java2vir/JavaLowering.class",
            "mpk/java2vir/JavaLoweringValidation.class",
            "mpk/java2vir/JavaSourceMaps.class",
            "mpk/java2vir/JavaSubset$1.class",
            "mpk/java2vir/JavaSubset$Binding.class",
            "mpk/java2vir/JavaSubset$Body.class",
            "mpk/java2vir/JavaSubset$Closure.class",
            "mpk/java2vir/JavaSubset$Declaration.class",
            "mpk/java2vir/JavaSubset$Method.class",
            "mpk/java2vir/JavaSubset.class",
            "mpk/java2vir/Main.class",
            "mpk/java2vir/Protocol.class",
            "mpk/java2vir/ScalarType.class",
            "mpk/java2vir/Selection.class",
            "mpk/java2vir/SourceText.class",
            "mpk/java2vir/SourceTokens.class",
            "mpk/java2vir/StrictJson$Cursor.class",
            "mpk/java2vir/StrictJson$Frame.class",
            "mpk/java2vir/StrictJson$Value.class",
            "mpk/java2vir/StrictJson.class",
            "mpk/java2vir/TreeInventory$1.class",
            "mpk/java2vir/TreeInventory$Node.class",
            "mpk/java2vir/TreeInventory$Origins$Origin.class",
            "mpk/java2vir/TreeInventory$Origins.class",
            "mpk/java2vir/TreeInventory$Pending.class",
            "mpk/java2vir/TreeInventory.class",
        ]
    );
    // The inactive main/provider check stay byte-identical to T02.
    assert_eq!(
        classes[0]["sha256"],
        "2cc9db1b015cbb6988fe39ef16f59144bf627a4156ce5bc560899779934c8a52"
    );
    assert_eq!(
        classes
            .iter()
            .find(|record| record["path"] == "mpk/java2vir/Main.class")
            .unwrap()["sha256"],
        "e7f7f8b630e05d8565ce8173c85b920046180a9d550aa2ec42e4c882c6e7e102"
    );
    assert_eq!(
        inventory["frontend_files"],
        json!([{"path":"java2vir.jar", "mode":"0644", "size_bytes":313051, "sha256":JAR_HASH}])
    );
    assert_eq!(inventory["notice_files"][0]["sha256"], records[1]["sha256"]);
}

#[test]
fn java_offline_input_owner_executes_hostile_input_tests_without_ambient_configuration() {
    let temporary = tempfile::tempdir().unwrap();
    let marker = temporary.path().join("python-startup-ran");
    fs::write(
        temporary.path().join("sitecustomize.py"),
        format!(
            "open({:?}, 'w').write('unexpected')\n",
            marker.to_str().unwrap()
        ),
    )
    .unwrap();
    let output = hostile_command()
        .env("PYTHONPATH", temporary.path())
        .env("PYTHONSTARTUP", temporary.path().join("sitecustomize.py"))
        .arg("--self-test")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(!marker.exists());
}

#[test]
fn java_candidate_does_not_install_a_release_or_accept_toolchain_overrides() {
    let installed = mpk_vc::semantic_profile_registry::validate_semantic_profile_registry(
        &fs::read(root().join("release/bundles/semantic-profile-registry.json")).unwrap(),
        mpk_vc::semantic_profile_registry::RegistryRevision::Revision2,
    )
    .unwrap();
    assert!(installed.lookup("java", "mpk.java.scalar.v0").is_none());
    let active = load("release/bundles/semantic-profile-registry.json");
    assert_eq!(
        active,
        load("develop/specs/vectors/semantic-profile-registry-v2.json")["registry"]
    );
    let registry = load("release/bundles/bundle-registry.json");
    let tuples = registry["tuples"].as_array().unwrap();
    assert_eq!(tuples.len(), 4);
    assert!(tuples.iter().all(|tuple| {
        matches!(
            tuple["semantic_context"]["source_language"].as_str(),
            Some("go" | "rust" | "csharp")
        )
    }));
    for arguments in [
        vec![],
        vec!["--jdk-dir", "/unselected/jdk"],
        vec!["--java-home", "/unselected/jdk"],
        vec!["--classpath", "/unselected/injected.jar"],
        vec!["--check", "--download"],
        vec!["--build"],
    ] {
        let output = hostile_command().args(arguments).output().unwrap();
        assert_eq!(output.status.code(), Some(64));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"JAVA_BUILD_USAGE\n");
    }
    let existing = tempfile::tempdir().unwrap();
    let output = hostile_command()
        .arg("--build")
        .arg(existing.path().canonicalize().unwrap())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"JAVA_BUILD_OUTPUT\n");
}

#[test]
fn compiled_java_contracts_do_not_open_public_source_routes() {
    let temporary = tempfile::tempdir().unwrap();
    let profile = load("develop/specs/vectors/java-profile-v0.json");
    let context_path = temporary.path().join("context.json");
    let selection_path = temporary.path().join("selection.json");
    let output_path = temporary.path().join("must-not-exist.json");
    fs::write(&selection_path, canonical(&profile["selection_fixture"])).unwrap();
    for use_installed_identity in [false, true] {
        let mut context = profile["semantic_context_fixture"].clone();
        if use_installed_identity {
            context["profile_registry"] = serde_json::to_value(
                mpk_vc::semantic_profile_registry::RegistryRevision::Revision2.identity(),
            )
            .unwrap();
        }
        fs::write(&context_path, canonical(&context)).unwrap();
        for (command, flag) in [
            (vec!["policy", "scan"], "--json-out"),
            (vec!["policy", "verify"], "--evidence-json"),
            (vec!["explain"], "--request-json-out"),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
                .args(command)
                .arg(temporary.path().join("missing-source-root"))
                .arg("--semantic-context")
                .arg(&context_path)
                .arg("--selection")
                .arg(&selection_path)
                .arg(flag)
                .arg(&output_path)
                .env("JAVA_HOME", "/unselected/jdk")
                .env(
                    "MPK_SEMANTIC_REGISTRY",
                    root().join("develop/specs/vectors/semantic-profile-registry-v3.json"),
                )
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(1), // Preserve the installed CLI's semantic-input error exit code.
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(if use_installed_identity {
                    "SEMANTIC_PROFILE_UNKNOWN"
                } else {
                    "SEMANTIC_REGISTRY_ASSERTION"
                }),
                "{error}"
            );
            assert!(!output_path.exists());
            assert!(!temporary.path().join("missing-source-root").exists());
        }
    }
}

#[test]
#[ignore = "requires the provisioned fixed JDK archive and local pinned Docker image; never downloads"]
fn offline_java_candidate_builds_twice_and_refuses_ambient_options() {
    let temporary = tempfile::tempdir().unwrap();
    let destination = temporary.path().canonicalize().unwrap().join("candidate");
    let output = hostile_command()
        .args(["--build", destination.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    let expected = load("release/build-inputs/java/candidate-inventory.json");
    assert_eq!(
        digest(&fs::read(destination.join("java2vir.jar")).unwrap()),
        JAR_HASH
    );
    let mut transport = canonical(&expected);
    transport.push(b'\n');
    assert_eq!(
        fs::read(destination.join("build-manifest.json")).unwrap(),
        transport
    );
    assert_eq!(
        fs::read(destination.join("notices/NOTICE.txt")).unwrap(),
        fs::read(root().join("java-tools/java2vir/NOTICE.txt")).unwrap()
    );
    let mut files = fs::read_dir(&destination)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, ["build-manifest.json", "java2vir.jar", "notices"]);
}
