use mpk_vc::release_bundle_v1::validate_successor_release_registry;
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_inventory, hash_canonical_json, parse_strict_json,
    sha256_raw_file_bytes, validate_release_limit, validate_release_registry, ExecutableRuntime,
    HashDomain, ReleaseRegistryErrorCode, ReleaseSelectionError, ReleaseSelectionRequest,
    ReleaseValidationPhase, StrictJsonLimits, StrictJsonValue, BUNDLE_CONTENT_HASH_DOMAIN,
    BUNDLE_REGISTRY_HASH_DOMAIN,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(68 * 1024 * 1024, 68 * 1024 * 1024, 256, 2 * 1024 * 1024);

#[test]
fn release_bundle_model_vector_records_are_closed_and_case_ids_are_global() {
    let vectors = load_vectors();
    assert_exact_keys(
        &vectors,
        &[
            "schema",
            "spec_schemas",
            "owner_tests",
            "fixtures",
            "registry_cases",
            "inventory_cases",
            "installation_cases",
            "selection_cases",
            "assembler_cases",
            "limit_cases",
            "hash_cases",
        ],
    );
    assert_exact_keys(
        field(&vectors, "fixtures"),
        &["bootstrap_registry", "valid_registry", "bundle_bytes"],
    );

    let case_arrays = [
        "registry_cases",
        "inventory_cases",
        "installation_cases",
        "selection_cases",
        "assembler_cases",
        "limit_cases",
        "hash_cases",
    ];
    let mut ids = BTreeSet::new();
    for array_name in case_arrays {
        for case in array(field(&vectors, array_name)) {
            let id = text(field(case, "id"));
            assert!(ids.insert(id), "duplicate global case ID {id}");
        }
    }
    assert_eq!(ids.len(), 138, "frozen vector case total changed");

    for case in array(field(&vectors, "registry_cases")) {
        if case.get("json_text").is_some() {
            assert_exact_keys(case, &["id", "json_text", "expect"]);
        } else {
            assert_exact_keys(case, &["id", "construction", "expect"]);
            assert_allowed_keys(
                field(case, "construction"),
                &["fixture", "operations", "rehash_registry"],
            );
            assert_operations_closed(field(field(case, "construction"), "operations"));
        }
        assert_registry_expect_closed(field(case, "expect"), false);
    }
    for case in array(field(&vectors, "inventory_cases")) {
        assert_exact_keys(case, &["id", "construction", "expect"]);
        assert_allowed_keys(
            field(case, "construction"),
            &[
                "fixture",
                "operations",
                "pointer",
                "rehash_content",
                "rehash_registry",
            ],
        );
        assert_operations_closed(field(field(case, "construction"), "operations"));
        assert_registry_expect_closed(field(case, "expect"), false);
    }
    for case in array(field(&vectors, "selection_cases")) {
        if case.get("input").is_some() {
            assert_exact_keys(case, &["id", "input", "expect"]);
            assert_exact_keys(
                field(case, "input"),
                &[
                    "schema",
                    "source_language",
                    "execution_host_profiles",
                    "native_runtime_layout_profiles",
                    "frontend_bundles",
                    "toolchain_bundles",
                    "tuples",
                ],
            );
        } else {
            assert_exact_keys(case, &["id", "registry_fixture", "request", "expect"]);
            assert_allowed_keys(
                field(case, "request"),
                &[
                    "registry_id",
                    "registry_sha256",
                    "source_language",
                    "semantic_profile",
                    "target_id",
                    "frontend_bundle_id",
                    "toolchain_bundle_id",
                ],
            );
        }
        assert_registry_expect_closed(field(case, "expect"), true);
    }
    for case in array(field(&vectors, "limit_cases")) {
        assert_exact_keys(case, &["id", "construction", "expect"]);
        assert_exact_keys(field(case, "construction"), &["kind", "value"]);
        assert_registry_expect_closed(field(case, "expect"), false);
    }
    for case in array(field(&vectors, "hash_cases")) {
        assert_allowed_keys(
            case,
            &[
                "id",
                "domain",
                "canonical_payload_utf8_length",
                "expected_preimage_length",
                "expected_sha256",
                "canonical_payload",
                "canonical_payload_sha256",
                "fixture",
                "pointer",
                "remove_pointer",
                "different_from",
            ],
        );
    }
}

