use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, SuccessorManifestUnitKind, SuccessorSourceManifestStage,
    SuccessorSourceManifestValidationContext, SuccessorSourceMapValidationContext,
    SUCCESSOR_RELEASE_REGISTRY_ID, SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
};
use mpk_vc::{
    CapturedInput, InputKind, ReleaseRegistryIdentity, SourceOrigin, VirFeature, VirInstruction,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_PATH: &str = "src/Case.cs";
const ROOT_CONTRACT_PATH: &str = "contracts/f.json";
const CALLEE_CONTRACT_PATH: &str = "contracts/g.json";
const ROOT_METHOD: &str = "Vector.Calls::F(i32)->i32";
const CALLEE_METHOD: &str = "Vector.Calls::G(i32)->i32";
const SOURCE: &str = concat!(
    "namespace Vector;\n",
    "public static class Calls\n",
    "{\n",
    "    public static int F(int x)\n",
    "    {\n",
    "        int y = G(x);\n",
    "        y = unchecked(y + 1);\n",
    "        if (y > 0)\n",
    "        {\n",
    "            y = unchecked(y - 1);\n",
    "        }\n",
    "\n",
    "        return y;\n",
    "    }\n",
    "\n",
    "    private static int G(int x) { return checked(x * 2); }\n",
    "}\n",
);
const ROOT_CONTRACT: &str = "{\"abrupt_completion\":\"forbidden\",\"ensures\":[{\"bool\":true}],\"method\":\"Vector.Calls::F(i32)->i32\",\"modifies\":[],\"requires\":[],\"schema\":\"mpk.csharp.contract.v0\",\"semantic_profile\":\"mpk.csharp.scalar.v0\",\"termination\":\"total\"}\n";
const CALLEE_CONTRACT: &str = "{\"abrupt_completion\":\"forbidden\",\"ensures\":[{\"bool\":true}],\"method\":\"Vector.Calls::G(i32)->i32\",\"modifies\":[],\"requires\":[],\"schema\":\"mpk.csharp.contract.v0\",\"semantic_profile\":\"mpk.csharp.scalar.v0\",\"termination\":\"total\"}\n";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(relative: &str) -> Value {
    let bytes = fs::read(repository_root().join(relative)).expect("read JSON");
    serde_json::from_slice(&bytes).expect("parse JSON")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn frozen_call_map_source_offsets_and_owned_profiles_are_exact() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    assert_eq!(
        profile["operation_mappings"]
            .as_array()
            .expect("operation mappings")
            .last()
            .expect("direct-call mapping"),
        &json!({
            "source": "direct_static_call",
            "operand_types": ["exact_signature"],
            "context": "any",
            "vir": ["CallStatic"],
            "checks": ["callee_contract_hash"]
        })
    );
    let row = profile["semantic_rows"]
        .as_array()
        .expect("semantic rows")
        .iter()
        .find(|row| row["row"] == "M27")
        .expect("M27");
    assert_eq!(
        row,
        &json!({
            "row": "M27",
            "disposition": "accept_under_profile_restrictions",
            "basis": "P04"
        })
    );
    assert_eq!(
        profile["source_map_cases"],
        json!([
            {
                "id": "map.ascii",
                "source": "a\nx\n",
                "utf16_start": 2,
                "utf16_end": 3,
                "expect": {
                    "outcome": "accept",
                    "utf8_start": 2,
                    "utf8_end": 3,
                    "line_start": 1,
                    "column_start_utf16": 0,
                    "line_end": 1,
                    "column_end_utf16": 1
                }
            },
            {
                "id": "map.bmp",
                "source": "// é\n",
                "utf16_start": 3,
                "utf16_end": 4,
                "expect": {
                    "outcome": "accept",
                    "utf8_start": 3,
                    "utf8_end": 5,
                    "line_start": 0,
                    "column_start_utf16": 3,
                    "line_end": 0,
                    "column_end_utf16": 4
                }
            },
            {
                "id": "map.surrogate_pair",
                "source": "// 😀\n",
                "utf16_start": 3,
                "utf16_end": 5,
                "expect": {
                    "outcome": "accept",
                    "utf8_start": 3,
                    "utf8_end": 7,
                    "line_start": 0,
                    "column_start_utf16": 3,
                    "line_end": 0,
                    "column_end_utf16": 5
                }
            },
            {
                "id": "map.reject_surrogate_split",
                "source": "// 😀\n",
                "utf16_start": 4,
                "utf16_end": 5,
                "expect": {"outcome": "reject", "code": "CSHARP_SOURCE_MAP_UTF16"}
            },
            {
                "id": "map.reject_zero_length",
                "source": "x\n",
                "utf16_start": 0,
                "utf16_end": 0,
                "expect": {"outcome": "reject", "code": "CSHARP_SOURCE_MAP_RANGE"}
            },
            {
                "id": "map.reject_out_of_range",
                "source": "x\n",
                "utf16_start": 0,
                "utf16_end": 3,
                "expect": {"outcome": "reject", "code": "CSHARP_SOURCE_MAP_RANGE"}
            }
        ])
    );

    let contracts = profile["profile_contracts"]
        .as_array()
        .expect("profile contracts")
        .iter()
        .filter(|contract| {
            matches!(
                contract["field"].as_str(),
                Some("manifest" | "source_map" | "vir")
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        Value::Array(contracts),
        json!([
            {
                "field": "manifest",
                "envelope": {
                    "profile_entry_sha256": "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac",
                    "contract_id": "mpk.profile.manifest.csharp_scalar.v0",
                    "value": {
                        "input_kinds": ["contract", "source"],
                        "source_extension": ".cs",
                        "unit_kind": "compilation"
                    }
                }
            },
            {
                "field": "source_map",
                "envelope": {
                    "profile_entry_sha256": "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac",
                    "contract_id": "mpk.profile.source_map.csharp_scalar.v0",
                    "value": {
                        "encoding": "utf-8",
                        "offset_unit": "utf8-byte",
                        "synthetic_reasons": []
                    }
                }
            },
            {
                "field": "vir",
                "envelope": {
                    "profile_entry_sha256": "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac",
                    "contract_id": "mpk.profile.vir.csharp_scalar.v0",
                    "value": {
                        "operation_profile_id": "mpk.csharp.vir_operations.v0",
                        "source_map_profile_id": "mpk.csharp.source_map.v0",
                        "vir_limit_profile_id": "mpk.vir.limits.v0"
                    }
                }
            }
        ])
    );
}

#[test]
fn emission_owners_are_complete_and_the_csharp_route_remains_inactive() {
    let root = repository_root();
    let project = fs::read_to_string(root.join("csharp-tools/csharp2vir/csharp2vir.csproj"))
        .expect("read C# project");
    for input in [
        "EmissionCanonical.cs",
        "EmissionModel.cs",
        "EmissionProfiles.cs",
        "FrontendSuccessEmitter.cs",
        "SourceManifestEmitter.cs",
        "SourceMapEmitter.cs",
        "VirEmitter.cs",
    ] {
        assert!(project.contains(input), "missing project input {input}");
    }
    let program = fs::read_to_string(root.join("csharp-tools/csharp2vir/Program.cs"))
        .expect("read inactive frontend");
    let lowering = program
        .find("CSharpLowering.Lower")
        .expect("lowering stage");
    let emission = program
        .find("CSharpFrontendSuccessEmitter.Emit")
        .expect("emission stage");
    let output = program
        .find("Console.OpenStandardOutput")
        .expect("success transport");
    assert!(lowering < emission && emission < output);
    assert!(program.contains("CSharpFrontendFailureEmitter.Emit"));

    let harness = fs::read_to_string(root.join("crates/mpk-cli/tests/csharp_emission_harness.cs"))
        .expect("read emission harness");
    for owner in [
        "StaticCallsAreCalleeFirstAndLeftToRight",
        "StableIdsIgnoreRoslynOrdinalsAndCaptures",
        "SourceMapVectorsAreExact",
        "ArtifactsAreCanonicalCompleteAndDeterministic",
        "OwnedCompiledProfilesAreExact",
        "SourceMapFailuresAreClosed",
    ] {
        assert!(harness.contains(owner), "missing harness owner {owner}");
    }
    let script = fs::read_to_string(root.join("scripts/csharp_build_inputs.py"))
        .expect("read C# build gate");
    for owner in [
        "validate_emission_implementation",
        "run_emission_tests=True",
        "csharp2vir-emission-tests.dll",
        "argv == [\"test-emission\"]",
        "argv == [\"emit-test-envelope\"]",
    ] {
        assert!(script.contains(owner), "missing build owner {owner}");
    }
    let active = fs::read_to_string(root.join("release/bundles/bundle-registry.json"))
        .expect("read active registry");
    assert!(!active.contains("csharp2vir"));
    assert!(!active.contains("mpk.csharp.scalar.v0"));

    let manifest = load("develop/specs/vectors/manifest.json");
    let record = manifest["vectors"]
        .as_array()
        .expect("vector records")
        .iter()
        .find(|record| record["path"] == "develop/specs/vectors/csharp-profile-v0.json")
        .expect("C# vector record");
    assert!(record["implementation_test_owners"]
        .as_array()
        .expect("implementation owners")
        .iter()
        .any(|owner| owner == "crates/mpk-cli/tests/csharp_emission.rs"));
}

#[test]
fn provisioned_emission_is_accepted_by_the_shared_successor_validators() {
    let root = repository_root();
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let hash = profile["toolchain_inputs"]["toolchain_inputs_sha256"]
        .as_str()
        .expect("toolchain hash");
    let cache = root
        .join("release/build-input-cache/csharp")
        .join(hash)
        .join("archives");
    let archives = profile["toolchain_inputs"]["archives"]
        .as_array()
        .expect("toolchain archives");
    let present = archives
        .iter()
        .filter(|record| {
            let suffix = match record["kind"].as_str().expect("archive kind") {
                "tar.gz" => ".tar.gz",
                "nupkg" => ".nupkg",
                kind => panic!("unexpected archive kind {kind}"),
            };
            cache
                .join(format!(
                    "{}{}",
                    record["id"].as_str().expect("archive id"),
                    suffix
                ))
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
        .arg("--emit-test-envelope")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("execute pinned C# emission harness");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));

    let registry_vectors = load("develop/specs/vectors/semantic-profile-registry-v2.json");
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&registry_vectors["registry"])
            .expect("canonical revision-2 registry transport"),
        RegistryRevision::Revision2,
    )
    .expect("frozen revision-2 registry validates");
    let context_value = json!({
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
    });
    let semantic_context =
        validate_registry_semantic_context(&registry, &context_value).expect("C# semantic context");
    let selection_value = json!({
        "schema": "mpk.selection.csharp_methods.v0",
        "value": {
            "compilation": "call-case",
            "contracts": [ROOT_CONTRACT_PATH, CALLEE_CONTRACT_PATH],
            "methods": [ROOT_METHOD],
            "sources": [SOURCE_PATH]
        }
    });
    let selection =
        validate_registry_selection_envelope(&registry, &semantic_context, &selection_value)
            .expect("C# selection");
    let release_registry: ReleaseRegistryIdentity = serde_json::from_value(json!({
        "schema": SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
        "id": SUCCESSOR_RELEASE_REGISTRY_ID,
        "registry_sha256": "1111111111111111111111111111111111111111111111111111111111111111"
    }))
    .expect("release-registry identity");
    let captured = [
        CapturedInput {
            kind: InputKind::Contract,
            normalized_path: ROOT_CONTRACT_PATH,
            bytes: ROOT_CONTRACT.as_bytes(),
        },
        CapturedInput {
            kind: InputKind::Contract,
            normalized_path: CALLEE_CONTRACT_PATH,
            bytes: CALLEE_CONTRACT.as_bytes(),
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: SOURCE_PATH,
            bytes: SOURCE.as_bytes(),
        },
    ];
    let envelope: Value = serde_json::from_slice(&output.stdout[..output.stdout.len() - 1])
        .expect("C# success envelope JSON");
    assert_eq!(
        envelope["source_manifest"]["inputs"],
        json!([
            {
                "kind": "contract",
                "normalized_path": ROOT_CONTRACT_PATH,
                "sha256": sha256(ROOT_CONTRACT.as_bytes()),
                "size_bytes": ROOT_CONTRACT.len()
            },
            {
                "kind": "contract",
                "normalized_path": CALLEE_CONTRACT_PATH,
                "sha256": sha256(CALLEE_CONTRACT.as_bytes()),
                "size_bytes": CALLEE_CONTRACT.len()
            },
            {
                "kind": "source",
                "normalized_path": SOURCE_PATH,
                "sha256": sha256(SOURCE.as_bytes()),
                "size_bytes": SOURCE.len()
            }
        ])
    );
    let map_entries = envelope["source_map"]["entries"]
        .as_array()
        .expect("source-map entries");
    assert!(map_entries.len() > 5);
    let function_origin = map_entries
        .iter()
        .find(|entry| {
            entry["reference"]["kind"] == "function"
                && entry["reference"]["function_id"] == ROOT_METHOD
        })
        .expect("root function origin");
    let function_start = SOURCE
        .find("public static int F(int x)")
        .expect("root declaration start");
    let function_end = SOURCE
        .find("\n    private static int G")
        .expect("root declaration end marker")
        - 1;
    assert_eq!(function_origin["origin"]["start"], function_start);
    assert_eq!(function_origin["origin"]["end"], function_end);
    let call_origin = map_entries
        .iter()
        .find(|entry| {
            entry["reference"]["kind"] == "instruction"
                && entry["reference"]["function_id"] == ROOT_METHOD
                && entry["reference"]["instruction"] == "t0"
        })
        .expect("root call origin");
    let call_start = SOURCE.find("G(x)").expect("call start");
    assert_eq!(call_origin["origin"]["start"], call_start);
    assert_eq!(call_origin["origin"]["end"], call_start + "G(x)".len());
    let vir_bytes = serde_json::to_vec(&envelope["ir"]["value"]).expect("serialize VIR");
    let vir = import_successor_vir_json(&vir_bytes, &registry).expect("shared VIR validator");
    let source_map_bytes =
        serde_json::to_vec(&envelope["source_map"]).expect("serialize source map");
    let source_map = import_successor_source_map_json(
        &source_map_bytes,
        SuccessorSourceMapValidationContext {
            registry: &registry,
            vir: &vir,
            captured_inputs: &captured,
            synthetic_permissions: &[],
        },
    )
    .expect("shared source-map validator");
    let source_manifest_bytes =
        serde_json::to_vec(&envelope["source_manifest"]).expect("serialize source manifest");
    import_successor_source_manifest_json(
        &source_manifest_bytes,
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: &registry,
            vir: &vir,
            source_map: &source_map,
            captured_inputs: &captured,
            expected_release_registry: &release_registry,
        },
    )
    .expect("shared source-manifest validator");
    let request = SuccessorFrontendProtocolRequest {
        registry: &registry,
        semantic_context: &semantic_context,
        selection: &selection,
        release_registry: &release_registry,
        captured_inputs: &captured,
        synthetic_permissions: &[],
    };
    let accepted = validate_successor_frontend_process(
        request,
        FrontendProcessFacts {
            exit_code: Some(0),
            signaled: false,
            stdout: &output.stdout,
            stderr_observed_bytes: 0,
        },
    )
    .expect("C# artifacts pass the shared successor validators without an adapter");
    assert_eq!(accepted.status(), "ir-lowered");
    assert_eq!(accepted.phase(), "emission");
    assert_eq!(accepted.selection(), &selection);
    assert_eq!(accepted.semantic_context(), &semantic_context);

    let artifacts = accepted.artifacts().expect("success artifacts");
    let units = artifacts.vir().module().units();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].id(), "call-case");
    assert_eq!(
        units[0]
            .functions()
            .iter()
            .map(|function| function.id())
            .collect::<Vec<_>>(),
        [CALLEE_METHOD, ROOT_METHOD]
    );
    let callee = &units[0].functions()[0];
    let caller = &units[0].functions()[1];
    let call = caller
        .blocks()
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|instruction| matches!(instruction, VirInstruction::CallStatic { .. }))
        .expect("CallStatic instruction");
    match call {
        VirInstruction::CallStatic {
            function,
            contract_hash,
            args,
            safety_checks,
            ..
        } => {
            assert_eq!(function, CALLEE_METHOD);
            assert_eq!(contract_hash, callee.contracts().contract_hash());
            assert_eq!(args.len(), 1);
            assert!(safety_checks.is_empty());
        }
        instruction => panic!("expected CallStatic, found {instruction:?}"),
    }
    assert_eq!(
        caller.features_used(),
        [
            VirFeature::Branch,
            VirFeature::CallStatic,
            VirFeature::MutableLocal,
        ]
    );
    assert!(callee
        .blocks()
        .iter()
        .flat_map(|block| &block.instructions)
        .any(|instruction| matches!(
            instruction,
            VirInstruction::BinOp { safety_checks, .. } if safety_checks.len() == 1
        )));
    assert_eq!(
        artifacts.source_map().map().entries().len(),
        map_entries.len()
    );
    assert!(artifacts
        .source_map()
        .map()
        .entries()
        .iter()
        .all(|entry| matches!(&entry.origin, SourceOrigin::Source { .. })));
    let manifest = artifacts.source_manifest();
    assert_eq!(manifest.stage(), SuccessorSourceManifestStage::Frontend);
    assert!(manifest.manifest().vc_hash().is_none());
    assert_eq!(manifest.manifest().inputs().len(), captured.len());
    assert_eq!(manifest.manifest().units().len(), 1);
    assert_eq!(
        manifest.manifest().units()[0].kind(),
        SuccessorManifestUnitKind::Compilation
    );
}
