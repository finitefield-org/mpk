use mpk_cli::frontend_protocol::{
    validate_frontend_process, FrontendProcessFacts, FrontendProtocolCode, FrontendProtocolRequest,
    FRONTEND_STDERR_BYTES_MAX, FRONTEND_STDOUT_BYTES_MAX,
};
use mpk_cli::successor_release_bundle::validate_successor_release_registry;
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, validate_release_registry, CapturedInput, InputKind,
    ReleaseSelectionRequest, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TEST_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(68 * 1024 * 1024, 68 * 1024 * 1024, 256, 2 * 1024 * 1024);

#[test]
fn frontend_protocol_vectors_are_closed_unique_and_executed() {
    let root = repository_root();
    let protocol = read_json(&root.join("develop/specs/vectors/frontend-protocol-v0.json"));
    assert_exact_keys(
        &protocol,
        &[
            "schema",
            "spec_schema",
            "dependencies",
            "owner_tests",
            "status_cases",
            "transport_cases",
            "identity_cases",
            "limit_cases",
        ],
    );
    assert_eq!(protocol["schema"], "mpk.frontend.protocol.conformance.v0");
    assert_eq!(
        protocol["owner_tests"][0],
        "crates/mpk-cli/tests/frontend_runner.rs"
    );

    let success = success_envelope(&root, &protocol);
    let registry_vectors = read_json(&root.join("develop/specs/vectors/release-bundles-v0.json"));
    let registry = validate_release_registry(&canonical_transport(
        &registry_vectors["fixtures"]["valid_registry"],
    ))
    .expect("synthetic release registry validates");
    let captured_storage = captured_fixture_inputs(&root);
    let captured = captured_refs(&captured_storage);
    let mut values = BTreeMap::<String, Value>::new();
    let mut requests = BTreeMap::<String, (Value, Value, String, String)>::new();
    let mut executed = BTreeSet::new();

    for case in protocol["status_cases"].as_array().expect("status cases") {
        assert_case_record(case);
        let id = text(&case["id"]);
        if id == "status.valid_cli_configuration_error" {
            assert_eq!(case["process"]["exit"], 2);
            assert_eq!(case["expect"]["json"], false);
            executed.insert(id.to_owned());
            continue;
        }
        let value = construct_case_value(case, &values, &success);
        let expected_identity = identity_for_status_case(case, &value, &values);
        let result =
            execute_protocol_case(case, &value, &expected_identity, Some(&registry), &captured);
        assert_protocol_expectation(case, result);
        values.insert(id.to_owned(), value);
        requests.insert(id.to_owned(), expected_identity);
        executed.insert(id.to_owned());
    }

    for group in ["transport_cases", "identity_cases"] {
        for case in protocol[group].as_array().expect("protocol case array") {
            assert_case_record(case);
            let id = text(&case["id"]);
            let value = if case.get("input_from").is_some() || case.get("construction").is_some() {
                Some(construct_case_value(case, &values, &success))
            } else {
                None
            };
            let identity = if group == "identity_cases" {
                let base = text(&case["construction"]["base"]);
                requests[base].clone()
            } else if let Some(base) = case.get("input_from").and_then(Value::as_str) {
                requests[base].clone()
            } else {
                go_identity(&success)
            };
            let result = execute_protocol_case_optional(
                case,
                value.as_ref(),
                &identity,
                Some(&registry),
                &captured,
            );
            assert_protocol_expectation(case, result);
            if let Some(value) = value {
                values.insert(id.to_owned(), value);
                requests.insert(id.to_owned(), identity);
            }
            executed.insert(id.to_owned());
        }
    }

    for case in protocol["limit_cases"].as_array().expect("limit cases") {
        assert_case_record(case);
        let id = text(&case["id"]);
        match id {
            "limit.stderr_bytes_at" | "limit.stderr_bytes_above" | "limit.path_leak_in_message" => {
                if id == "limit.stderr_bytes_at" {
                    assert_eq!(
                        case["process"]["stderr"]["count"],
                        FRONTEND_STDERR_BYTES_MAX
                    );
                }
                let value = construct_case_value(case, &values, &success);
                let base = case
                    .get("input_from")
                    .and_then(Value::as_str)
                    .or_else(|| case.pointer("/construction/base").and_then(Value::as_str))
                    .expect("limit base");
                let result = execute_protocol_case(
                    case,
                    &value,
                    &requests[base],
                    Some(&registry),
                    &captured,
                );
                assert_protocol_expectation(case, result);
            }
            "limit.stdout_bytes_at_continues_parsing" => {
                assert_eq!(case["construction"]["count"], FRONTEND_STDOUT_BYTES_MAX);
                assert_eq!(case["expect"]["code"], "FRONTEND_PROTOCOL_MALFORMED");
            }
            "limit.stdout_bytes_above" => {
                assert_eq!(case["construction"]["count"], FRONTEND_STDOUT_BYTES_MAX + 1);
                assert_eq!(case["expect"]["code"], "FRONTEND_PROTOCOL_LIMIT");
            }
            _ => assert_normalization_construction(case),
        }
        executed.insert(id.to_owned());
    }

    let expected_count: usize = [
        "status_cases",
        "transport_cases",
        "identity_cases",
        "limit_cases",
    ]
    .iter()
    .map(|group| protocol[*group].as_array().expect("case array").len())
    .sum();
    assert_eq!(
        executed.len(),
        expected_count,
        "a protocol case was skipped or duplicated"
    );
}

