use mpk_vc::{
    attach_vc_hash, canonical_json_bytes, import_certificate_source_manifest_json,
    import_frontend_source_manifest_json, import_source_map_json, import_vir_json, input_set_hash,
    parse_strict_json, source_manifest_hash, validate_component_identity,
    validate_language_configuration, validate_manifest_normalized_path,
    validate_source_manifest_canonical_size, validate_source_manifest_input_count,
    validate_source_manifest_transition, CapturedInput, ComponentIdentity, InputKind,
    LanguageConfiguration, SemanticParameters, SemanticProfile, SourceLanguage, SourceManifest,
    SourceManifestError, SourceManifestValidationContext, SourceMapValidationContext,
    StrictJsonLimits, ValidatedReleaseRegistry, ValidatedSourceMap, ValidatedVcIdentity, VirModule,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const ALL_JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);

#[test]
fn source_manifest_vectors_are_exhaustive_and_match_normative_outcomes() {
    let vectors = load_json("develop/specs/vectors/source-manifest-v0.json");
    assert_exact_keys(
        &vectors,
        &[
            "schema",
            "spec_schema",
            "dependencies",
            "owner_tests",
            "fixture_inputs",
            "manifest_cases",
            "configuration_cases",
            "release_cases",
            "input_cases",
            "lifecycle_cases",
            "path_cases",
            "hash_cases",
            "limit_cases",
        ],
    );
    assert_eq!(vectors["schema"], "mpk.source_manifest.conformance.v0");
    assert_eq!(vectors["spec_schema"], "mpk.source_manifest.v0");
    assert_exact_keys(
        &vectors["dependencies"],
        &[
            "vir_vector",
            "vir_case",
            "source_map_vector",
            "source_map_case",
            "release_vector",
            "release_case",
        ],
    );
    assert_eq!(
        vectors["owner_tests"],
        json!([
            "crates/mpk-vc/tests/source_manifest.rs",
            "go-tools/go2vir/bundle_candidate_test.go",
            "rust-tools/rust2vir/tests/frontend_envelope.rs"
        ])
    );

    let vir = valid_vir();
    let capture_storage = fixture_inputs(&vectors);
    let captured_inputs = captured_refs(&capture_storage);
    let source_map = valid_source_map(&vir, &capture_storage);
    let registry = valid_registry();
    let context = SourceManifestValidationContext {
        vir: &vir,
        source_map: &source_map,
        captured_inputs: &captured_inputs,
        release_registry: &registry,
        expected_language_configuration: None,
    };
    let base = vectors["manifest_cases"][0]["input"].clone();
    let valid_frontend =
        import_frontend_source_manifest_json(&serde_json::to_vec(&base).unwrap(), context)
            .expect("normative frontend manifest must validate");
    assert_eq!(
        valid_frontend.canonical_bytes().len() as u64,
        vectors["manifest_cases"][0]["expect"]["canonical_jcs_utf8_length"]
            .as_u64()
            .unwrap()
    );
    assert_non_vector_collection_limits(&base, context);
    assert_input_subphase_precedence(&base, context);
    assert_source_map_capture_swap_rejected(&base, &vir, &source_map, &registry, &capture_storage);

    let mut all_ids = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for group in [
        "manifest_cases",
        "configuration_cases",
        "release_cases",
        "input_cases",
        "lifecycle_cases",
        "path_cases",
        "hash_cases",
        "limit_cases",
    ] {
        for case in vectors[group].as_array().expect("case array") {
            assert_case_shape(group, case);
            let id = case["id"].as_str().expect("case id");
            assert!(all_ids.insert(id.to_owned()), "duplicate case ID {id}");
            visited.insert(id.to_owned());
            match group {
                "configuration_cases" => run_configuration_case(case, &vectors),
                "release_cases" if case["context"]["validator"] == "component_identity" => {
                    run_component_case(case)
                }
                "lifecycle_cases" => run_lifecycle_case(case, &base, context),
                "hash_cases" => run_hash_case(case, &base, context),
                "limit_cases" => run_limit_case(case),
                _ => run_frontend_case(case, &base, context),
            }
        }
    }
    assert_eq!(
        visited, all_ids,
        "a normative source-manifest case was skipped"
    );
}