#[test]
fn release_bundle_registry_and_inventory_vectors_match_all_model_outcomes() {
    let vectors = load_vectors();
    let fixtures = object(field(&vectors, "fixtures"));
    let mut seen = BTreeSet::new();

    for case in array(field(&vectors, "registry_cases")) {
        let id = text(field(case, "id"));
        assert!(seen.insert(id.to_owned()), "duplicate case ID {id}");
        let input = if let Some(json_text) = case.get("json_text") {
            text(json_text).as_bytes().to_vec()
        } else {
            let construction = field(case, "construction");
            let mut registry = field_value(fixtures, text(field(construction, "fixture"))).clone();
            apply_operations(&mut registry, field(construction, "operations"));
            if construction
                .get("rehash_registry")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                rehash_registry(&mut registry);
            }
            canonical_transport(&registry)
        };
        assert_registry_outcome(id, &input, field(case, "expect"));
    }

    for case in array(field(&vectors, "inventory_cases")) {
        let id = text(field(case, "id"));
        assert!(seen.insert(id.to_owned()), "duplicate case ID {id}");
        let construction = field(case, "construction");
        let mut registry = field_value(fixtures, text(field(construction, "fixture"))).clone();
        apply_operations(&mut registry, field(construction, "operations"));
        if let Some(pointers) = construction.get("rehash_content") {
            for pointer in array(pointers) {
                rehash_content(&mut registry, text(pointer));
            }
        }
        if construction
            .get("rehash_registry")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            rehash_registry(&mut registry);
        }
        assert_registry_outcome(id, &canonical_transport(&registry), field(case, "expect"));
    }

    assert_eq!(
        seen.len(),
        28,
        "every model registry/inventory case is owned"
    );
}

#[test]
fn cgroup2_tmpfs_execution_host_profile_is_closed_and_versioned() {
    let vectors = load_vectors();
    let mut registry = field(field(&vectors, "fixtures"), "valid_registry").clone();
    let profile = &mut registry["execution_host_profiles"][0];
    profile["minimum_kernel_abi"] = Value::String("6.4.0".to_owned());
    profile["probe_profile_id"] =
        Value::String("mpk.release.probe.linux_namespaces_cgroup2_tmpfs.v0".to_owned());
    profile["required_primitives"] = serde_json::json!([
        "filesystem.atomic_no_replace",
        "filesystem.immutable_handle",
        "filesystem.no_follow_open",
        "filesystem.tmpfs_allocated_blocks",
        "filesystem.tmpfs_inode_limit",
        "isolation.cgroup_v2",
        "isolation.mount_namespace",
        "isolation.network_namespace",
        "isolation.user_namespace",
        "memory.cgroup_accounting",
        "mount.no_exec",
        "mount.read_only",
        "mount.tmpfs_noswap",
        "process.cgroup_tasks",
        "process.closed_environment",
        "process.no_new_privileges",
        "process.rlimit_address_space",
        "process.rlimit_open_files",
        "process.task_tree_kill"
    ]);
    rehash_registry(&mut registry);
    validate_release_registry(&canonical_transport(&registry))
        .expect("the new closed execution-host profile validates");

    let profile = &mut registry["execution_host_profiles"][0];
    profile["minimum_kernel_abi"] = Value::String("6.3.0".to_owned());
    rehash_registry(&mut registry);
    let error = validate_release_registry(&canonical_transport(&registry))
        .expect_err("the profile must reject a different minimum kernel");
    assert_eq!(error.phase(), ReleaseValidationPhase::Scalar);
}

#[test]
fn registry_shape_precedes_streamed_scalar_count_limits_without_full_vectors() {
    let vectors = load_vectors();
    let mut registry = field(field(&vectors, "fixtures"), "valid_registry").clone();
    let exemplar = registry["frontend_bundles"]
        .as_array()
        .and_then(|bundles| bundles.first())
        .cloned()
        .expect("tracked registry has a frontend bundle");
    registry["frontend_bundles"] = Value::Array(vec![exemplar; 1_025]);

    let mut transport = serde_json::to_vec(&registry).expect("oversized registry serializes");
    transport.push(b'\n');
    let limit = validate_release_registry(&transport)
        .expect_err("descriptor count above the scalar limit rejects");
    assert_eq!(limit.phase(), ReleaseValidationPhase::Scalar);
    assert_eq!(limit.code(), ReleaseRegistryErrorCode::Limit);

    registry["frontend_bundles"]
        .as_array_mut()
        .and_then(|bundles| bundles.last_mut())
        .expect("above-limit final bundle exists")["unexpected"] = Value::Bool(true);
    let mut transport =
        serde_json::to_vec(&registry).expect("mixed shape/count registry serializes");
    transport.push(b'\n');
    let shape = validate_release_registry(&transport)
        .expect_err("a later bundle shape failure precedes the scalar count limit");
    assert_eq!(shape.phase(), ReleaseValidationPhase::Shape);
    assert_eq!(shape.code(), ReleaseRegistryErrorCode::Invalid);
}

