use mpk_cli::successor_frontend_runner::{
    run_staged_installed_csharp_frontend, StagedCSharpRunRequest,
};
use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
    CSHARP_FRONTEND_BUNDLE_ID, CSHARP_STAGING_REGISTRY_SHA256, CSHARP_TOOLCHAIN_BUNDLE_ID,
    SUCCESSOR_RELEASE_REGISTRY_HASH_DOMAIN,
};
use mpk_vc::semantic_profile_registry::{
    validate_inactive_semantic_profile_registry, InactiveRegistryRevision,
};
use mpk_vc::{hash_domain_separated_raw, validate_release_registry, CapturedInput, InputKind};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const PROFILE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/specs/vectors/csharp-profile-v0.json"
));
const STAGED_REGISTRY_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/migrations/csharp-02-staging/bundle-registry.json"
));
const STAGED_CANDIDATE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/migrations/csharp-02-staging/csharp-bundle-candidate.json"
));
const STAGED_SEMANTIC_REGISTRY_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../develop/migrations/csharp-02-staging/semantic-profile-registry.json"
));
const ACTIVE_REGISTRY_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../release/bundles/bundle-registry.json"
));
const DOTNET_PROGRAM: &str = "/mpk/toolchain/dotnet/dotnet";
const CSHARP_TOOLCHAIN_SHA256: &str =
    "6cc7711334ffcd9216cfb241e1508491abb62a13b01ebc9b2d883a1ace9627bc";

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("__mpk_frontend_sandbox_v0") {
        return ExitCode::from(mpk_cli::run_frontend_sandbox_bootstrap(&arguments[1..]));
    }
    if arguments.as_slice() == ["__mpk_frontend_probe_v0"] {
        return ExitCode::from(mpk_cli::run_frontend_sandbox_probe());
    }
    let result = if arguments.as_slice() == ["--inside-csharp-runner"] {
        run_inside_installed_fixture()
    } else {
        run_outer_suite()
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run_inside_installed_fixture() -> Result<(), String> {
    let profile: Value = serde_json::from_slice(PROFILE_BYTES).map_err(display)?;
    let semantic_context = csharp_semantic_context(&profile);
    let selection = profile["case_harness"]["baseline_selection"].clone();
    let source = profile["case_harness"]["baseline_files"]["src/Policy.cs"]
        .as_str()
        .ok_or("baseline source is absent")?
        .as_bytes()
        .to_vec();
    let mut contract =
        serde_json::to_vec(&profile["case_harness"]["baseline_files"]["contracts/approved.json"])
            .map_err(display)?;
    contract.push(b'\n');
    let captured = [
        CapturedInput {
            kind: InputKind::Contract,
            normalized_path: "contracts/approved.json",
            bytes: &contract,
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: "src/Policy.cs",
            bytes: &source,
        },
    ];
    let accepted = run_staged_installed_csharp_frontend(StagedCSharpRunRequest {
        semantic_context: &semantic_context,
        selection: &selection,
        captured_inputs: &captured,
    })
    .map_err(display)?;
    let artifacts = accepted
        .envelope()
        .artifacts()
        .ok_or("successful C# launch has no artifacts")?;
    let report = json!({
        "argv": accepted.launcher().argv(),
        "environment": accepted.launcher().environment(),
        "environment_count": accepted.launcher().environment().len(),
        "phase": accepted.envelope().phase(),
        "program": accepted.launcher().program(),
        "registry_sha256": accepted.release_registry().registry_sha256,
        "source_manifest_hash": artifacts.source_manifest().hash().as_str(),
        "source_map_hash": artifacts.source_map().hash().as_str(),
        "status": accepted.envelope().status(),
        "vir_hash": artifacts.vir().hash().as_str(),
        "working_directory": accepted.launcher().working_directory()
    });
    let mut bytes = serde_json::to_vec(&report).map_err(display)?;
    bytes.push(b'\n');
    print!("{}", String::from_utf8(bytes).map_err(display)?);
    Ok(())
}

fn run_outer_suite() -> Result<(), String> {
    validate_staged_models()?;
    validate_launcher_and_isolation_contract()?;
    validate_active_registry_boundary()?;
    run_assembled_fixture()?;
    Ok(())
}

fn validate_staged_models() -> Result<(), String> {
    let semantic = validate_inactive_semantic_profile_registry(
        STAGED_SEMANTIC_REGISTRY_BYTES,
        InactiveRegistryRevision::Revision2,
    )
    .map_err(display)?;
    let registry =
        validate_successor_release_registry(STAGED_REGISTRY_BYTES, &semantic).map_err(display)?;
    let candidate =
        validate_successor_bundle_candidate(STAGED_CANDIDATE_BYTES, &semantic).map_err(display)?;
    ensure(
        registry.registry_sha256() == CSHARP_STAGING_REGISTRY_SHA256,
        "staged release-registry hash differs from the compiled assertion",
    )?;
    let registry_value = serde_json::to_value(registry.registry()).map_err(display)?;
    let candidate_value = serde_json::to_value(candidate.candidate()).map_err(display)?;
    for field in [
        "profile_registry",
        "execution_host_profiles",
        "native_runtime_layout_profiles",
        "frontend_bundles",
        "toolchain_bundles",
        "tuples",
    ] {
        ensure(
            registry_value[field] == candidate_value[field],
            "candidate and registry projections differ",
        )?;
    }
    ensure(
        registry.registry().frontend_bundles.len() == 1
            && registry.registry().toolchain_bundles.len() == 1
            && registry.registry().tuples.len() == 1
            && registry.registry().frontend_bundles[0].bundle_id == CSHARP_FRONTEND_BUNDLE_ID
            && registry.registry().toolchain_bundles[0].bundle_id == CSHARP_TOOLCHAIN_BUNDLE_ID,
        "staged C# projection is not closed",
    )?;

    let mut legacy_payload = registry_value.clone();
    legacy_payload
        .as_object_mut()
        .ok_or("registry is not an object")?
        .remove("registry_sha256");
    let legacy_canonical = serde_json::to_vec(&legacy_payload).map_err(display)?;
    let legacy = hash_domain_separated_raw(
        mpk_vc::HashDomain::new("MPK-BUNDLE-REGISTRY-0.1"),
        &legacy_canonical,
    )
    .map_err(display)?
    .to_hex();
    let successor =
        hash_domain_separated_raw(SUCCESSOR_RELEASE_REGISTRY_HASH_DOMAIN, &legacy_canonical)
            .map_err(display)?
            .to_hex();
    ensure(
        successor == CSHARP_STAGING_REGISTRY_SHA256 && legacy != successor,
        "successor release hash domain is not migration-distinct",
    )?;

    let mut wrong_schema: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_schema["schema"] = json!("mpk.release.bundle_candidate.v0");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_schema)?, &semantic).is_err(),
        "legacy candidate schema was accepted",
    )?;
    let mut unknown_field: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    unknown_field["candidate_path"] = json!("/tmp/candidate");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&unknown_field)?, &semantic).is_err(),
        "unknown candidate field was accepted",
    )?;
    let mut wrong_contract: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_contract["frontend_bundles"][0]["profile_contracts"][0]["contract_id"] =
        json!("mpk.profile.frontend.rust_checked.v0");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_contract)?, &semantic).is_err(),
        "cross-profile frontend contract was accepted",
    )?;
    let mut wrong_frontend_limit: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_frontend_limit["frontend_bundles"][0]["profile_contracts"][0]["value"]
        ["limit_profile_id"] = json!("mpk.vir.limits.v0");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_frontend_limit)?, &semantic)
            .is_err(),
        "shared VIR limit was accepted as the private frontend limit",
    )?;
    let mut wrong_tuple_limit: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_tuple_limit["tuples"][0]["limit_profile_id"] = json!("mpk.csharp.limits.v0");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_tuple_limit)?, &semantic)
            .is_err(),
        "private frontend limit was accepted as the release tuple limit",
    )?;
    let mut wrong_inventory: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_inventory["frontend_bundles"][0]["inventory"]["files"][0]["sha256"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_inventory)?, &semantic).is_err(),
        "mutated bundle inventory was accepted",
    )?;
    let mut wrong_runtime_link: Value =
        serde_json::from_slice(STAGED_CANDIDATE_BYTES).map_err(display)?;
    wrong_runtime_link["toolchain_bundles"][0]["components"][0]["runtime"]["libraries"][0]
        ["component_path"] = json!("lib/x86_64-linux-gnu/libc.so.6");
    ensure(
        validate_successor_bundle_candidate(&canonical_line(&wrong_runtime_link)?, &semantic)
            .is_err(),
        "runtime library was accepted with a mismatched inventory identity",
    )?;
    Ok(())
}

