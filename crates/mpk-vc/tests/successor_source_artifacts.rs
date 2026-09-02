use mpk_vc::semantic_profile_registry::{
    validate_semantic_profile_registry, RegistryRevision, ValidatedSemanticProfileRegistry,
};
use mpk_vc::source_manifest::{ReleaseRegistryIdentity, SourceManifest};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, successor_contract_hash_value, successor_source_manifest_hash_value,
    successor_source_map_hash_value, successor_vir_hash_value, SuccessorArtifactErrorCode,
    SuccessorSourceManifestStage, SuccessorSourceManifestValidationContext,
    SuccessorSourceMapValidationContext, SUCCESSOR_CONTRACT_HASH_DOMAIN,
    SUCCESSOR_SOURCE_MANIFEST_HASH_DOMAIN, SUCCESSOR_SOURCE_MAP_HASH_DOMAIN,
    SUCCESSOR_VIR_HASH_DOMAIN, SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::{
    canonical_json_bytes, import_vir_json, parse_strict_json, CapturedInput, InputKind,
    StrictJsonLimits,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const REGISTRY_VECTORS: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const CSHARP_VECTORS: &[u8] =
    include_bytes!("../../../develop/specs/vectors/csharp-profile-v0.json");
const ACTIVE_VIR: &[u8] = include_bytes!("../../../fixtures/vir-go/frontend/basic-arith/vir.json");
const ACTIVE_RUST_CALL_VIR: &[u8] =
    include_bytes!("../../../fixtures/rust-basic/positive/module-calls/artifacts/vir.json");
const ACTIVE_SOURCE_MAP: &[u8] =
    include_bytes!("../../../fixtures/vir-go/frontend/basic-arith/source-map.json");
const ACTIVE_SOURCE_MANIFEST: &[u8] =
    include_bytes!("../../../fixtures/vir-go/frontend/basic-arith/source-manifest.frontend.json");
const GO_MOD: &[u8] = include_bytes!("../../../fixtures/go-basic/go.mod");
const GO_SOURCE: &[u8] = include_bytes!("../../../fixtures/go-basic/positive/arith/arith.go");
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

struct StagedArtifacts {
    registry: ValidatedSemanticProfileRegistry,
    selection: Value,
    release_registry: ReleaseRegistryIdentity,
    vir: Value,
    source_map: Value,
    source_manifest: Value,
}

fn load(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("checked-in JSON fixture")
}

fn canonical(value: &Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).expect("JSON value serializes");
    let strict = parse_strict_json(
        &encoded,
        StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576),
    )
    .expect("test value is strict JSON");
    canonical_json_bytes(&strict).expect("test value canonicalizes")
}

fn captured_inputs() -> [CapturedInput<'static>; 3] {
    [
        CapturedInput {
            kind: InputKind::BuildManifest,
            normalized_path: "go.mod",
            bytes: GO_MOD,
        },
        CapturedInput {
            kind: InputKind::Lockfile,
            normalized_path: "go.sum",
            bytes: b"",
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: "positive/arith/arith.go",
            bytes: GO_SOURCE,
        },
    ]
}

fn successor_vir_fixture(active: &[u8], context: &Value) -> Value {
    let mut vir = load(active);
    vir["schema"] = Value::String(SUCCESSOR_VIR_SCHEMA.into());
    vir.as_object_mut()
        .expect("VIR root")
        .remove("source_language");
    vir.as_object_mut()
        .expect("VIR root")
        .remove("semantic_profile");
    vir.as_object_mut()
        .expect("VIR root")
        .remove("semantic_parameters");
    vir["semantic_context"] = context.clone();

    let mut contract_hashes = BTreeMap::new();
    for unit in vir["units"].as_array_mut().expect("VIR units") {
        for function in unit["functions"].as_array_mut().expect("VIR functions") {
            let function_id = function["id"].as_str().expect("function id").to_owned();
            let contract = &mut function["contracts"];
            contract
                .as_object_mut()
                .expect("VIR contract")
                .remove("semantic_profile");
            contract
                .as_object_mut()
                .expect("VIR contract")
                .remove("semantic_parameters");
            contract["semantic_context"] = context.clone();
            contract["contract_hash"] = Value::String(ZERO_SHA256.into());
            let hash = successor_contract_hash_value(contract)
                .expect("successor contract hash")
                .as_str()
                .to_owned();
            contract["contract_hash"] = Value::String(hash.clone());
            contract_hashes.insert(function_id, hash);
        }
    }
    for unit in vir["units"].as_array_mut().expect("VIR units") {
        for function in unit["functions"].as_array_mut().expect("VIR functions") {
            for block in function["blocks"].as_array_mut().expect("VIR blocks") {
                for instruction in block["instructions"]
                    .as_array_mut()
                    .expect("VIR instructions")
                {
                    if instruction["kind"] == "CallStatic" {
                        let callee = instruction["function"].as_str().expect("static callee");
                        instruction["contract_hash"] = Value::String(
                            contract_hashes
                                .get(callee)
                                .expect("static callee contract")
                                .clone(),
                        );
                    }
                }
            }
        }
    }
    vir["vir_hash"] = Value::String(ZERO_SHA256.into());
    vir["vir_hash"] = Value::String(
        successor_vir_hash_value(&vir)
            .expect("successor VIR hash")
            .as_str()
            .into(),
    );
    vir
}