fn assert_input_subphase_precedence(base: &Value, context: SourceManifestValidationContext<'_>) {
    let mut value = base.clone();
    value["inputs"][0]["sha256"] = Value::String("0".repeat(64));
    value["inputs"][1]["kind"] = Value::String("source".to_owned());
    let error = import_frontend_source_manifest_json(&serde_json::to_vec(&value).unwrap(), context)
        .expect_err("input-kind inventory must be checked before captured byte digests");
    assert_eq!(error.phase.as_str(), "inputs");
    assert_eq!(error.code.as_str(), "SOURCE_MANIFEST_INPUT_KIND");
}

fn assert_source_map_capture_swap_rejected(
    base: &Value,
    vir: &VirModule,
    source_map: &ValidatedSourceMap,
    registry: &ValidatedReleaseRegistry,
    original_storage: &[(InputKind, String, Vec<u8>)],
) {
    let mut altered_storage = original_storage.to_vec();
    let altered_source_hash = {
        let (_, _, source_bytes) = altered_storage
            .iter_mut()
            .find(|(kind, path, _)| *kind == InputKind::Source && path == "identity.go")
            .unwrap();
        source_bytes[0] = b'P';
        sha256_hex(source_bytes)
    };
    let altered_captures = captured_refs(&altered_storage);

    let mut manifest: SourceManifest = serde_json::from_value(base.clone()).unwrap();
    let source = manifest
        .inputs
        .iter_mut()
        .find(|input| input.kind == InputKind::Source && input.normalized_path == "identity.go")
        .unwrap();
    source.sha256 = altered_source_hash;
    manifest.input_set_hash = input_set_hash(&manifest.inputs)
        .unwrap()
        .as_str()
        .to_owned();
    manifest.source_manifest_hash = "0".repeat(64);
    manifest.source_manifest_hash = source_manifest_hash(&manifest).unwrap().as_str().to_owned();

    let error = import_frontend_source_manifest_json(
        &serde_json::to_vec(&manifest).unwrap(),
        SourceManifestValidationContext {
            vir,
            source_map,
            captured_inputs: &altered_captures,
            release_registry: registry,
            expected_language_configuration: None,
        },
    )
    .expect_err("manifest must not swap bytes after source-map validation");
    assert_eq!(error.phase.as_str(), "artifacts");
    assert_eq!(error.code.as_str(), "SOURCE_MANIFEST_SOURCE_MAP_LINKAGE");
}