fn validate_launcher_and_isolation_contract() -> Result<(), String> {
    let profile: Value = serde_json::from_slice(PROFILE_BYTES).map_err(display)?;
    let launcher = &profile["launcher_contract"];
    ensure(
        launcher["program"] == DOTNET_PROGRAM
            && launcher["working_directory"] == "/mpk/source"
            && launcher["stdin"] == "null"
            && launcher["stdout"] == "bounded_frontend_protocol"
            && launcher["stderr"] == "bounded_diagnostic_only"
            && launcher["inherited_environment"] == json!([]),
        "frozen launcher scalar contract differs",
    )?;
    ensure(
        launcher["runtime_config"]
            == json!({
                "framework_name": "Microsoft.NETCore.App",
                "framework_version": "10.0.11",
                "roll_forward": "Disable",
                "tfm": "net10.0"
            }),
        "frozen runtime configuration differs",
    )?;
    let prefix = launcher["argv_prefix"]
        .as_array()
        .ok_or("launcher prefix is not an array")?
        .iter()
        .map(|value| value.as_str().ok_or("launcher prefix is not text"))
        .collect::<Result<Vec<_>, _>>()?;
    ensure(
        prefix
            == [
                DOTNET_PROGRAM,
                "exec",
                "--depsfile",
                "/mpk/frontend/csharp2vir.deps.json",
                "--runtimeconfig",
                "/mpk/frontend/csharp2vir.runtimeconfig.json",
                "--fx-version",
                "10.0.11",
                "/mpk/frontend/csharp2vir.dll",
            ],
        "frozen launcher argv prefix differs",
    )?;
    let environment = launcher["environment"]
        .as_object()
        .ok_or("launcher environment is not an object")?;
    ensure(
        environment
            == json!({
                "COMPlus_ReadyToRun": "0",
                "DOTNET_CLI_TELEMETRY_OPTOUT": "1",
                "DOTNET_MULTILEVEL_LOOKUP": "0",
                "DOTNET_NOLOGO": "1",
                "DOTNET_ROOT": "/mpk/toolchain/dotnet",
                "DOTNET_SKIP_FIRST_TIME_EXPERIENCE": "1",
                "DOTNET_SYSTEM_GLOBALIZATION_INVARIANT": "1",
                "DOTNET_TieredCompilation": "0",
                "DOTNET_TieredPGO": "0",
                "HOME": "/mpk/empty-home",
                "LANG": "C.UTF-8",
                "LC_ALL": "C.UTF-8",
                "NUGET_HTTP_CACHE_PATH": "/mpk/empty-nuget-http",
                "NUGET_PACKAGES": "/mpk/empty-nuget",
                "NUGET_PLUGINS_CACHE_PATH": "/mpk/empty-nuget-plugins",
                "PATH": "/nonexistent",
                "TMPDIR": "/mpk/tmp",
                "TZ": "UTC"
            })
            .as_object()
            .ok_or("expected launcher environment is not an object")?,
        "frozen closed environment differs",
    )?;
    let isolation = profile["isolation_cases"]
        .as_array()
        .ok_or("isolation cases are not an array")?;
    let observed = isolation
        .iter()
        .map(|case| case["id"].as_str().ok_or("isolation ID is not text"))
        .collect::<Result<Vec<_>, _>>()?;
    ensure(
        observed
            == [
                "isolation.no_network",
                "isolation.no_restore",
                "isolation.no_msbuild",
                "isolation.no_analyzers",
                "isolation.no_generators",
                "isolation.no_compiler_server",
                "isolation.no_ambient_references",
                "isolation.no_roll_forward",
                "isolation.no_environment_inheritance",
                "isolation.no_dynamic_native_search",
                "isolation.no_candidate_execution",
                "isolation.no_plugins",
            ],
        "launcher/isolation vector set is not exact",
    )?;
    ensure(
        isolation
            == json!([
                {
                    "expect": "reject_or_unavailable",
                    "id": "isolation.no_network",
                    "mutation": "enable or require network"
                },
                {
                    "expect": "reject_or_unavailable",
                    "id": "isolation.no_restore",
                    "mutation": "invoke dotnet restore or NuGet client"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_msbuild",
                    "mutation": "open project, solution, or MSBuild workspace"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_analyzers",
                    "mutation": "load analyzer assembly or execute analyzer"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_generators",
                    "mutation": "load or run source generator"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_compiler_server",
                    "mutation": "use compiler server or subordinate csc"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_ambient_references",
                    "mutation": "use TPA, GAC, runtime implementation, or host reference"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_roll_forward",
                    "mutation": "runtime 10.0.11 absent but another runtime exists"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_environment_inheritance",
                    "mutation": "add undeclared environment variable"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_dynamic_native_search",
                    "mutation": "probe native library outside the .NET archive root and selected native runtime layout"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_candidate_execution",
                    "mutation": "emit or execute candidate source"
                },
                {
                    "expect": "reject",
                    "id": "isolation.no_plugins",
                    "mutation": "payload names path, URI, callback, checker, code, or plugin"
                }
            ])
            .as_array()
            .ok_or("expected isolation cases are not an array")?,
        "launcher/isolation vector payload differs",
    )?;
    let sandbox_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/frontend_sandbox.rs"
    ));
    for required in [
        "UnshareFlags::NEWNET",
        "MountFlags::RDONLY",
        "MountFlags::NOEXEC",
        "set_no_new_privs(true)",
        "memory.swap.max",
        "pids.max",
        "cgroup.kill",
        "env_clear()",
        "CSharpLauncherPlan",
    ] {
        let source = if required == "CSharpLauncherPlan" {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/successor_frontend_runner.rs"
            ))
        } else {
            sandbox_source
        };
        ensure(source.contains(required), "isolation mechanism is absent")?;
    }
    Ok(())
}