fn staged_artifacts() -> StagedArtifacts {
    let registry = validate_semantic_profile_registry(
        include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("installed revision-3 registry validates");
    let vir = load(ACTIVE_VIR);
    let source_map = load(ACTIVE_SOURCE_MAP);
    let source_manifest = load(ACTIVE_SOURCE_MANIFEST);
    let selection = source_manifest["selection"].clone();
    let release_registry = serde_json::from_value(source_manifest["release_registry"].clone())
        .expect("release-registry identity");

    StagedArtifacts {
        registry,
        selection,
        release_registry,
        vir,
        source_map,
        source_manifest,
    }
}

fn rehash_contracts_and_vir(vir: &mut Value) {
    for unit in vir["units"].as_array_mut().expect("VIR units") {
        for function in unit["functions"].as_array_mut().expect("VIR functions") {
            let contract = &mut function["contracts"];
            contract["contract_hash"] = Value::String(ZERO_SHA256.into());
            contract["contract_hash"] = Value::String(
                successor_contract_hash_value(contract)
                    .expect("mutated contract hashes")
                    .as_str()
                    .into(),
            );
        }
    }
    vir["vir_hash"] = Value::String(ZERO_SHA256.into());
    vir["vir_hash"] = Value::String(
        successor_vir_hash_value(vir)
            .expect("mutated VIR hashes")
            .as_str()
            .into(),
    );
}

#[test]
fn successor_artifacts_round_trip_and_use_the_frozen_domains() {
    let staged = staged_artifacts();
    let captured = captured_inputs();
    let vir = import_successor_vir_json(&canonical(&staged.vir), &staged.registry)
        .expect("successor VIR validates");
    assert_eq!(vir.canonical_bytes(), canonical(&staged.vir));
    assert_eq!(vir.module().schema(), SUCCESSOR_VIR_SCHEMA);

    let source_map = import_successor_source_map_json(
        &canonical(&staged.source_map),
        SuccessorSourceMapValidationContext {
            registry: &staged.registry,
            vir: &vir,
            captured_inputs: &captured,
            synthetic_permissions: &[],
        },
    )
    .expect("successor source map validates");
    assert_eq!(source_map.canonical_bytes(), canonical(&staged.source_map));

    let source_manifest = import_successor_source_manifest_json(
        &canonical(&staged.source_manifest),
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: &staged.registry,
            vir: &vir,
            source_map: &source_map,
            captured_inputs: &captured,
            expected_release_registry: &staged.release_registry,
        },
    )
    .expect("successor source manifest validates");
    assert_eq!(
        source_manifest.canonical_bytes(),
        canonical(&staged.source_manifest)
    );
    assert_eq!(
        source_manifest.manifest().semantic_context(),
        vir.module().semantic_context()
    );
    assert_eq!(
        source_manifest.manifest().selection().value()["function"],
        staged.selection["value"]["function"]
    );

    let migrations = &load(REGISTRY_VECTORS)["hash_domain_migration_cases"];
    let expected = [
        ("contract", SUCCESSOR_CONTRACT_HASH_DOMAIN.as_str()),
        ("vir", SUCCESSOR_VIR_HASH_DOMAIN.as_str()),
        ("source_map", SUCCESSOR_SOURCE_MAP_HASH_DOMAIN.as_str()),
        (
            "source_manifest",
            SUCCESSOR_SOURCE_MANIFEST_HASH_DOMAIN.as_str(),
        ),
    ];
    for (case, (surface, domain)) in migrations
        .as_array()
        .expect("migration cases")
        .iter()
        .take(expected.len())
        .zip(expected)
    {
        assert_eq!(case["surface"], surface);
        assert_eq!(case["successor"], domain);
        assert_ne!(case["current"], case["successor"]);
    }

    let csharp = load(CSHARP_VECTORS);
    let normalized = &csharp["normalized_contract_fixture"];
    let hash = successor_contract_hash_value(normalized).expect("normalized C# contract hashes");
    assert_eq!(hash.as_str(), normalized["contract_hash"]);
    assert_eq!(
        hash.as_str(),
        "b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab"
    );
}