fn assert_non_vector_collection_limits(base: &Value, context: SourceManifestValidationContext<'_>) {
    let cases = [
        ("/units", Value::Array(vec![base["units"][0].clone(); 257])),
        (
            "/toolchain/components",
            Value::Array(vec![base["toolchain"]["components"][0].clone(); 8_193]),
        ),
        (
            "/frontend/subordinate_binaries",
            json!([{"name":"extra","version":"v1","binary_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]),
        ),
        (
            "/target/language_configuration/cfg",
            Value::Array(
                (0..16_385)
                    .map(|index| Value::String(format!("cfg{index:05}")))
                    .collect(),
            ),
        ),
    ];
    for (pointer, replacement) in cases {
        let mut value = base.clone();
        if pointer.ends_with("/cfg") {
            value["target"]["language_configuration"]
                .as_object_mut()
                .unwrap()
                .insert("cfg".to_owned(), Value::Array(Vec::new()));
        }
        *value.pointer_mut(pointer).unwrap() = replacement;
        let error =
            import_frontend_source_manifest_json(&serde_json::to_vec(&value).unwrap(), context)
                .expect_err("collection above its inclusive limit must reject");
        assert_eq!(error.phase.as_str(), "transport", "{pointer}");
        assert_eq!(error.code.as_str(), "SOURCE_MANIFEST_LIMIT", "{pointer}");
    }

    let mut rust_sources = base.clone();
    rust_sources["source_language"] = Value::String("rust".to_owned());
    rust_sources["inputs"] = Value::Array((0..257).map(|_| json!({"kind":"source"})).collect());
    let error =
        import_frontend_source_manifest_json(&serde_json::to_vec(&rust_sources).unwrap(), context)
            .expect_err("Rust compiled-source count above 256 must reject");
    assert_eq!(error.phase.as_str(), "transport");
    assert_eq!(error.code.as_str(), "SOURCE_MANIFEST_LIMIT");
}

fn run_frontend_case(case: &Value, base: &Value, context: SourceManifestValidationContext<'_>) {
    let id = case["id"].as_str().unwrap();
    let bytes = if let Some(text) = case.get("json_text").and_then(Value::as_str) {
        text.as_bytes().to_vec()
    } else {
        let value = if let Some(input) = case.get("input") {
            input.clone()
        } else {
            apply_construction(base.clone(), &case["construction"])
        };
        serde_json::to_vec(&value).unwrap()
    };
    let result = import_frontend_source_manifest_json(&bytes, context).map(|_| ());
    assert_expected(case, result, id);
}

fn run_configuration_case(case: &Value, vectors: &Value) {
    let id = case["id"].as_str().unwrap();
    let (language, profile, input) = if id == "configuration.valid_rust_core" {
        let base = vectors["configuration_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["id"] == "configuration.valid_rust_std")
            .unwrap()["input"]
            .clone();
        (
            SourceLanguage::Rust,
            SemanticProfile::RustCheckedV0,
            apply_construction(base, &case["construction"]),
        )
    } else if id == "configuration.reject_wrong_language_branch" {
        let input = vectors["configuration_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["id"] == "configuration.valid_go")
            .unwrap()["input"]
            .clone();
        (SourceLanguage::Rust, SemanticProfile::RustCheckedV0, input)
    } else if case.get("construction").is_some() {
        let base = vectors["configuration_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["id"] == "configuration.valid_rust_std")
            .unwrap()["input"]
            .clone();
        (
            SourceLanguage::Rust,
            SemanticProfile::RustCheckedV0,
            apply_construction(base, &case["construction"]),
        )
    } else {
        let language = if case["source_language"] == "go" {
            SourceLanguage::Go
        } else {
            SourceLanguage::Rust
        };
        let profile = if language == SourceLanguage::Go {
            SemanticProfile::GoFixedV0
        } else {
            SemanticProfile::RustCheckedV0
        };
        (language, profile, case["input"].clone())
    };
    let parsed: Result<LanguageConfiguration, _> = serde_json::from_value(input);
    let result = match parsed {
        Ok(configuration) => validate_language_configuration(language, profile, &configuration),
        Err(error) => Err(SourceManifestError {
            phase: mpk_vc::SourceManifestValidationPhase::Shape,
            code: mpk_vc::SourceManifestErrorCode::Shape,
            detail: error.to_string(),
        }),
    };
    assert_expected(case, result, id);
}

fn run_component_case(case: &Value) {
    let id = case["id"].as_str().unwrap();
    let parsed: Result<ComponentIdentity, _> = serde_json::from_value(case["input"].clone());
    let result = match parsed {
        Ok(component) => validate_component_identity(
            &component,
            SourceLanguage::Rust,
            case["context"]["compiler_release"].as_str(),
            case["context"]["rustc_commit"].as_str(),
        ),
        Err(error) => Err(SourceManifestError {
            phase: mpk_vc::SourceManifestValidationPhase::Shape,
            code: mpk_vc::SourceManifestErrorCode::Shape,
            detail: error.to_string(),
        }),
    };
    assert_expected(case, result, id);
}

fn run_lifecycle_case(case: &Value, base: &Value, context: SourceManifestValidationContext<'_>) {
    let id = case["id"].as_str().unwrap();
    let valid_vc = vector_vc(&case_or_valid_vc(case));
    let result = match id {
        "lifecycle.valid_certificate_stage" => {
            let frontend = canonical_bytes(base);
            attach_vc_hash(&frontend, context, &valid_vc).map(|manifest| {
                assert_eq!(
                    manifest.hash().as_str(),
                    case["construction"]["expected_source_manifest_hash"]
                );
                assert_eq!(
                    manifest.canonical_bytes().len() as u64,
                    case["construction"]["expected_canonical_jcs_utf8_length"]
                        .as_u64()
                        .unwrap()
                );
            })
        }
        "lifecycle.reject_vc_at_frontend_stage" => {
            let value = apply_construction(base.clone(), &case["construction"]);
            import_frontend_source_manifest_json(&serde_json::to_vec(&value).unwrap(), context)
                .map(|_| ())
        }
        "lifecycle.reject_missing_vc_at_certificate_stage" => {
            import_certificate_source_manifest_json(
                &serde_json::to_vec(base).unwrap(),
                context,
                &valid_vc,
            )
            .map(|_| ())
        }
        "lifecycle.reject_mismatched_vc" => {
            let vc_value = case_or_valid_vc(case);
            let mut mismatched = vc_value;
            apply_patches(&mut mismatched, &case["construction"]["vc_patches"]);
            let vc = vector_vc(&mismatched);
            attach_vc_hash(&canonical_bytes(base), context, &vc).map(|_| ())
        }
        "lifecycle.reject_other_field_mutation" => {
            let certificate = attach_vc_hash(&canonical_bytes(base), context, &valid_vc).unwrap();
            let mut mutated: Value = serde_json::from_slice(certificate.canonical_bytes()).unwrap();
            apply_patches(&mut mutated, &case["construction"]["patches"]);
            validate_source_manifest_transition(
                &canonical_bytes(base),
                &canonical_bytes(&mutated),
                context,
                &valid_vc,
            )
            .map(|_| ())
        }
        _ => unreachable!(),
    };
    assert_expected(case, result, id);
}

fn run_hash_case(case: &Value, base: &Value, context: SourceManifestValidationContext<'_>) {
    let id = case["id"].as_str().unwrap();
    let manifest: SourceManifest = serde_json::from_value(base.clone()).unwrap();
    match id {
        "hash.valid_input_set" => {
            assert_eq!(
                input_set_hash(&manifest.inputs).unwrap().as_str(),
                case["expected_sha256"]
            );
            let strict = parse_strict_json(
                &serde_json::to_vec(&manifest.inputs).unwrap(),
                ALL_JSON_LIMITS,
            )
            .unwrap();
            assert_eq!(
                canonical_json_bytes(&strict).unwrap().len() as u64,
                case["canonical_preimage_utf8_length"].as_u64().unwrap()
            );
        }
        "hash.valid_frontend_manifest" => {
            assert_eq!(
                source_manifest_hash(&manifest).unwrap().as_str(),
                case["expected_sha256"]
            );
            let strict =
                parse_strict_json(&serde_json::to_vec(&manifest).unwrap(), ALL_JSON_LIMITS)
                    .unwrap();
            let preimage = strict
                .clone_without_fields(&["source_manifest_hash"])
                .unwrap();
            assert_eq!(
                canonical_json_bytes(&preimage).unwrap().len() as u64,
                case["canonical_without_hash_utf8_length"].as_u64().unwrap()
            );
        }
        "hash.valid_certificate_manifest" => {
            let vc = vector_vc(&case_or_valid_vc(case));
            let certificate = attach_vc_hash(&canonical_bytes(base), context, &vc).unwrap();
            assert_eq!(certificate.hash().as_str(), case["expected_sha256"]);
            let strict = parse_strict_json(certificate.canonical_bytes(), ALL_JSON_LIMITS).unwrap();
            let preimage = strict
                .clone_without_fields(&["source_manifest_hash"])
                .unwrap();
            assert_eq!(
                canonical_json_bytes(&preimage).unwrap().len() as u64,
                case["canonical_without_hash_utf8_length"].as_u64().unwrap()
            );
        }
        "hash.reject_wrong_manifest_hash" => run_frontend_case(case, base, context),
        _ => {
            let source_value = if case.get("source_pointer").is_some() {
                base["inputs"].clone()
            } else {
                let mut value = base.clone();
                value
                    .as_object_mut()
                    .unwrap()
                    .remove("source_manifest_hash");
                value
            };
            let canonical = canonical_json_bytes(
                &parse_strict_json(&serde_json::to_vec(&source_value).unwrap(), ALL_JSON_LIMITS)
                    .unwrap(),
            )
            .unwrap();
            let domain = case
                .get("wrong_domain_utf8")
                .and_then(Value::as_str)
                .unwrap_or_else(|| {
                    if case.get("source_pointer").is_some() {
                        "MPK-INPUT-SET-0.1"
                    } else {
                        "MPK-SOURCE-MANIFEST-0.1"
                    }
                });
            let separator = case
                .get("wrong_separator_hex")
                .and_then(Value::as_str)
                .unwrap_or("00");
            let mut bytes = domain.as_bytes().to_vec();
            if separator == "00" {
                bytes.push(0);
            }
            bytes.extend(canonical);
            let actual = sha256_hex(&bytes);
            let expected = case["wrong_domain_sha256"]
                .as_str()
                .or_else(|| case["wrong_separator_sha256"].as_str())
                .unwrap();
            assert_eq!(actual, expected);
        }
    }
}

fn run_limit_case(case: &Value) {
    let id = case["id"].as_str().unwrap();
    let count = case["construction"]["count"].as_u64().unwrap();
    let result = if id.contains("canonical_bytes") {
        validate_source_manifest_canonical_size(count)
    } else if id.contains("inputs") {
        let language = if id.contains("rust") {
            SourceLanguage::Rust
        } else {
            SourceLanguage::Go
        };
        validate_source_manifest_input_count(language, count)
    } else {
        validate_manifest_normalized_path(&portable_path(count as usize))
    };
    assert_expected(case, result, id);
}

fn valid_vir() -> VirModule {
    let vectors = load_json("develop/specs/vectors/vir-v0.json");
    let input = vectors["module_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "module.valid_go_identity")
        .unwrap()["input"]
        .clone();
    import_vir_json(&serde_json::to_vec(&input).unwrap()).unwrap()
}

fn valid_source_map(
    vir: &VirModule,
    captures: &[(InputKind, String, Vec<u8>)],
) -> ValidatedSourceMap {
    let vectors = load_json("develop/specs/vectors/source-map-v0.json");
    let input = vectors["map_cases"][0]["input"].clone();
    let refs: Vec<_> = captures
        .iter()
        .filter(|(kind, _, _)| *kind == InputKind::Source)
        .map(|(kind, path, bytes)| CapturedInput {
            kind: *kind,
            normalized_path: path,
            bytes,
        })
        .collect();
    import_source_map_json(
        &serde_json::to_vec(&input).unwrap(),
        SourceMapValidationContext {
            vir,
            captured_inputs: &refs,
            synthetic_permissions: &[],
        },
    )
    .unwrap()
}

fn valid_registry() -> ValidatedReleaseRegistry {
    let vectors = load_json("develop/specs/vectors/release-bundles-v0.json");
    mpk_vc::validate_release_registry(&canonical_transport(&vectors["fixtures"]["valid_registry"]))
        .unwrap()
}

fn fixture_inputs(vectors: &Value) -> Vec<(InputKind, String, Vec<u8>)> {
    vectors["fixture_inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fixture| {
            assert_exact_keys(
                fixture,
                &[
                    "id",
                    "kind",
                    "normalized_path",
                    "base64",
                    "size_bytes",
                    "sha256",
                ],
            );
            let kind: InputKind =
                serde_json::from_value(fixture["kind"].clone()).expect("known input kind");
            let bytes = decode_base64(fixture["base64"].as_str().unwrap());
            assert_eq!(bytes.len() as u64, fixture["size_bytes"].as_u64().unwrap());
            assert_eq!(sha256_hex(&bytes), fixture["sha256"]);
            (
                kind,
                fixture["normalized_path"].as_str().unwrap().to_owned(),
                bytes,
            )
        })
        .collect()
}

fn captured_refs(storage: &[(InputKind, String, Vec<u8>)]) -> Vec<CapturedInput<'_>> {
    storage
        .iter()
        .map(|(kind, path, bytes)| CapturedInput {
            kind: *kind,
            normalized_path: path,
            bytes,
        })
        .collect()
}

fn case_or_valid_vc(case: &Value) -> Value {
    case.pointer("/construction/vc")
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "input_set_hash": "e05e9fa46ee44a198470ada4935f756b12e9d6779601a7f88e4cb468d151ab31",
                "source_ir_schema": "mpk.vir.v0",
                "source_ir_hash": "374dbbcc0c9454bf29c0117c02f1bbdc0424df970297af9fe4560512d40d0690",
                "semantic_profile": "mpk.go.fixed.v0",
                "semantic_parameters": {"target_id":"linux/amd64","pointer_width":64},
                "vc_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            })
        })
}

fn vector_vc(value: &Value) -> ValidatedVcIdentity {
    ValidatedVcIdentity::new(
        value["input_set_hash"].as_str().unwrap().to_owned(),
        value["source_ir_schema"].as_str().unwrap().to_owned(),
        value["source_ir_hash"].as_str().unwrap().to_owned(),
        serde_json::from_value(value["semantic_profile"].clone()).unwrap(),
        serde_json::from_value::<SemanticParameters>(value["semantic_parameters"].clone()).unwrap(),
        value["vc_hash"].as_str().unwrap().to_owned(),
    )
    .unwrap()
}

fn apply_construction(mut value: Value, construction: &Value) -> Value {
    if construction["fixture"] == "swap_inputs" {
        let indices = construction["indices"].as_array().unwrap();
        value["inputs"].as_array_mut().unwrap().swap(
            indices[0].as_u64().unwrap() as usize,
            indices[1].as_u64().unwrap() as usize,
        );
    }
    apply_patches(&mut value, &construction["patches"]);
    value
}

fn apply_patches(value: &mut Value, patches: &Value) {
    if let Some(patches) = patches.as_array() {
        for patch in patches {
            apply_patch(value, patch);
        }
    }
}

fn apply_patch(root: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().unwrap();
    let (parent_path, token) = path.rsplit_once('/').unwrap();
    let parent = if parent_path.is_empty() {
        root
    } else {
        root.pointer_mut(parent_path).unwrap()
    };
    let token = token.replace("~1", "/").replace("~0", "~");
    match patch["op"].as_str().unwrap() {
        "replace" => *child_mut(parent, &token) = patch["value"].clone(),
        "remove" => match parent {
            Value::Array(values) => {
                values.remove(token.parse::<usize>().unwrap());
            }
            Value::Object(values) => {
                values.remove(&token);
            }
            _ => panic!("patch parent is not a container"),
        },
        "add" => match parent {
            Value::Array(values) => {
                let index = token.parse::<usize>().unwrap();
                if index == values.len() {
                    values.push(patch["value"].clone());
                } else {
                    values.insert(index, patch["value"].clone());
                }
            }
            Value::Object(values) => {
                values.insert(token, patch["value"].clone());
            }
            _ => panic!("patch parent is not a container"),
        },
        operation => panic!("unknown patch operation {operation}"),
    }
}

fn child_mut<'a>(parent: &'a mut Value, token: &str) -> &'a mut Value {
    match parent {
        Value::Array(values) => &mut values[token.parse::<usize>().unwrap()],
        Value::Object(values) => values.get_mut(token).unwrap(),
        _ => panic!("patch parent is not a container"),
    }
}

fn assert_expected(case: &Value, result: Result<(), SourceManifestError>, id: &str) {
    match case["expect"]["outcome"].as_str().unwrap() {
        "accept" => result.unwrap_or_else(|error| panic!("{id} rejected: {error}")),
        "reject" => {
            let error = result.expect_err(&format!("{id} unexpectedly accepted"));
            assert_eq!(error.phase.as_str(), case["expect"]["phase"], "{id}");
            assert_eq!(error.code.as_str(), case["expect"]["code"], "{id}");
        }
        outcome => panic!("unknown outcome {outcome}"),
    }
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected);
}

