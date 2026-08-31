//! T07 owner. Ordinary runs check pure release/launcher invariants. The
//! explicit native gate copies this test executable to its private bin/mpk;
//! neither that entrypoint nor the Java registry is a public installed route.

use mpk_cli::frontend_protocol;
#[path = "../src/frontend_registry.rs"]
#[allow(unused_imports)]
mod frontend_registry;
#[path = "../src/frontend_sandbox.rs"]
#[allow(unused_imports)]
mod frontend_sandbox;
#[path = "../src/java_frontend_runner.rs"]
mod java_frontend_runner;

use java_frontend_runner::{JavaRunError, PreparedJavaRun};
use mpk_vc::java_release;
use mpk_vc::release_bundle_v1::{
    successor_release_registry_hash, validate_successor_bundle_candidate,
    validate_successor_release_registry, SuccessorReleaseSelectionRequest,
};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::{CapturedInput, InputKind};
use serde_json::{json, Value};
use std::process::ExitCode;

const VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/java-profile-v0.json");
const SEMANTIC: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v3.json");
const CANDIDATE: &[u8] = include_bytes!("../../../release/build-inputs/java/bundle-candidate.json");
const REGISTRY: &[u8] = include_bytes!("../../../release/build-inputs/java/bundle-registry.json");

fn line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn check_candidates_and_launcher() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let check = std::process::Command::new(root.join("scripts/build-java-candidate.sh"))
        .arg("--check")
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("JAVA_HOME", "/host/jdk")
        .env("CLASSPATH", "/host/classes")
        .env("JAVA_TOOL_OPTIONS", "-javaagent:/host/agent.jar")
        .output()
        .unwrap();
    assert!(
        check.status.success() && check.stdout.is_empty() && check.stderr.is_empty(),
        "candidate reconstruction: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let trace = std::process::Command::new(root.join("scripts/check-java-runner.sh"))
        .arg("--check-trace-parser")
        .env_clear()
        .output()
        .unwrap();
    assert!(
        trace.status.success() && trace.stdout.is_empty() && trace.stderr.is_empty(),
        "JVM trace attribution: {}",
        String::from_utf8_lossy(&trace.stderr)
    );
    let vector: Value = serde_json::from_slice(VECTORS).unwrap();
    let semantic_vector: Value = serde_json::from_slice(SEMANTIC).unwrap();
    let semantic = validate_semantic_profile_registry(
        &line(&semantic_vector["registry"]),
        RegistryRevision::Revision3,
    )
    .unwrap();
    let candidate = validate_successor_bundle_candidate(CANDIDATE, &semantic).unwrap();
    let registry = validate_successor_release_registry(REGISTRY, &semantic).unwrap();
    assert_eq!(candidate.candidate(), java_release::candidate());
    assert_eq!(registry.registry(), java_release::registry());
    assert_eq!(
        candidate.candidate().toolchain_bundles[0]
            .inventory
            .files
            .len(),
        405
    );
    let resolved = registry
        .resolve(
            &semantic,
            SuccessorReleaseSelectionRequest {
                semantic_context: &vector["semantic_context_fixture"],
                frontend_bundle_id: java_release::FRONTEND_ID,
                toolchain_bundle_id: java_release::TOOLCHAIN_ID,
            },
        )
        .unwrap();
    let selection = validate_registry_selection_envelope(
        &semantic,
        &resolved.semantic_context,
        &json!({"schema":"mpk.selection.java_methods.v0", "value": {
            "compilation":"vector", "sources":["src/vector/Case.java"],
            "contracts":["contracts/selected.json"], "methods":["vector.Case::f(int)->int"]}}),
    )
    .unwrap();
    let launcher = java_release::launcher_plan(&registry, &resolved, &selection).unwrap();
    assert_eq!(
        json!(java_release::ARGV_PREFIX),
        vector["launcher_contract"]["argv_prefix"]
    );
    assert_eq!(
        json!(launcher.environment()),
        vector["launcher_contract"]["environment"]
    );
    let template = vector["launcher_contract"]["frontend_argv_template"]
        .as_array()
        .unwrap();
    let substitutions = [
        ("{selection.compilation}", "vector"),
        (
            "{each selection.sources in stored order}",
            "src/vector/Case.java",
        ),
        (
            "{each selection.contracts in stored order}",
            "contracts/selected.json",
        ),
        (
            "{each selection.methods in stored order}",
            "vector.Case::f(int)->int",
        ),
        ("{release.frontend_bundle_id}", java_release::FRONTEND_ID),
        (
            "{release.frontend_sha256}",
            &resolved.frontend.main.binary_sha256,
        ),
        ("{release.registry_sha256}", registry.registry_sha256()),
        ("{release.toolchain_bundle_id}", java_release::TOOLCHAIN_ID),
        (
            "{release.toolchain_distribution_sha256}",
            &resolved.toolchain.distribution_sha256,
        ),
    ];
    let mut expected = java_release::ARGV_PREFIX
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    for token in template {
        let value = token.as_str().unwrap();
        expected.push(
            substitutions
                .iter()
                .find(|(key, _)| *key == value)
                .map_or(value, |(_, replacement)| *replacement)
                .to_owned(),
        );
    }
    assert_eq!(launcher.argv(), expected);
    let multiple = validate_registry_selection_envelope(
        &semantic,
        &resolved.semantic_context,
        &json!({
        "schema":"mpk.selection.java_methods.v0", "value": {
            "compilation":"vector", "sources":["src/vector/Alpha.java", "src/vector/Case.java"],
            "contracts":["contracts/a.json", "contracts/z.json"],
            "methods":["vector.Alpha::f(int)->int", "vector.Case::f(int)->int"]}}),
    )
    .unwrap();
    let multiple_plan = java_release::launcher_plan(&registry, &resolved, &multiple).unwrap();
    let pairs = multiple_plan
        .argv()
        .windows(2)
        .filter(|pair| ["--source", "--contract", "--method"].contains(&pair[0].as_str()))
        .map(|pair| (pair[0].as_str(), pair[1].as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        pairs,
        [
            ("--source", "src/vector/Alpha.java"),
            ("--source", "src/vector/Case.java"),
            ("--contract", "contracts/a.json"),
            ("--contract", "contracts/z.json"),
            ("--method", "vector.Alpha::f(int)->int"),
            ("--method", "vector.Case::f(int)->int"),
        ]
    );
    let isolation_ids = vector["isolation_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        isolation_ids,
        [
            "isolation.no_network",
            "isolation.no_ambient_environment",
            "isolation.no_agent",
            "isolation.no_java_override",
            "isolation.no_classpath_override",
            "isolation.no_jar_launcher",
            "isolation.no_manifest_classpath",
            "isolation.no_unknown_native",
            "isolation.no_host_proc",
            "isolation.no_writable_inputs",
            "isolation.no_swap",
            "isolation.no_unbounded_cgroup",
            "isolation.no_candidate_execution",
            "isolation.no_restore",
            "isolation.no_plugins"
        ]
    );
    // Repairing outer hashes cannot replace the pinned JAR, JVM, native
    // closure, budgets, host ABI, or layout with a caller-selected variant.
    for (pointer, replacement) in [
        ("/frontend_bundles/0/main/path", json!("evil.jar")),
        (
            "/frontend_bundles/0/main/binary_sha256",
            json!("0".repeat(64)),
        ),
        (
            "/toolchain_bundles/0/components/0/path",
            json!("jdk/bin/javac"),
        ),
        (
            "/toolchain_bundles/0/distribution_sha256",
            json!("1".repeat(64)),
        ),
        (
            "/toolchain_bundles/0/execution_host_profile_id",
            json!("mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"),
        ),
        ("/execution_host_profiles/0/abi", json!("glibc-2.27")),
        (
            "/execution_host_profiles/0/required_primitives/0",
            json!("cgroup2.optional"),
        ),
        (
            "/native_runtime_layout_profiles/0/runtime_root",
            json!("/usr/lib"),
        ),
    ] {
        let mut changed: Value = serde_json::from_slice(REGISTRY).unwrap();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        changed["registry_sha256"] = json!(successor_release_registry_hash(&changed).unwrap());
        assert!(
            validate_successor_release_registry(&line(&changed), &semantic).is_err(),
            "{pointer}"
        );
    }
    assert!(
        PreparedJavaRun::open().is_err(),
        "an ordinary test path must not be an installed release"
    );
}

fn native_case(id: &str) -> Result<(), String> {
    // This must happen before looking at any source or contract fixture.
    let prepared = PreparedJavaRun::open().map_err(|error| format!("JAVA_RUN_{error:?}"))?;
    let vector: Value = serde_json::from_slice(VECTORS).unwrap();
    let case = vector["accepted_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .ok_or_else(|| "unknown native test case".to_owned())?;
    let mut storage = Vec::new();
    for (path, value) in case["sources"].as_object().unwrap() {
        storage.push((
            InputKind::Source,
            path.to_owned(),
            value.as_str().unwrap().as_bytes().to_vec(),
        ));
    }
    for (index, contract) in case["contracts"].as_array().unwrap().iter().enumerate() {
        storage.push((
            InputKind::Contract,
            format!("contracts/c{index}.json"),
            line(contract),
        ));
    }
    let paths = |kind| {
        storage
            .iter()
            .filter(|(actual, _, _)| *actual == kind)
            .map(|(_, path, _)| path.clone())
            .collect::<Vec<_>>()
    };
    let selection = json!({"schema":"mpk.selection.java_methods.v0", "value": {
        "compilation":"vector", "sources":paths(InputKind::Source), "contracts":paths(InputKind::Contract),
        "methods":case["methods"]}});
    let captured = storage
        .iter()
        .map(|(kind, path, bytes)| CapturedInput {
            kind: *kind,
            normalized_path: path,
            bytes,
        })
        .collect::<Vec<_>>();
    let result = prepared
        .run(&selection, &captured)
        .map_err(|error| format!("JAVA_RUN_{error:?}"))?;
    if result.status() != "ir-lowered" {
        return Err(format!(
            "Java native case {id} failed: {}",
            String::from_utf8_lossy(result.canonical_bytes())
        ));
    }
    use std::io::Write;
    std::io::stdout()
        .write_all(result.canonical_bytes())
        .map_err(|_| "native report write failed".to_owned())
}

fn poison_environment() {
    // These are introduced after the trusted test executable has started, so
    // LD_PRELOAD tests child inheritance without poisoning the host loader.
    for (key, value) in [
        ("JAVA_HOME", "/host/jdk"),
        ("CLASSPATH", "/host/classes"),
        ("JAVA_TOOL_OPTIONS", "-javaagent:/host/agent.jar"),
        ("JDK_JAVA_OPTIONS", "--patch-module=java.base=/host"),
        ("JDK_JAVAC_OPTIONS", "-processor poison.Processor"),
        ("_JAVA_OPTIONS", "-Xmx1m"),
        ("LD_PRELOAD", "/host/preload.so"),
        ("LD_LIBRARY_PATH", "/host/native"),
        ("HOME", "/host/home"),
        ("PATH", "/nonexistent"),
        ("MPK_PLUGIN", "/host/plugin"),
        ("JAVA_TEST_SECRET_CANARY", "must-not-reach-the-jvm"),
    ] {
        std::env::set_var(key, value);
    }
}

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    #[cfg(target_os = "linux")]
    if let [flag, case] = arguments.as_slice() {
        if flag == "__mpk_java_resource_fault_v0" {
            return ExitCode::from(frontend_sandbox::run_java_resource_test_child(case));
        }
        if flag == "--native-resource" {
            return match frontend_sandbox::test_java_resource_boundary(case) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("JAVA_RESOURCE_TEST_{error:?}");
                    ExitCode::FAILURE
                }
            };
        }
    }
    if arguments.first().map(String::as_str) == Some("__mpk_frontend_sandbox_v0") {
        return ExitCode::from(frontend_sandbox::run_bootstrap(&arguments[1..]));
    }
    if arguments.as_slice() == ["__mpk_frontend_probe_v0"] {
        return ExitCode::from(frontend_sandbox::run_probe());
    }
    let result = match arguments.as_slice() {
        [] => {
            check_candidates_and_launcher();
            println!("Java candidate/launcher checks passed; native gate requires explicit Linux invocation");
            Ok(())
        }
        [flag, id]
            if flag == "--native-case"
                && cfg!(all(target_os = "linux", target_arch = "x86_64")) =>
        {
            native_case(id)
        }
        [flag, id]
            if flag == "--native-hostile-case"
                && cfg!(all(target_os = "linux", target_arch = "x86_64")) =>
        {
            poison_environment();
            native_case(id)
        }
        [flag] if flag == "--release-before-source" => match PreparedJavaRun::open() {
            Err(JavaRunError::Release) => Ok(()),
            _ => Err("release did not fail before source access".to_owned()),
        },
        _ => Err("JAVA_RUNNER_TEST_USAGE".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