#[test]
fn release_bundle_selection_vectors_are_exact_and_have_no_default() {
    let vectors = load_vectors();
    let valid = field(field(&vectors, "fixtures"), "valid_registry");
    let registry = validate_release_registry(&canonical_transport(valid))
        .expect("synthetic registry must validate");
    let cases = array(field(&vectors, "selection_cases"));
    assert_eq!(cases.len(), 6);
    let mut seen = BTreeSet::new();

    for case in cases {
        let id = text(field(case, "id"));
        assert!(seen.insert(id));
        let expect = field(case, "expect");
        if case.get("input").is_some() {
            assert_eq!(id, "selection.candidate_schema_forbidden");
            assert_eq!(text(field(expect, "code")), "FRONTEND_BUNDLE_UNKNOWN");
            continue;
        }

        let request_value = field(case, "request").clone();
        let request: VectorSelectionRequest =
            serde_json::from_value(request_value).expect("closed selection request shape");
        let request = ReleaseSelectionRequest {
            registry_id: request.registry_id,
            registry_sha256: request.registry_sha256,
            source_language: request.source_language,
            semantic_profile: request.semantic_profile,
            target_id: request.target_id,
            frontend_bundle_id: request.frontend_bundle_id,
            toolchain_bundle_id: request.toolchain_bundle_id,
        };
        match text(field(expect, "outcome")) {
            "accept" => {
                let selected = registry.resolve(&request).expect("selection must resolve");
                assert_eq!(
                    selected.release_tuple.pointer_width,
                    integer(field(expect, "selected_pointer_width"))
                );
                assert_eq!(
                    selected.release_tuple.limit_profile_id,
                    text(field(expect, "selected_limit_profile_id"))
                );
                assert_eq!(
                    registry
                        .frontend_bundle(&selected.frontend.bundle_id)
                        .expect("frontend lookup is exact"),
                    selected.frontend
                );
                assert_eq!(
                    registry
                        .toolchain_bundle(&selected.toolchain.bundle_id)
                        .expect("toolchain lookup is exact"),
                    selected.toolchain
                );
            }
            "reject" => {
                let error = registry
                    .resolve(&request)
                    .expect_err("selection must reject");
                assert_eq!(error.code(), text(field(expect, "code")), "{id}");
            }
            outcome => panic!("unknown selection outcome {outcome}"),
        }
    }

    assert_eq!(
        registry
            .resolve(&ReleaseSelectionRequest {
                registry_id: registry.registry().id.clone(),
                registry_sha256: registry.registry().registry_sha256.clone(),
                source_language: "go".to_owned(),
                semantic_profile: "mpk.go.fixed.v0".to_owned(),
                target_id: "linux/amd64".to_owned(),
                frontend_bundle_id: None,
                toolchain_bundle_id: None,
            })
            .expect_err("missing bundle IDs never select defaults"),
        ReleaseSelectionError::BundleUnknown
    );
}

#[test]
fn release_bundle_limit_vectors_match_all_boundaries() {
    let vectors = load_vectors();
    let cases = array(field(&vectors, "limit_cases"));
    assert_eq!(cases.len(), 42);
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = text(field(case, "id"));
        assert!(seen.insert(id));
        let construction = field(case, "construction");
        let result = validate_release_limit(
            text(field(construction, "kind")),
            integer(field(construction, "value")) as u64,
        );
        let expect = field(case, "expect");
        match text(field(expect, "outcome")) {
            "accept_boundary" => result.expect("boundary must accept"),
            "reject" => {
                let error = result.expect_err("above-boundary value must reject");
                assert_eq!(error.phase().as_str(), text(field(expect, "phase")), "{id}");
                assert_eq!(error.code().as_str(), text(field(expect, "code")), "{id}");
            }
            outcome => panic!("unknown limit outcome {outcome}"),
        }
    }
}