fn assert_case_shape(group: &str, case: &Value) {
    let allowed: BTreeSet<&str> = match group {
        "manifest_cases" => [
            "id",
            "context",
            "input",
            "json_text",
            "construction",
            "expect",
        ]
        .into_iter()
        .collect(),
        "configuration_cases" => [
            "id",
            "context",
            "source_language",
            "semantic_profile",
            "input",
            "input_from",
            "construction",
            "expect",
        ]
        .into_iter()
        .collect(),
        "release_cases" => ["id", "context", "input", "construction", "expect"]
            .into_iter()
            .collect(),
        "input_cases" | "path_cases" | "limit_cases" => {
            ["id", "construction", "expect"].into_iter().collect()
        }
        "lifecycle_cases" => ["id", "construction", "input_from", "context", "expect"]
            .into_iter()
            .collect(),
        "hash_cases" => [
            "id",
            "source_case",
            "source_pointer",
            "domain_utf8",
            "wrong_domain_utf8",
            "separator_hex",
            "wrong_separator_hex",
            "canonical_preimage_utf8_length",
            "canonical_without_hash_utf8_length",
            "expected_sha256",
            "wrong_domain_sha256",
            "wrong_separator_sha256",
            "expect_different",
            "construction",
            "expect",
        ]
        .into_iter()
        .collect(),
        _ => unreachable!(),
    };
    for key in case.as_object().unwrap().keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unknown {group} case field {key}"
        );
    }
    if let Some(context) = case.get("context") {
        assert_allowed_keys(
            context,
            &[
                "lifecycle",
                "input_cases",
                "vir_case",
                "source_map_case",
                "release_case",
                "validator",
                "source_language",
                "compiler_release",
                "rustc_commit",
            ],
            "source-manifest case context",
        );
    }
    if let Some(construction) = case.get("construction") {
        assert_allowed_keys(
            construction,
            &[
                "base",
                "context",
                "count",
                "expected_canonical_jcs_utf8_length",
                "expected_source_manifest_hash",
                "fixture",
                "indices",
                "patches",
                "pointer",
                "source_language",
                "vc",
                "vc_from",
                "vc_patches",
            ],
            "source-manifest construction",
        );
        assert_patches(construction.get("patches"));
        assert_patches(construction.get("vc_patches"));
        if let Some(context) = construction.get("context") {
            assert_allowed_keys(
                context,
                &["lifecycle", "validator"],
                "source-manifest construction context",
            );
        }
        if let Some(vc) = construction.get("vc") {
            assert_exact_keys(
                vc,
                &[
                    "input_set_hash",
                    "source_ir_schema",
                    "source_ir_hash",
                    "semantic_profile",
                    "semantic_parameters",
                    "vc_hash",
                ],
            );
        }
    }
    if let Some(expect) = case.get("expect") {
        assert_allowed_keys(
            expect,
            &["outcome", "phase", "code", "canonical_jcs_utf8_length"],
            "source-manifest expectation",
        );
    }
}