#[test]
fn release_installation_selection_and_assembler_vectors_are_all_owned() {
    let root = repository_root();
    let vectors = read_json(&root.join("develop/specs/vectors/release-bundles-v0.json"));
    assert_eq!(
        vectors["owner_tests"],
        json!([
            "crates/mpk-vc/tests/release_bundle.rs",
            "crates/mpk-cli/tests/frontend_runner.rs"
        ])
    );
    let registry =
        validate_release_registry(&canonical_transport(&vectors["fixtures"]["valid_registry"]))
            .expect("selection fixture registry validates");
    let mut ids = BTreeSet::new();
    let mut visited = 0usize;
    for group in [
        "registry_cases",
        "inventory_cases",
        "installation_cases",
        "selection_cases",
        "assembler_cases",
        "limit_cases",
        "hash_cases",
    ] {
        for case in vectors[group].as_array().expect("release case array") {
            let id = text(&case["id"]);
            assert!(ids.insert(id.to_owned()), "duplicate release case {id}");
            visited += 1;
            match group {
                "installation_cases" => validate_installation_case_shape(case),
                "selection_cases" => validate_selection_case(case, &registry),
                "assembler_cases" => validate_assembler_case_shape(case),
                _ => assert!(case.is_object(), "{id} must be a closed case object"),
            }
        }
    }
    assert_eq!(visited, ids.len());

    let semantic_bytes = fs::read(root.join("release/bundles/semantic-profile-registry.json"))
        .expect("read semantic registry");
    let semantic = validate_semantic_profile_registry(&semantic_bytes, RegistryRevision::Revision3)
        .expect("revision-3 semantic registry validates");
    let tracked_bytes =
        fs::read(root.join("release/bundles/bundle-registry.json")).expect("read tracked registry");
    let tracked = validate_successor_release_registry(&tracked_bytes, &semantic)
        .expect("tracked successor registry validates");
    assert_eq!(tracked.registry().tuples.len(), 5);
    assert_eq!(tracked.registry().frontend_bundles.len(), 4);
    assert_eq!(tracked.registry().toolchain_bundles.len(), 4);
    let tracked_value: Value = serde_json::from_slice(&tracked_bytes).expect("tracked JSON");
    assert_eq!(
        tracked_value["tuples"]
            .as_array()
            .expect("successor tuples")
            .iter()
            .map(|tuple| (
                tuple["semantic_context"]["source_language"]
                    .as_str()
                    .expect("source language"),
                tuple["semantic_context"]["semantic_parameters"]["value"]["target_id"]
                    .as_str()
                    .expect("target ID")
            ))
            .collect::<Vec<_>>(),
        [
            ("go", "linux/amd64"),
            ("rust", "i686-unknown-linux-gnu"),
            ("rust", "x86_64-unknown-linux-gnu"),
            ("csharp", "linux-x64"),
            ("java", "linux-x64"),
        ]
    );
    let rust_frontend = tracked_value["frontend_bundles"]
        .as_array()
        .expect("frontend bundles")
        .iter()
        .find(|bundle| bundle["bundle_id"] == "frontend.rust.rust2vir.candidate.v2")
        .expect("registered Rust frontend");
    assert_eq!(
        rust_frontend["subordinate_binaries"]
            .as_array()
            .expect("subordinate binaries")
            .len(),
        1
    );
    assert_eq!(
        rust_frontend["subordinate_binaries"][0]["name"],
        "rust2vir-driver"
    );
}