#[test]
fn every_context_member_and_cross_artifact_link_is_fail_closed() {
    let staged = staged_artifacts();
    let mut mutations = [
        "/profile_registry/schema",
        "/profile_registry/id",
        "/profile_registry/revision",
        "/profile_registry/registry_sha256",
        "/profile_entry_sha256",
        "/source_language",
        "/semantic_profile",
        "/semantic_parameters/schema",
        "/semantic_parameters/value/target_id",
        "/semantic_parameters/value/pointer_width",
    ];
    for pointer in &mut mutations {
        let mut vir = staged.vir.clone();
        let member = vir["semantic_context"]
            .pointer_mut(pointer)
            .expect("semantic-context member exists");
        *member = match member {
            Value::Number(_) => json!(63),
            _ => Value::String("mismatch".into()),
        };
        rehash_contracts_and_vir(&mut vir);
        let error = import_successor_vir_json(&canonical(&vir), &staged.registry)
            .expect_err("every changed semantic-context member rejects");
        assert!(matches!(
            error.code(),
            SuccessorArtifactErrorCode::SemanticContext | SuccessorArtifactErrorCode::Linkage
        ));
    }

    let vectors = load(REGISTRY_VECTORS);
    let rust_context = vectors["fixtures"]["rust_request"]["semantic_context"].clone();
    let mut crossed_contract = staged.vir.clone();
    crossed_contract["units"][0]["functions"][0]["contracts"]["semantic_context"] =
        rust_context.clone();
    rehash_contracts_and_vir(&mut crossed_contract);
    let error = import_successor_vir_json(&canonical(&crossed_contract), &staged.registry)
        .expect_err("a valid but crossed contract context rejects");
    assert!(matches!(
        error.code(),
        SuccessorArtifactErrorCode::SemanticContext | SuccessorArtifactErrorCode::Linkage
    ));

    let captured = captured_inputs();
    let vir = import_successor_vir_json(&canonical(&staged.vir), &staged.registry)
        .expect("base successor VIR");
    let mut crossed_map = staged.source_map.clone();
    crossed_map["semantic_context"] = rust_context;
    crossed_map["source_map_hash"] = Value::String(
        successor_source_map_hash_value(&crossed_map)
            .expect("crossed map rehashes")
            .as_str()
            .into(),
    );
    let error = import_successor_source_map_json(
        &canonical(&crossed_map),
        SuccessorSourceMapValidationContext {
            registry: &staged.registry,
            vir: &vir,
            captured_inputs: &captured,
            synthetic_permissions: &[],
        },
    )
    .expect_err("a valid but crossed map context rejects");
    assert!(matches!(
        error.code(),
        SuccessorArtifactErrorCode::SemanticContext | SuccessorArtifactErrorCode::Linkage
    ));

    let source_map = import_successor_source_map_json(
        &canonical(&staged.source_map),
        SuccessorSourceMapValidationContext {
            registry: &staged.registry,
            vir: &vir,
            captured_inputs: &captured,
            synthetic_permissions: &[],
        },
    )
    .expect("base successor source map");
    for pointer in [
        "/vir_hash",
        "/source_map_hash",
        "/release_registry/registry_sha256",
        "/target/id",
        "/target/pointer_width",
        "/selection/value/function",
        "/limit_profile",
        "/frontend/binary_sha256",
        "/toolchain/distribution_sha256",
        "/toolchain/components/0/binary_sha256",
        "/inputs/0/size_bytes",
    ] {
        let mut manifest = staged.source_manifest.clone();
        let member = manifest
            .pointer_mut(pointer)
            .expect("manifest linkage member exists");
        *member = match member {
            Value::Number(_) => json!(32),
            _ => Value::String("mismatch".into()),
        };
        manifest["source_manifest_hash"] = Value::String(
            successor_source_manifest_hash_value(&manifest)
                .expect("mutated manifest rehashes")
                .as_str()
                .into(),
        );
        assert!(import_successor_source_manifest_json(
            &canonical(&manifest),
            SuccessorSourceManifestStage::Frontend,
            SuccessorSourceManifestValidationContext {
                registry: &staged.registry,
                vir: &vir,
                source_map: &source_map,
                captured_inputs: &captured,
                expected_release_registry: &staged.release_registry,
            },
        )
        .is_err());
    }
}