fn validate_active_registry_boundary() -> Result<(), String> {
    let active = validate_release_registry(ACTIVE_REGISTRY_BYTES).map_err(display)?;
    let languages = active
        .registry()
        .tuples
        .iter()
        .map(|tuple| tuple.source_language.as_str())
        .collect::<BTreeSet<_>>();
    ensure(
        active.registry().schema == "mpk.release.bundle_registry.v0"
            && languages == BTreeSet::from(["go", "rust"])
            && active
                .registry()
                .frontend_bundles
                .iter()
                .all(|bundle| bundle.source_language != "csharp")
            && active
                .registry()
                .toolchain_bundles
                .iter()
                .all(|bundle| bundle.source_language != "csharp"),
        "active release registry is no longer Go/Rust-only",
    )?;
    ensure(
        validate_release_registry(STAGED_REGISTRY_BYTES).is_err(),
        "active validator accepted the staged successor registry",
    )?;
    let semantic = validate_inactive_semantic_profile_registry(
        STAGED_SEMANTIC_REGISTRY_BYTES,
        InactiveRegistryRevision::Revision2,
    )
    .map_err(display)?;
    ensure(
        validate_successor_release_registry(ACTIVE_REGISTRY_BYTES, &semantic).is_err(),
        "successor validator accepted the active registry",
    )?;
    Ok(())
}