#[test]
fn assembler_rejects_unknown_argv_without_writes() {
    let root = repository_root();
    let script = root.join("scripts/build-release-bundles.sh");
    for (args, exit, stderr) in [
        (vec!["--unsupported", "go"], 64, "BUNDLE_ASSEMBLER_USAGE\n"),
        (vec![], 64, "BUNDLE_ASSEMBLER_USAGE\n"),
    ] {
        let output = Command::new(&script)
            .args(args)
            .output()
            .expect("run assembler argument boundary");
        assert_eq!(output.status.code(), Some(exit));
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8(output.stderr).unwrap(), stderr);
    }
}

#[test]
fn stable_root_workspace_explicitly_excludes_standalone_rust_packages() {
    let root = repository_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read root manifest");
    assert!(
        manifest.contains("exclude = [\"rust-tools/rust2vir\", \"examples/rust-payment-policy\"]"),
        "root workspace must explicitly exclude the pinned nightly frontend and standalone example"
    );

    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(&root)
        .output()
        .expect("read stable workspace metadata");
    assert!(metadata.status.success());
    assert!(metadata.stderr.is_empty());
    let value: Value = serde_json::from_slice(&metadata.stdout).expect("parse workspace metadata");
    let package_manifests = value["packages"]
        .as_array()
        .expect("metadata packages")
        .iter()
        .filter_map(|package| package["manifest_path"].as_str())
        .collect::<BTreeSet<_>>();
    for (relative_path, description) in [
        ("rust-tools/rust2vir/Cargo.toml", "pinned nightly frontend"),
        (
            "examples/rust-payment-policy/Cargo.toml",
            "standalone payment-policy example",
        ),
    ] {
        let excluded_manifest = root.join(relative_path);
        assert!(
            !package_manifests.contains(excluded_manifest.to_str().expect("UTF-8 manifest path")),
            "stable workspace metadata selected the {description}"
        );
    }
}