#[test]
fn release_bundle_hash_vectors_match_every_payload_and_domain() {
    let vectors = load_vectors();
    let fixtures = field(&vectors, "fixtures");
    let cases = array(field(&vectors, "hash_cases"));
    assert_eq!(cases.len(), 19);
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = text(field(case, "id"));
        assert!(seen.insert(id));
        let domain = domain(text(field(case, "domain")));
        let payload = if case.get("fixture").is_none() {
            strict(text(field(case, "canonical_payload")).as_bytes())
        } else {
            let mut fixture = field(fixtures, text(field(case, "fixture"))).clone();
            if let Some(pointer) = case.get("pointer") {
                fixture
                    .pointer(text(pointer))
                    .unwrap_or_else(|| panic!("missing hash pointer for {id}"))
                    .clone()
            } else {
                remove_pointer(&mut fixture, text(field(case, "remove_pointer")));
                fixture
            }
            .pipe(|value| strict_value(&value))
        };
        let canonical = canonical_json_bytes(&payload).expect("hash payload canonicalizes");
        assert_eq!(
            canonical.len() as i64,
            integer(field(case, "canonical_payload_utf8_length")),
            "{id}"
        );
        assert_eq!(
            canonical.len() as i64 + domain.as_str().len() as i64 + 1,
            integer(field(case, "expected_preimage_length")),
            "{id}"
        );
        if let Some(expected) = case.get("canonical_payload") {
            assert_eq!(canonical, text(expected).as_bytes(), "{id}");
        }
        if let Some(expected) = case.get("canonical_payload_sha256") {
            assert_eq!(
                sha256_raw_file_bytes(&canonical).to_hex(),
                text(expected),
                "{id}"
            );
        }
        let digest = if domain == BUNDLE_CONTENT_HASH_DOMAIN {
            hash_canonical_inventory(domain, &payload)
                .expect("content payload hashes")
                .to_hex()
        } else {
            hash_canonical_json(domain, &payload)
                .expect("registry payload hashes")
                .to_hex()
        };
        assert_eq!(digest, text(field(case, "expected_sha256")), "{id}");
        if let Some(different_from) = case.get("different_from") {
            assert_ne!(digest, text(different_from), "{id}");
        }
    }
}

#[test]
fn tracked_successor_release_registry_is_valid_and_build_inputs_are_derived() {
    let root = repository_root();
    let semantic_bytes = fs::read(root.join("release/bundles/semantic-profile-registry.json"))
        .expect("read tracked semantic registry");
    let semantic = validate_semantic_profile_registry(&semantic_bytes, RegistryRevision::Revision2)
        .expect("tracked revision-2 semantic registry validates");
    let bytes = fs::read(root.join("release/bundles/bundle-registry.json"))
        .expect("read tracked bundle registry");
    let validated = validate_successor_release_registry(&bytes, &semantic)
        .expect("tracked successor registry validates");
    assert_eq!(validated.registry().id, "mpk.release.registry.v1");
    assert_eq!(
        validated.registry().profile_registry.registry_sha256,
        semantic.identity().registry_sha256()
    );
    assert_eq!(validated.registry().frontend_bundles.len(), 3);
    assert_eq!(validated.registry().toolchain_bundles.len(), 3);
    assert_eq!(validated.registry().tuples.len(), 4);
    assert_eq!(validated.registry().native_runtime_layout_profiles.len(), 1);
    let frontend = validated
        .registry()
        .frontend_bundles
        .iter()
        .find(|bundle| bundle.bundle_id == "frontend.go.go2vir.candidate.v1")
        .expect("registered Go frontend");
    assert!(frontend.subordinate_binaries.is_empty());
    assert!(matches!(frontend.main.runtime, ExecutableRuntime::Static));
}