fn run_assembled_fixture() -> Result<(), String> {
    let root = repository_root();
    let temporary = tempfile::Builder::new()
        .prefix("mpk-csharp-runner-")
        .tempdir_in("/tmp")
        .map_err(display)?;
    let installed = temporary.path().join("installed");
    let current_executable = env::current_exe().map_err(display)?;
    let materialize = Command::new("python3")
        .args([
            root.join("scripts/csharp_release_bundles.py")
                .to_str()
                .ok_or("assembler path is not UTF-8")?,
            "materialize-fixture",
            current_executable
                .to_str()
                .ok_or("test executable path is not UTF-8")?,
            installed
                .to_str()
                .ok_or("installed fixture path is not UTF-8")?,
        ])
        .current_dir(&root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .map_err(display)?;
    ensure(
        materialize.status.success()
            && materialize.stdout.is_empty()
            && materialize.stderr.is_empty(),
        "staged installed fixture could not be materialized",
    )?;
    let executable = installed.join("bin/mpk");
    let run_once = |hostile_ambient: bool| {
        let mut command = Command::new("python3");
        command
            .args([
                root.join("scripts/csharp_release_bundles.py")
                    .to_str()
                    .ok_or("assembler path is not UTF-8")?,
                "run-installed",
                executable
                    .to_str()
                    .ok_or("installed executable path is not UTF-8")?,
            ])
            .current_dir(&root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("PYTHONDONTWRITEBYTECODE", "1");
        if hostile_ambient {
            command
                .env("DOTNET_ROOT", "/host/dotnet")
                .env("DOTNET_ROLL_FORWARD", "LatestMajor")
                .env("HOME", "/host/home")
                .env("LD_LIBRARY_PATH", "/host/lib")
                .env("MPK_PLUGIN", "/host/plugin")
                .env("NUGET_PACKAGES", "/host/nuget");
        }
        command.output().map_err(display)
    };
    let execution = run_once(false)?;
    if !execution.status.success() {
        return Err(format!(
            "installed C# runner failed: stdout={} stderr={}",
            String::from_utf8_lossy(&execution.stdout),
            String::from_utf8_lossy(&execution.stderr)
        ));
    }
    ensure(
        execution.stderr.is_empty(),
        "installed C# runner wrote stderr",
    )?;
    let replay = run_once(true)?;
    ensure(
        replay.status.success() && replay.stderr.is_empty() && replay.stdout == execution.stdout,
        "two staged C# launches were not byte-identical",
    )?;
    let report: Value = serde_json::from_slice(&execution.stdout).map_err(display)?;
    let profile: Value = serde_json::from_slice(PROFILE_BYTES).map_err(display)?;
    ensure(
        canonical_line(&report)? == execution.stdout
            && report["argv"] == expected_baseline_argv()
            && report["environment"] == profile["launcher_contract"]["environment"]
            && report["status"] == "ir-lowered"
            && report["phase"] == "emission"
            && report["program"] == DOTNET_PROGRAM
            && report["working_directory"] == "/mpk/source"
            && report["environment_count"] == 18
            && report["registry_sha256"] == CSHARP_STAGING_REGISTRY_SHA256,
        "installed C# runner report differs",
    )?;
    for field in ["vir_hash", "source_map_hash", "source_manifest_hash"] {
        let value = report[field].as_str().ok_or("artifact hash is absent")?;
        ensure(lower_sha256(value), "artifact hash is malformed")?;
    }
    let tampered = installed
        .join("libexec/mpk/bundles")
        .join(CSHARP_FRONTEND_BUNDLE_ID)
        .join("csharp2vir.runtimeconfig.json");
    fs::set_permissions(&tampered, fs::Permissions::from_mode(0o644)).map_err(display)?;
    let mut bytes = fs::read(&tampered).map_err(display)?;
    bytes.push(b' ');
    fs::write(&tampered, bytes).map_err(display)?;
    fs::set_permissions(&tampered, fs::Permissions::from_mode(0o444)).map_err(display)?;
    let rejected = run_once(false)?;
    ensure(
        !rejected.status.success()
            && rejected.stdout.is_empty()
            && rejected.stderr == b"BUNDLE_REPRODUCIBILITY_MISMATCH\n",
        "tampered installed C# bytes did not fail closed",
    )?;
    Ok(())
}

fn expected_baseline_argv() -> Value {
    json!([
        DOTNET_PROGRAM,
        "exec",
        "--depsfile",
        "/mpk/frontend/csharp2vir.deps.json",
        "--runtimeconfig",
        "/mpk/frontend/csharp2vir.runtimeconfig.json",
        "--fx-version",
        "10.0.11",
        "/mpk/frontend/csharp2vir.dll",
        "lower",
        "/mpk/source",
        "--semantic-profile",
        "mpk.csharp.scalar.v0",
        "--target",
        "linux-x64",
        "--compilation",
        "payment-policy",
        "--source",
        "src/Policy.cs",
        "--contract",
        "contracts/approved.json",
        "--method",
        "Example.Payment.Policy::Approved(i64,i64)->bool",
        "--profile-registry-id",
        "mpk.semantic_profile.registry.v1",
        "--profile-registry-revision",
        "2",
        "--profile-registry-sha256",
        "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75",
        "--profile-entry-sha256",
        "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac",
        "--frontend-bundle-id",
        CSHARP_FRONTEND_BUNDLE_ID,
        "--frontend-sha256",
        "0783dc269c152ad1b13e77f42f9eff6f6891002c65890bc1445f2fe1a1a0410d",
        "--release-registry-id",
        "mpk.release.registry.v1",
        "--release-registry-sha256",
        CSHARP_STAGING_REGISTRY_SHA256,
        "--toolchain-bundle-id",
        CSHARP_TOOLCHAIN_BUNDLE_ID,
        "--toolchain-root",
        "/mpk/toolchain",
        "--toolchain-distribution-sha256",
        CSHARP_TOOLCHAIN_SHA256
    ])
}

fn csharp_semantic_context(profile: &Value) -> Value {
    json!({
        "profile_registry": {
            "schema": "mpk.semantic_profile.registry.v1",
            "id": "mpk.semantic_profile.registry.v1",
            "revision": 2,
            "registry_sha256": "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75"
        },
        "profile_entry_sha256": "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac",
        "source_language": "csharp",
        "semantic_profile": "mpk.csharp.scalar.v0",
        "semantic_parameters": profile["semantic_parameters"].clone()
    })
}

fn canonical_line(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(display)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}