#[test]
#[cfg(target_os = "linux")]
fn rust_build_input_vectors_are_owned_by_the_internal_conformance_harness() {
    let root = repository_root();
    let output = Command::new("/usr/bin/env")
        .args([
            "-i",
            "PATH=/usr/bin:/bin",
            "/usr/bin/python3",
            "scripts/rust_build_inputs.py",
            "self-test",
        ])
        .current_dir(root)
        .output()
        .expect("run Rust build-input conformance harness");
    assert!(
        output.status.success(),
        "Rust build-input conformance failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn released_cli_help_exposes_only_successor_semantic_selection() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("--help")
        .output()
        .expect("run released help");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let help = String::from_utf8(output.stdout).expect("help is UTF-8");
    for expected in ["--semantic-context", "--selection"] {
        assert!(help.contains(expected), "released help omitted {expected}");
    }
    for private in [
        "go2vir",
        "rust2vir",
        "--frontend-bundle",
        "--toolchain-bundle",
        "--release-registry",
        "__mpk_frontend_",
    ] {
        assert!(!help.contains(private), "released help exposed {private}");
    }
}

fn success_envelope(root: &Path, protocol: &Value) -> Value {
    let construction = &protocol["status_cases"][0]["construction"];
    let vir_vectors = read_json(&root.join("develop/specs/vectors/vir-v0.json"));
    let map_vectors = read_json(&root.join("develop/specs/vectors/source-map-v0.json"));
    let manifest_vectors = read_json(&root.join("develop/specs/vectors/source-manifest-v0.json"));
    let vir = find_case(
        &vir_vectors["module_cases"],
        text(&construction["vir_case"]),
    )["input"]
        .clone();
    let source_map = find_case(
        &map_vectors["map_cases"],
        text(&construction["source_map_case"]),
    )["input"]
        .clone();
    let manifest = find_case(
        &manifest_vectors["manifest_cases"],
        text(&construction["source_manifest_case"]),
    )["input"]
        .clone();
    json!({
        "schema":"mpk.frontend.cli.v0","status":"ir-lowered","phase":"emission",
        "source_language":construction["source_language"],
        "semantic_profile":construction["semantic_profile"],
        "semantic_parameters":construction["semantic_parameters"],
        "selection":construction["selection"],
        "ir":{"schema":"mpk.vir.v0","sha256":vir["vir_hash"],"value":vir},
        "source_manifest":manifest,"source_map":source_map,
        "rejected_features":[],"diagnostics":[]
    })
}

fn construct_case_value(case: &Value, values: &BTreeMap<String, Value>, success: &Value) -> Value {
    if let Some(input) = case.get("input") {
        return input.clone();
    }
    if let Some(base) = case.get("input_from").and_then(Value::as_str) {
        return values[base].clone();
    }
    let construction = &case["construction"];
    if construction["fixture"] == "canonical_from_dependencies" {
        return success.clone();
    }
    if let Some(base) = construction.get("base").and_then(Value::as_str) {
        let mut value = values[base].clone();
        for patch in construction["patches"].as_array().unwrap_or(&Vec::new()) {
            apply_patch(&mut value, patch);
        }
        return value;
    }
    panic!("unsupported case construction: {}", case["id"])
}

fn identity_for_status_case(
    case: &Value,
    value: &Value,
    values: &BTreeMap<String, Value>,
) -> (Value, Value, String, String) {
    if let Some(base) = case.pointer("/construction/base").and_then(Value::as_str) {
        return go_identity(&values[base]);
    }
    go_identity(value)
}

fn go_identity(value: &Value) -> (Value, Value, String, String) {
    (
        value["semantic_parameters"].clone(),
        value["selection"].clone(),
        text(&value["source_language"]).to_owned(),
        text(&value["semantic_profile"]).to_owned(),
    )
}

fn execute_protocol_case(
    case: &Value,
    value: &Value,
    identity: &(Value, Value, String, String),
    registry: Option<&mpk_vc::ValidatedReleaseRegistry>,
    captured: &[CapturedInput<'_>],
) -> Result<String, FrontendProtocolCode> {
    execute_protocol_case_optional(case, Some(value), identity, registry, captured)
}

fn execute_protocol_case_optional(
    case: &Value,
    value: Option<&Value>,
    identity: &(Value, Value, String, String),
    registry: Option<&mpk_vc::ValidatedReleaseRegistry>,
    captured: &[CapturedInput<'_>],
) -> Result<String, FrontendProtocolCode> {
    let process = &case["process"];
    let stdout = process_stdout(process, value);
    let stderr_bytes = process
        .pointer("/stderr/count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let signaled = process.get("signal").is_some();
    validate_frontend_process(
        FrontendProtocolRequest {
            source_language: &identity.2,
            semantic_profile: &identity.3,
            semantic_parameters: &identity.0,
            selection: &identity.1,
            release_registry: registry,
            captured_inputs: captured,
        },
        FrontendProcessFacts {
            exit_code: process
                .get("exit")
                .and_then(Value::as_i64)
                .map(|value| value as i32),
            signaled,
            stdout: &stdout,
            stderr_observed_bytes: stderr_bytes,
        },
    )
    .map(|accepted| accepted.status)
    .map_err(|error| error.code())
}

fn process_stdout(process: &Value, value: Option<&Value>) -> Vec<u8> {
    if let Some(literal) = process.get("stdout_utf8").and_then(Value::as_str) {
        return literal.as_bytes().to_vec();
    }
    if process.get("stdout_base64").is_some() {
        return Vec::new();
    }
    let Some(value) = value else {
        return Vec::new();
    };
    let canonical = canonical_transport(value);
    match process.get("stdout").and_then(Value::as_str) {
        Some("canonical") => canonical,
        Some("pretty") => {
            let mut bytes = serde_json::to_vec_pretty(value).unwrap();
            bytes.push(b'\n');
            bytes
        }
        Some("missing_lf") => canonical[..canonical.len() - 1].to_vec(),
        Some("extra_lf") => [canonical.as_slice(), b"\n"].concat(),
        Some("bom") => [&[0xef, 0xbb, 0xbf], canonical.as_slice()].concat(),
        Some("second_value") => {
            let mut bytes = canonical[..canonical.len() - 1].to_vec();
            bytes.extend_from_slice(&canonical_transport(&process["second_value"]));
            bytes
        }
        Some("truncate") => {
            let count = process["truncate_bytes"].as_u64().unwrap() as usize;
            canonical[..canonical.len() - count].to_vec()
        }
        None => Vec::new(),
        other => panic!("unsupported stdout construction {other:?}"),
    }
}

fn assert_protocol_expectation(case: &Value, result: Result<String, FrontendProtocolCode>) {
    let id = text(&case["id"]);
    match text(&case["expect"]["outcome"]) {
        "accept" => {
            let status = result.unwrap_or_else(|code| panic!("{id} rejected as {}", code.as_str()));
            if let Some(expected) = case["expect"].get("status").and_then(Value::as_str) {
                assert_eq!(status, expected, "{id}");
            }
        }
        "frontend-error" => assert_eq!(
            result
                .expect_err(&format!("{id} unexpectedly accepted"))
                .as_str(),
            text(&case["expect"]["code"]),
            "{id}"
        ),
        outcome => panic!("unexpected protocol outcome {outcome} for {id}"),
    }
}

fn validate_selection_case(case: &Value, registry: &mpk_vc::ValidatedReleaseRegistry) {
    let id = text(&case["id"]);
    if id == "selection.candidate_schema_forbidden" {
        assert_eq!(case["input"]["schema"], "mpk.release.bundle_candidate.v0");
        return;
    }
    let value = &case["request"];
    let request = ReleaseSelectionRequest {
        registry_id: text(&value["registry_id"]).to_owned(),
        registry_sha256: text(&value["registry_sha256"]).to_owned(),
        source_language: text(&value["source_language"]).to_owned(),
        semantic_profile: text(&value["semantic_profile"]).to_owned(),
        target_id: text(&value["target_id"]).to_owned(),
        frontend_bundle_id: value
            .get("frontend_bundle_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        toolchain_bundle_id: value
            .get("toolchain_bundle_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
    };
    let result = registry.resolve(&request);
    match text(&case["expect"]["outcome"]) {
        "accept" => {
            let selected = result.unwrap_or_else(|error| panic!("{id}: {error}"));
            assert_eq!(
                selected.release_tuple.pointer_width,
                case["expect"]["selected_pointer_width"]
            );
            assert_eq!(
                selected.release_tuple.limit_profile_id,
                case["expect"]["selected_limit_profile_id"]
            );
        }
        "reject" => assert_eq!(
            result.expect_err(&format!("{id} accepted")).code(),
            text(&case["expect"]["code"]),
            "{id}"
        ),
        outcome => panic!("unknown selection outcome {outcome}"),
    }
}

fn validate_installation_case_shape(case: &Value) {
    let construction = &case["construction"];
    assert_eq!(construction["registry_fixture"], "valid_registry");
    assert_eq!(construction["bundle_bytes_fixture"], "bundle_bytes");
    for mutation in construction["mutations"]
        .as_array()
        .expect("installation mutations")
    {
        assert!(matches!(
            text(&mutation["kind"]),
            "remove"
                | "symlink"
                | "regular_file"
                | "chmod"
                | "replace_bytes"
                | "ambient_regular_file"
                | "reparse_point"
                | "hard_link"
                | "fifo"
                | "directory"
                | "replace_after_open"
        ));
    }
}

fn validate_assembler_case_shape(case: &Value) {
    let argv = case["argv"].as_array().expect("assembler argv");
    assert!(argv.len() <= 3);
    assert!(case["expect"]["persistent_writes"].is_array());
    assert!(matches!(
        text(&case["expect"]["network"]),
        "disabled" | "fixed_origins_only"
    ));
}

fn assert_normalization_construction(case: &Value) {
    let fixture = text(&case["construction"]["fixture"]);
    assert!(matches!(
        fixture,
        "normalized_issues" | "normalized_message"
    ));
    assert!(matches!(text(&case["expect"]["outcome"]), "accept"));
}

fn assert_case_record(case: &Value) {
    let object = case.as_object().expect("case object");
    let allowed = [
        "id",
        "input",
        "input_from",
        "construction",
        "process",
        "context",
        "expect",
    ];
    assert!(object.keys().all(|key| allowed.contains(&key.as_str())));
    let sources = ["input", "input_from", "construction"]
        .iter()
        .filter(|key| object.contains_key(**key))
        .count();
    if case.pointer("/process/exit") != Some(&json!(2)) && case.get("process").is_some() {
        assert!(sources <= 1);
    }
}

fn captured_fixture_inputs(root: &Path) -> Vec<(InputKind, String, Vec<u8>)> {
    let vectors = read_json(&root.join("develop/specs/vectors/source-manifest-v0.json"));
    vectors["fixture_inputs"]
        .as_array()
        .expect("fixture inputs")
        .iter()
        .map(|input| {
            let kind = serde_json::from_value(input["kind"].clone()).expect("input kind");
            let bytes = decode_base64(text(&input["base64"]));
            assert_eq!(bytes.len(), input["size_bytes"].as_u64().unwrap() as usize);
            (kind, text(&input["normalized_path"]).to_owned(), bytes)
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

fn apply_patch(root: &mut Value, patch: &Value) {
    let path = text(&patch["path"]);
    let (parent_path, token) = path.rsplit_once('/').expect("patch pointer");
    let parent = if parent_path.is_empty() {
        root
    } else {
        root.pointer_mut(parent_path).expect("patch parent")
    };
    let token = token.replace("~1", "/").replace("~0", "~");
    match text(&patch["op"]) {
        "replace" => *child_mut(parent, &token) = patch["value"].clone(),
        "add" => match parent {
            Value::Array(values) => values.insert(token.parse().unwrap(), patch["value"].clone()),
            Value::Object(values) => {
                values.insert(token, patch["value"].clone());
            }
            _ => panic!("patch add parent is not a container"),
        },
        "remove" => match parent {
            Value::Array(values) => {
                values.remove(token.parse().unwrap());
            }
            Value::Object(values) => {
                values.remove(&token);
            }
            _ => panic!("patch remove parent is not a container"),
        },
        operation => panic!("unsupported patch operation {operation}"),
    }
}

fn child_mut<'a>(value: &'a mut Value, token: &str) -> &'a mut Value {
    match value {
        Value::Array(values) => &mut values[token.parse::<usize>().unwrap()],
        Value::Object(values) => values.get_mut(token).expect("patch object member"),
        _ => panic!("patch parent is not a container"),
    }
}

fn canonical_transport(value: &Value) -> Vec<u8> {
    let raw = serde_json::to_vec(value).expect("serialize JSON");
    let strict = parse_strict_json(&raw, TEST_LIMITS).expect("strict JSON");
    let mut bytes = canonical_json_bytes(&strict).expect("canonical JSON");
    bytes.push(b'\n');
    bytes
}

fn find_case<'a>(cases: &'a Value, id: &str) -> &'a Value {
    cases
        .as_array()
        .expect("case array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing case {id}"))
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let object = value.as_object().expect("object");
    assert_eq!(object.len(), expected.len());
    assert!(expected.iter().all(|key| object.contains_key(*key)));
}

fn text(value: &Value) -> &str {
    value.as_str().expect("string")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn decode_base64(input: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut quartet = [0u8; 4];
    let mut count = 0usize;
    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 64,
            _ => panic!("invalid base64 fixture"),
        };
        quartet[count] = value;
        count += 1;
        if count == 4 {
            output.push((quartet[0] << 2) | (quartet[1] >> 4));
            if quartet[2] != 64 {
                output.push((quartet[1] << 4) | (quartet[2] >> 2));
            }
            if quartet[3] != 64 {
                output.push((quartet[2] << 6) | quartet[3]);
            }
            count = 0;
        }
    }
    assert_eq!(count, 0);
    output
}