#[test]
fn release_bundle_portable_paths_and_ambiguous_selection_keys_reject() {
    let vectors = load_vectors();
    let mut invalid_path = field(field(&vectors, "fixtures"), "valid_registry").clone();
    *invalid_path
        .pointer_mut("/frontend_bundles/0/inventory/files/0/path")
        .expect("fixture path") = Value::String("aux.txt".to_owned());
    rehash_content(&mut invalid_path, "/frontend_bundles/0");
    rehash_registry(&mut invalid_path);
    let error = validate_release_registry(&canonical_transport(&invalid_path))
        .expect_err("Windows device path must reject");
    assert_eq!(error.phase(), ReleaseValidationPhase::Scalar);

    let mut source_only_path = field(field(&vectors, "fixtures"), "valid_registry").clone();
    *source_only_path
        .pointer_mut("/frontend_bundles/0/inventory/files/0/path")
        .expect("fixture path") = Value::String("build-inputs/descriptor.json".to_owned());
    rehash_content(&mut source_only_path, "/frontend_bundles/0");
    rehash_registry(&mut source_only_path);
    let error = validate_release_registry(&canonical_transport(&source_only_path))
        .expect_err("source-only release material must not enter an inventory");
    assert_eq!(error.phase(), ReleaseValidationPhase::Scalar);

    let mut ambiguous = field(field(&vectors, "fixtures"), "valid_registry").clone();
    let mut second = ambiguous
        .pointer("/tuples/0")
        .expect("tuple fixture")
        .clone();
    second["pointer_width"] = Value::from(32);
    ambiguous["tuples"] = Value::Array(vec![second, ambiguous["tuples"][0].clone()]);
    rehash_registry(&mut ambiguous);
    let error = validate_release_registry(&canonical_transport(&ambiguous))
        .expect_err("selection keys differing only in descriptor output must reject");
    assert_eq!(error.phase(), ReleaseValidationPhase::Order);
}

#[test]
fn release_bundle_reviewed_scalar_runtime_and_content_edges_are_enforced() {
    let vectors = load_vectors();
    let valid = field(field(&vectors, "fixtures"), "valid_registry");

    let mut negative_size = valid.clone();
    negative_size["frontend_bundles"][0]["inventory"]["files"][0]["size_bytes"] = Value::from(-1);
    rehash_registry(&mut negative_size);
    let error = validate_release_registry(&canonical_transport(&negative_size))
        .expect_err("negative size must reach and fail scalar validation");
    assert_eq!(error.phase(), ReleaseValidationPhase::Scalar);

    let mut overlapping_mount = valid.clone();
    overlapping_mount["native_runtime_layout_profiles"][0]["interpreter_mounts"][0]
        ["sandbox_path"] = Value::String("/lib/x86_64-linux-gnu".to_owned());
    rehash_registry(&mut overlapping_mount);
    let error = validate_release_registry(&canonical_transport(&overlapping_mount))
        .expect_err("an interpreter file cannot equal a library mount directory");
    assert_eq!(error.phase(), ReleaseValidationPhase::Invariant);

    let mut executable_notice = valid.clone();
    executable_notice["toolchain_bundles"][0]["components"][3]["inventory"]["files"][0]
        ["executable"] = Value::Bool(true);
    executable_notice["toolchain_bundles"][0]["inventory"]["files"][0]["executable"] =
        Value::Bool(true);
    rehash_content(&mut executable_notice, "/toolchain_bundles/0/components/3");
    rehash_content(&mut executable_notice, "/toolchain_bundles/0");
    rehash_registry(&mut executable_notice);
    let error = validate_release_registry(&canonical_transport(&executable_notice))
        .expect_err("executable content outside native runtime must reject");
    assert_eq!(error.phase(), ReleaseValidationPhase::Invariant);

    let mut shared_toolchain = valid.clone();
    shared_toolchain["toolchain_bundles"][0]["components"][0]["runtime"] =
        serde_json::json!({"kind":"static"});
    let mut static_frontend = shared_toolchain["frontend_bundles"][0].clone();
    static_frontend["bundle_id"] = Value::String("frontend.go.static.v0".to_owned());
    static_frontend["inventory"]["scope"]["bundle_id"] =
        Value::String("frontend.go.static.v0".to_owned());
    static_frontend["main"]["runtime"] = serde_json::json!({"kind":"static"});
    shared_toolchain["frontend_bundles"] = Value::Array(vec![
        static_frontend,
        shared_toolchain["frontend_bundles"][0].clone(),
    ]);
    rehash_content(&mut shared_toolchain, "/frontend_bundles/0");
    let mut static_tuple = shared_toolchain["tuples"][0].clone();
    static_tuple["frontend_bundle_id"] = Value::String("frontend.go.static.v0".to_owned());
    shared_toolchain["tuples"] =
        Value::Array(vec![static_tuple, shared_toolchain["tuples"][0].clone()]);
    rehash_registry(&mut shared_toolchain);
    validate_release_registry(&canonical_transport(&shared_toolchain)).expect(
        "a runtime component remains referenced when any frontend paired to the toolchain is dynamic",
    );

    let mut read_only_library = valid.clone();
    read_only_library["toolchain_bundles"][0]["components"][2]["inventory"]["files"][0]
        ["executable"] = Value::Bool(false);
    read_only_library["toolchain_bundles"][0]["inventory"]["files"][2]["executable"] =
        Value::Bool(false);
    rehash_content(&mut read_only_library, "/toolchain_bundles/0/components/2");
    rehash_content(&mut read_only_library, "/toolchain_bundles/0");
    rehash_registry(&mut read_only_library);
    validate_release_registry(&canonical_transport(&read_only_library))
        .expect("a digest-matched shared library may use the non-executable file class");
}