fn assert_patches(patches: Option<&Value>) {
    let Some(patches) = patches.and_then(Value::as_array) else {
        return;
    };
    for patch in patches {
        let expected = if patch.get("value").is_some() {
            vec!["op", "path", "value"]
        } else {
            vec!["op", "path"]
        };
        assert_exact_keys(patch, &expected);
    }
}

fn assert_allowed_keys(value: &Value, allowed: &[&str], owner: &str) {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    for key in value.as_object().unwrap().keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unknown {owner} field {key}"
        );
    }
}

fn canonical_bytes(value: &Value) -> Vec<u8> {
    canonical_json_bytes(
        &parse_strict_json(&serde_json::to_vec(value).unwrap(), ALL_JSON_LIMITS).unwrap(),
    )
    .unwrap()
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = canonical_bytes(value);
    bytes.push(b'\n');
    bytes
}

fn portable_path(count: usize) -> String {
    let component_count = count / 256 + 1;
    let mut letters = count - (component_count - 1);
    let mut components = Vec::with_capacity(component_count);
    for index in 0..component_count {
        let remaining_components = component_count - index - 1;
        let length = (letters - remaining_components).min(255);
        components.push("a".repeat(length));
        letters -= length;
    }
    let path = components.join("/");
    assert_eq!(path.len(), count);
    path
}

fn load_json(path: &str) -> Value {
    serde_json::from_slice(&fs::read(repository_root().join(path)).unwrap()).unwrap()
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn decode_base64(input: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in input.bytes().take_while(|byte| *byte != b'=') {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
            accumulator &= (1_u32 << bits) - 1;
        }
    }
    output
}

fn sha256_hex(input: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(input);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