#[test]
fn successor_static_calls_use_callee_first_order_and_new_contract_hashes() {
    let staged = staged_artifacts();
    let rust_fixture = load(ACTIVE_RUST_CALL_VIR);
    let rust_context = &rust_fixture["semantic_context"];
    let vir = successor_vir_fixture(ACTIVE_RUST_CALL_VIR, rust_context);
    import_successor_vir_json(&canonical(&vir), &staged.registry)
        .expect("callee-first Rust successor VIR validates");

    let mut wrong_order = vir.clone();
    wrong_order["units"][0]["functions"]
        .as_array_mut()
        .expect("functions")
        .swap(1, 2);
    wrong_order["vir_hash"] = Value::String(
        successor_vir_hash_value(&wrong_order)
            .expect("wrong-order VIR rehashes")
            .as_str()
            .into(),
    );
    let error = import_successor_vir_json(&canonical(&wrong_order), &staged.registry)
        .expect_err("caller-before-callee order rejects");
    assert_eq!(error.code(), SuccessorArtifactErrorCode::Order);

    let mut wrong_call_hash = vir;
    wrong_call_hash["units"][0]["functions"][1]["blocks"][0]["instructions"][0]["contract_hash"] =
        Value::String(ZERO_SHA256.into());
    wrong_call_hash["vir_hash"] = Value::String(
        successor_vir_hash_value(&wrong_call_hash)
            .expect("wrong-call-hash VIR rehashes")
            .as_str()
            .into(),
    );
    let error = import_successor_vir_json(&canonical(&wrong_call_hash), &staged.registry)
        .expect_err("a stale v0 or crossed callee contract hash rejects");
    assert_eq!(error.code(), SuccessorArtifactErrorCode::Linkage);
}

#[test]
fn current_and_successor_artifact_parsers_reject_each_other_without_adapters() {
    let staged = staged_artifacts();
    let captured = captured_inputs();

    let successor_vir_bytes = canonical(&staged.vir);
    assert!(import_vir_json(&successor_vir_bytes).is_err());
    let mut predecessor = staged.vir.clone();
    predecessor["schema"] = json!("mpk.vir.v0");
    assert!(import_successor_vir_json(&canonical(&predecessor), &staged.registry).is_err());
    assert!(serde_json::from_slice::<SourceManifest>(&canonical(&staged.source_manifest)).is_err());
    let mut predecessor_manifest = staged.source_manifest.clone();
    predecessor_manifest["schema"] = json!("mpk.source_manifest.v0");
    assert!(import_successor_source_manifest_json(
        &canonical(&predecessor_manifest),
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: &staged.registry,
            vir: &import_successor_vir_json(&successor_vir_bytes, &staged.registry)
                .expect("successor VIR"),
            source_map: &import_successor_source_map_json(
                &canonical(&staged.source_map),
                SuccessorSourceMapValidationContext {
                    registry: &staged.registry,
                    vir: &import_successor_vir_json(&successor_vir_bytes, &staged.registry)
                        .expect("successor VIR"),
                    captured_inputs: &captured,
                    synthetic_permissions: &[],
                },
            )
            .expect("successor map"),
            captured_inputs: &captured,
            expected_release_registry: &staged.release_registry,
        },
    )
    .is_err());
}