fn assert_registry_outcome(id: &str, input: &[u8], expect: &Value) {
    match text(field(expect, "outcome")) {
        "accept" => {
            validate_release_registry(input)
                .unwrap_or_else(|error| panic!("{id} must accept: {error}"));
        }
        "reject" => {
            let error = match validate_release_registry(input) {
                Ok(_) => panic!("{id} must reject"),
                Err(error) => error,
            };
            assert_eq!(error.phase().as_str(), text(field(expect, "phase")), "{id}");
            assert_eq!(error.code().as_str(), text(field(expect, "code")), "{id}");
        }
        outcome => panic!("unknown registry outcome {outcome}"),
    }
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let mut bytes = canonical_json_bytes(&strict_value(value)).expect("fixture canonicalizes");
    bytes.push(b'\n');
    bytes
}

fn strict_value(value: &Value) -> StrictJsonValue {
    let bytes = serde_json::to_vec(value).expect("serialize vector value");
    strict(&bytes)
}

fn strict(bytes: &[u8]) -> StrictJsonValue {
    parse_strict_json(bytes, TEST_LIMITS).expect("strict test JSON parses")
}

fn rehash_content(root: &mut Value, pointer: &str) {
    let target = root
        .pointer(pointer)
        .unwrap_or_else(|| panic!("missing content pointer {pointer}"));
    let (inventory, hash_field) = if target.get("bundle_sha256").is_some() {
        (field(target, "inventory").clone(), "bundle_sha256")
    } else if target.get("distribution_sha256").is_some() {
        (field(target, "inventory").clone(), "distribution_sha256")
    } else {
        (field(target, "inventory").clone(), "content_sha256")
    };
    let digest = hash_canonical_inventory(BUNDLE_CONTENT_HASH_DOMAIN, &strict_value(&inventory))
        .expect("content hash recomputes")
        .to_hex();
    target_mut(root, pointer)[hash_field] = Value::String(digest);
}

fn rehash_registry(root: &mut Value) {
    let mut preimage = root.clone();
    object_mut(&mut preimage)
        .remove("registry_sha256")
        .expect("registry hash field exists");
    let digest = hash_canonical_json(BUNDLE_REGISTRY_HASH_DOMAIN, &strict_value(&preimage))
        .expect("registry hash recomputes")
        .to_hex();
    root["registry_sha256"] = Value::String(digest);
}

fn apply_operations(target: &mut Value, operations: &Value) {
    for operation in array(operations) {
        let kind = text(field(operation, "op"));
        let path = text(field(operation, "path"));
        match kind {
            "add" => add_pointer(target, path, field(operation, "value").clone(), false),
            "replace" => add_pointer(target, path, field(operation, "value").clone(), true),
            "remove" => {
                remove_pointer(target, path);
            }
            "copy" => {
                let value = target
                    .pointer(text(field(operation, "from")))
                    .expect("copy source exists")
                    .clone();
                add_pointer(target, path, value, false);
            }
            "move" => {
                let value = remove_pointer(target, text(field(operation, "from")));
                add_pointer(target, path, value, false);
            }
            _ => panic!("unsupported vector patch operation {kind}"),
        }
    }
}

fn add_pointer(target: &mut Value, path: &str, value: Value, replace: bool) {
    let mut tokens = pointer_tokens(path);
    let last = tokens.pop().expect("patch path has a final token");
    let parent_path = pointer_path(&tokens);
    let parent = if parent_path.is_empty() {
        target
    } else {
        target
            .pointer_mut(&parent_path)
            .expect("patch parent exists")
    };
    match parent {
        Value::Object(entries) => {
            if replace {
                assert!(entries.contains_key(&last), "replace field exists");
            }
            entries.insert(last, value);
        }
        Value::Array(values) => {
            let index = if last == "-" {
                values.len()
            } else {
                last.parse::<usize>().expect("array patch index")
            };
            if replace {
                values[index] = value;
            } else {
                values.insert(index, value);
            }
        }
        _ => panic!("patch parent must be a container"),
    }
}

fn remove_pointer(target: &mut Value, path: &str) -> Value {
    let mut tokens = pointer_tokens(path);
    let last = tokens.pop().expect("remove path has a final token");
    let parent_path = pointer_path(&tokens);
    let parent = if parent_path.is_empty() {
        target
    } else {
        target
            .pointer_mut(&parent_path)
            .expect("remove parent exists")
    };
    match parent {
        Value::Object(entries) => entries.remove(&last).expect("remove field exists"),
        Value::Array(values) => values.remove(last.parse::<usize>().expect("remove index")),
        _ => panic!("remove parent must be a container"),
    }
}

fn target_mut<'a>(target: &'a mut Value, pointer: &str) -> &'a mut Value {
    target
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("missing mutable pointer {pointer}"))
}

fn pointer_tokens(path: &str) -> Vec<String> {
    assert!(path.starts_with('/'));
    path[1..]
        .split('/')
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn pointer_path(tokens: &[String]) -> String {
    tokens.iter().fold(String::new(), |mut path, token| {
        path.push('/');
        path.push_str(&token.replace('~', "~0").replace('/', "~1"));
        path
    })
}

fn domain(value: &str) -> HashDomain {
    match value {
        "MPK-BUNDLE-REGISTRY-0.1" => BUNDLE_REGISTRY_HASH_DOMAIN,
        "MPK-BUNDLE-CONTENT-0.1" => BUNDLE_CONTENT_HASH_DOMAIN,
        _ => panic!("unknown vector hash domain {value}"),
    }
}

fn load_vectors() -> Value {
    let bytes = fs::read(repository_root().join("develop/specs/vectors/release-bundles-v0.json"))
        .expect("read release bundle vectors");
    strict(&bytes);
    let vectors: Value = serde_json::from_slice(&bytes).expect("parse vector container");
    assert_eq!(
        text(field(&vectors, "schema")),
        "mpk.release.bundle_conformance.v0"
    );
    assert_eq!(
        array(field(&vectors, "owner_tests")),
        [
            Value::String("crates/mpk-vc/tests/release_bundle.rs".to_owned()),
            Value::String("crates/mpk-cli/tests/frontend_runner.rs".to_owned()),
        ]
    );
    vectors
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing field {name:?}"))
}

fn field_value<'a>(value: &'a Map<String, Value>, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing object field {name:?}"))
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("value is an array")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("value is an object")
}

fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("value is an object")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("value is a string")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().expect("value is an integer")
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = object(value).keys().map(String::as_str).collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected, "closed object keys differ");
}

fn assert_allowed_keys(value: &Value, allowed: &[&str]) {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    for key in object(value).keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unknown closed-object key {key}"
        );
    }
}

fn assert_operations_closed(operations: &Value) {
    for operation in array(operations) {
        match text(field(operation, "op")) {
            "add" | "replace" => assert_exact_keys(operation, &["op", "path", "value"]),
            "remove" => assert_exact_keys(operation, &["op", "path"]),
            "copy" | "move" => assert_exact_keys(operation, &["op", "from", "path"]),
            kind => panic!("unknown patch operation {kind}"),
        }
    }
}

fn assert_registry_expect_closed(expect: &Value, selection: bool) {
    match text(field(expect, "outcome")) {
        "accept" if selection => assert_exact_keys(
            expect,
            &[
                "outcome",
                "selected_pointer_width",
                "selected_limit_profile_id",
            ],
        ),
        "accept" | "accept_boundary" => assert_exact_keys(expect, &["outcome"]),
        "reject" => assert_exact_keys(expect, &["outcome", "phase", "code"]),
        outcome => panic!("unknown model expectation outcome {outcome}"),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorSelectionRequest {
    registry_id: String,
    registry_sha256: String,
    source_language: String,
    semantic_profile: String,
    target_id: String,
    frontend_bundle_id: Option<String>,
    toolchain_bundle_id: Option<String>,
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}
