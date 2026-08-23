use rust2vir_internal::driver_process::{
    classify_invocation, consume_result, publish_primary_result, publish_result, read_request,
    WrapperInvocation,
};
use rust2vir_internal::driver_protocol::{
    construct_request, encode_lowered, encode_non_success, parse_output_transport,
    parse_request_transport, validate_transport_size, DriverProtocolCode, DriverStatus,
    PrivateDiagnostic, OUTPUT_TRANSPORT_MAX, REQUEST_TRANSPORT_MAX,
};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::sha256::{digest, hex};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const VECTOR_BYTES: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
const VECTOR_SHA256: &str = "c126a970fdd72eaee41d19fd521a7387c075f3e3779a6d3136cbfaf7856ce640";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[test]
fn captured_inputs_and_release_projection_construct_a_path_free_request() {
    let fixture = Fixture::new();
    let request = construct_request(
        fixture.request(),
        fixture.snapshot().inputs(),
        fixture.candidate().driver_release_identity(),
    )
    .unwrap();
    let text = std::str::from_utf8(request.transport()).unwrap();
    assert!(text.ends_with('\n'));
    assert!(!text.contains(fixture.request().source_root.to_str().unwrap()));
    assert!(!text.contains(fixture.request().release.toolchain_root.to_str().unwrap()));
    assert!(!text.contains(fixture.request().release.driver.to_str().unwrap()));
    assert!(!text.contains("output_path"));
    assert!(text.contains("src/lib.rs"));
    assert_eq!(
        parse_request_transport(request.transport())
            .unwrap()
            .transport(),
        request.transport()
    );
}

#[test]
fn normative_vector_request_and_every_status_are_byte_exact() {
    assert_eq!(hex(&digest(VECTOR_BYTES)), VECTOR_SHA256);
    let vector = vector();
    let root = vector.as_object().unwrap();
    assert_eq!(
        root["schema"].as_str(),
        Some("mpk.rust.driver.conformance.v0")
    );
    assert_eq!(
        root["owner_test"].as_str(),
        Some("rust-tools/rust2vir/tests/driver_protocol.rs")
    );

    let request_fixture = root["valid_request"].as_object().unwrap();
    let request_transport = transport(request_fixture);
    assert_transport_metadata(request_fixture, &request_transport);
    let request = parse_request_transport(&request_transport).unwrap();
    assert_eq!(request.transport(), request_transport);
    assert_eq!(
        request.request_fingerprint(),
        "4726919a5c8b3aa8d21f6a93bb71c2e030300a34d52cca9b30036f3d983c3831"
    );

    let lowered = root["valid_lowered"].as_object().unwrap();
    let lowered_transport = transport(lowered);
    assert_transport_metadata(lowered, &lowered_transport);
    let output = parse_output_transport(&lowered_transport, &request, 0, false).unwrap();
    assert_eq!(output.status(), DriverStatus::Lowered);
    assert_eq!(output.phase(), "lowering");
    assert_eq!(output.transport(), lowered_transport);

    for fixture in root["non_success"].as_array().unwrap() {
        let fixture = fixture.as_object().unwrap();
        let bytes = transport(fixture);
        assert_transport_metadata(fixture, &bytes);
        let exit = fixture["exit"].integer().unwrap() as i32;
        let output = parse_output_transport(&bytes, &request, exit, false).unwrap();
        assert_ne!(output.status(), DriverStatus::Lowered);
        let value = fixture["value"].as_object().unwrap();
        let status = match value["status"].as_str().unwrap() {
            "rejected" => DriverStatus::Rejected,
            "source-error" => DriverStatus::SourceError,
            "frontend-error" => DriverStatus::FrontendError,
            _ => panic!("unknown vector status"),
        };
        let diagnostics = value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|diagnostic| {
                let diagnostic = diagnostic.as_object().unwrap();
                PrivateDiagnostic {
                    code: diagnostic["code"].as_str().unwrap().to_owned(),
                    message: diagnostic["message"].as_str().unwrap().to_owned(),
                    function_id: diagnostic
                        .get("function_id")
                        .and_then(JsonValue::as_str)
                        .map(str::to_owned),
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            encode_non_success(
                &request,
                status,
                value["phase"].as_str().unwrap(),
                &diagnostics,
            )
            .unwrap(),
            bytes
        );
        assert!(!value.contains_key("payload_hash"));
        assert!(!value.contains_key("source_inventory"));
        assert!(!value.contains_key("raw_lowering"));
        assert!(!value.contains_key("raw_source_map"));
    }
}

#[test]
fn lowered_encoder_reconstructs_the_exact_success_inventory_and_hashes() {
    let request = parse_request_transport(&valid_request_bytes()).unwrap();
    let expected = output_value();
    let expected = expected.as_object().unwrap();
    let encoded = encode_lowered(
        &request,
        expected["raw_lowering"].clone(),
        expected["raw_source_map"].clone(),
    )
    .unwrap();
    assert_eq!(encoded, valid_lowered_bytes());
}

#[test]
fn request_transport_shape_hash_and_identity_fail_closed() {
    let request_bytes = valid_request_bytes();
    let request = parse_request_transport(&request_bytes).unwrap();
    assert_code(
        parse_request_transport(&request_bytes[..request_bytes.len() - 1])
            .unwrap_err()
            .code,
        DriverProtocolCode::Transport,
    );
    let mut crlf = request_bytes[..request_bytes.len() - 1].to_vec();
    crlf.extend_from_slice(b"\r\n");
    assert_code(
        parse_request_transport(&crlf).unwrap_err().code,
        DriverProtocolCode::Transport,
    );
    let mut pretty = request_bytes.clone();
    pretty.insert(1, b' ');
    assert_code(
        parse_request_transport(&pretty).unwrap_err().code,
        DriverProtocolCode::Canonical,
    );
    let duplicate = duplicate_root_member(&request_bytes, "schema", "mpk.rust.driver.request.v0");
    assert_code(
        parse_request_transport(&duplicate).unwrap_err().code,
        DriverProtocolCode::Canonical,
    );

    let mut unknown = request.value().clone();
    unknown.as_object_mut().unwrap().insert(
        "output_path".to_owned(),
        JsonValue::String("result".to_owned()),
    );
    assert_code(
        parse_request_transport(&canonical_transport(&unknown))
            .unwrap_err()
            .code,
        DriverProtocolCode::Shape,
    );

    let mut wrong_fingerprint = request.value().clone();
    replace_hash_nibble(&mut wrong_fingerprint, "request_fingerprint");
    assert_code(
        parse_request_transport(&canonical_transport(&wrong_fingerprint))
            .unwrap_err()
            .code,
        DriverProtocolCode::Hash,
    );

    let mut wrong_input_set = request.value().clone();
    replace_hash_nibble(&mut wrong_input_set, "input_set_hash");
    recompute_request_fingerprint(&mut wrong_input_set);
    assert_code(
        parse_request_transport(&canonical_transport(&wrong_input_set))
            .unwrap_err()
            .code,
        DriverProtocolCode::Identity,
    );

    let mut wrong_commit = request.value().clone();
    let root = wrong_commit.as_object_mut().unwrap();
    root.get_mut("compiler")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "commit_hash".to_owned(),
            JsonValue::String("4d08223c054cf5a56d9761ca925fd46ffebe7114".to_owned()),
        );
    root.get_mut("toolchain")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .get_mut("components")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "commit_hash".to_owned(),
            JsonValue::String("4d08223c054cf5a56d9761ca925fd46ffebe7114".to_owned()),
        );
    recompute_request_fingerprint(&mut wrong_commit);
    assert_code(
        parse_request_transport(&canonical_transport(&wrong_commit))
            .unwrap_err()
            .code,
        DriverProtocolCode::ToolchainCommit,
    );

    let mut distinct_package = request.value().clone();
    distinct_package
        .as_object_mut()
        .unwrap()
        .get_mut("selection")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "package".to_owned(),
            JsonValue::String("payment-policy".to_owned()),
        );
    recompute_request_fingerprint(&mut distinct_package);
    parse_request_transport(&canonical_transport(&distinct_package)).unwrap();

    let mut nonportable_path = request.value().clone();
    for field in ["inputs", "source_inventory"] {
        nonportable_path
            .as_object_mut()
            .unwrap()
            .get_mut(field)
            .unwrap()
            .as_array_mut()
            .unwrap()[0]
            .as_object_mut()
            .unwrap()
            .insert(
                "normalized_path".to_owned(),
                JsonValue::String("src/CON.rs".to_owned()),
            );
    }
    recompute_request_hashes(&mut nonportable_path);
    assert_code(
        parse_request_transport(&canonical_transport(&nonportable_path))
            .unwrap_err()
            .code,
        DriverProtocolCode::Shape,
    );
}

#[test]
fn output_transport_branches_and_cross_process_identity_fail_closed() {
    let request = parse_request_transport(&valid_request_bytes()).unwrap();
    let lowered = valid_lowered_bytes();
    assert_code(
        parse_output_transport(&lowered[..lowered.len() - 1], &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Transport,
    );
    assert_code(
        parse_output_transport(&lowered, &request, 0, true)
            .unwrap_err()
            .code,
        DriverProtocolCode::Process,
    );
    let mut pretty = lowered.clone();
    pretty.insert(1, b' ');
    assert_code(
        parse_output_transport(&pretty, &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Canonical,
    );
    let duplicate = duplicate_root_member(&lowered, "schema", "mpk.rust.driver.v0");
    assert_code(
        parse_output_transport(&duplicate, &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Canonical,
    );

    let mut unknown = output_value();
    unknown.as_object_mut().unwrap().insert(
        "certificate_path".to_owned(),
        JsonValue::String("certificate".to_owned()),
    );
    assert_code(
        parse_output_transport(&canonical_transport(&unknown), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Shape,
    );

    let mut wrong_request = output_value();
    replace_hash_nibble(&mut wrong_request, "request_fingerprint");
    assert_code(
        parse_output_transport(&canonical_transport(&wrong_request), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Identity,
    );

    let mut wrong_payload = output_value();
    replace_hash_nibble(&mut wrong_payload, "payload_hash");
    assert_code(
        parse_output_transport(&canonical_transport(&wrong_payload), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Hash,
    );

    let rejected = non_success_value(0);
    let mut partial = rejected.clone();
    partial.as_object_mut().unwrap().insert(
        "source_inventory".to_owned(),
        request.value().as_object().unwrap()["source_inventory"].clone(),
    );
    assert_code(
        parse_output_transport(&canonical_transport(&partial), &request, 3, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Shape,
    );

    let mut external_diagnostic = rejected.clone();
    external_diagnostic
        .as_object_mut()
        .unwrap()
        .get_mut("diagnostics")
        .unwrap()
        .as_array_mut()
        .unwrap()[0]
        .as_object_mut()
        .unwrap()
        .insert(
            "span".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("end".to_owned(), JsonValue::Number("1".to_owned())),
                (
                    "normalized_path".to_owned(),
                    JsonValue::String("external.rs".to_owned()),
                ),
                ("start".to_owned(), JsonValue::Number("0".to_owned())),
            ])),
        );
    assert_code(
        parse_output_transport(
            &canonical_transport(&external_diagnostic),
            &request,
            3,
            false,
        )
        .unwrap_err()
        .code,
        DriverProtocolCode::Shape,
    );

    let mut external = output_value();
    raw_map_origin(&mut external, 0).insert(
        "normalized_path".to_owned(),
        JsonValue::String("rustc/sysroot.rs".to_owned()),
    );
    recompute_payload_hash(&mut external);
    assert_code(
        parse_output_transport(&canonical_transport(&external), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::SourceMapExternal,
    );

    let mut range = output_value();
    raw_map_origin(&mut range, 0).insert("end".to_owned(), JsonValue::Number("36".to_owned()));
    recompute_payload_hash(&mut range);
    assert_code(
        parse_output_transport(&canonical_transport(&range), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::SourceMapRange,
    );

    let mut missing_reference = output_value();
    raw_map_entries(&mut missing_reference).pop();
    recompute_payload_hash(&mut missing_reference);
    assert_code(
        parse_output_transport(&canonical_transport(&missing_reference), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::SourceMapReference,
    );

    let mut instruction = output_value();
    add_instruction_and_mapping(&mut instruction);
    parse_output_transport(&canonical_transport(&instruction), &request, 0, false).unwrap();
    raw_map_entries(&mut instruction).swap(1, 2);
    recompute_payload_hash(&mut instruction);
    assert_code(
        parse_output_transport(&canonical_transport(&instruction), &request, 0, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::SourceMapReference,
    );
}

#[test]
fn fixed_files_use_exclusive_atomic_publication_and_exact_directory_state() {
    assert_eq!(
        case_ids("filesystem_cases"),
        [
            "publish.valid",
            "publish.final_preexists",
            "publish.partial_preexists",
            "consume.unstable_identity",
        ]
    );
    let root = temporary("driver-files");
    let request_path = root.join("driver-request.json");
    fs::write(&request_path, valid_request_bytes()).unwrap();
    fs::set_permissions(&request_path, fs::Permissions::from_mode(0o400)).unwrap();
    assert_eq!(read_request(&request_path).unwrap(), valid_request_bytes());
    fs::set_permissions(&request_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_code(
        read_request(&request_path).unwrap_err().code,
        DriverProtocolCode::Filesystem,
    );
    let oversized_request = root.join("oversized-request.json");
    let file = fs::File::create(&oversized_request).unwrap();
    file.set_len((REQUEST_TRANSPORT_MAX + 1) as u64).unwrap();
    fs::set_permissions(&oversized_request, fs::Permissions::from_mode(0o400)).unwrap();
    assert_code(
        read_request(&oversized_request).unwrap_err().code,
        DriverProtocolCode::Transport,
    );

    let request = parse_request_transport(&valid_request_bytes()).unwrap();
    let output = encode_non_success(
        &request,
        DriverStatus::FrontendError,
        "lowering",
        &[PrivateDiagnostic {
            code: "RUST_TOOLCHAIN_MIR_ADAPTER".to_owned(),
            message: "pinned MIR adapter identity does not match".to_owned(),
            function_id: Some("vector::identity".to_owned()),
        }],
    )
    .unwrap();
    let output_directory = root.join("driver-output");
    fs::create_dir(&output_directory).unwrap();
    fs::set_permissions(&output_directory, fs::Permissions::from_mode(0o700)).unwrap();
    publish_result(&output_directory, &output).unwrap();
    assert_eq!(consume_result(&output_directory).unwrap(), output);
    assert_code(
        publish_result(&output_directory, &output).unwrap_err().code,
        DriverProtocolCode::Filesystem,
    );
    assert_code(
        publish_primary_result(&output_directory, &output)
            .unwrap_err()
            .code,
        DriverProtocolCode::Count,
    );

    let hostile = root.join("hostile-output");
    fs::create_dir(&hostile).unwrap();
    fs::set_permissions(&hostile, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(hostile.join("result.json.partial"), b"partial").unwrap();
    assert_code(
        publish_result(&hostile, &output).unwrap_err().code,
        DriverProtocolCode::Filesystem,
    );
    assert_code(
        consume_result(&hostile).unwrap_err().code,
        DriverProtocolCode::Filesystem,
    );
    fs::remove_file(hostile.join("result.json.partial")).unwrap();
    symlink("missing", hostile.join("result.json")).unwrap();
    assert_code(
        consume_result(&hostile).unwrap_err().code,
        DriverProtocolCode::Filesystem,
    );
    let oversized = root.join("oversized-output");
    fs::create_dir(&oversized).unwrap();
    fs::set_permissions(&oversized, fs::Permissions::from_mode(0o700)).unwrap();
    let file = fs::File::create(oversized.join("result.json")).unwrap();
    file.set_len((OUTPUT_TRANSPORT_MAX + 1) as u64).unwrap();
    fs::set_permissions(
        oversized.join("result.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_code(
        consume_result(&oversized).unwrap_err().code,
        DriverProtocolCode::OutputLimit,
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wrapper_accepts_only_frozen_probes_and_selected_primary() {
    let request = parse_request_transport(&valid_request_bytes()).unwrap();
    assert_eq!(
        classify_invocation(
            &request,
            &["/mpk/toolchain/bin/rustc".to_owned(), "-vV".to_owned()]
        ),
        Ok(WrapperInvocation::VersionProbe)
    );
    assert_eq!(
        classify_invocation(
            &request,
            &[
                "/mpk/toolchain/bin/rustc".to_owned(),
                "--print".to_owned(),
                "sysroot".to_owned(),
            ]
        ),
        Ok(WrapperInvocation::SysrootProbe)
    );
    let vector = vector();
    let cases = vector.as_object().unwrap()["invocation_cases"]
        .as_array()
        .unwrap();
    assert_eq!(
        cases
            .iter()
            .map(|case| case.as_object().unwrap()["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "probe.rustc_vv",
            "probe.sysroot",
            "probe.crate_information_host",
            "probe.crate_information_target",
            "primary.selected_lib",
            "reject.response_file",
            "reject.target_cpu_native",
            "reject.second_primary",
            "reject.unknown_non_primary",
        ]
    );
    for (index, expected) in [
        WrapperInvocation::VersionProbe,
        WrapperInvocation::SysrootProbe,
        WrapperInvocation::CrateInformationHost,
        WrapperInvocation::CrateInformationTarget,
    ]
    .into_iter()
    .enumerate()
    {
        let case = cases[index].as_object().unwrap();
        if let Some(argv) = case.get("argv").and_then(JsonValue::as_array) {
            let argv = argv
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(classify_invocation(&request, &argv), Ok(expected));
        }
    }
    assert_code(
        classify_invocation(
            &request,
            &["/mpk/toolchain/bin/rustc".to_owned(), "@args".to_owned()],
        )
        .unwrap_err()
        .code,
        DriverProtocolCode::Identity,
    );
    let primary = primary_argv();
    assert_eq!(
        classify_invocation(&request, &primary),
        Ok(WrapperInvocation::Primary)
    );
    let mut injected = primary;
    injected.push("-Ctarget-cpu=native".to_owned());
    assert_code(
        classify_invocation(&request, &injected).unwrap_err().code,
        DriverProtocolCode::Identity,
    );
}

#[test]
fn checked_transport_counters_accept_exact_boundaries_only() {
    let vector = vector();
    let cases = vector.as_object().unwrap()["limit_cases"]
        .as_array()
        .unwrap();
    assert_eq!(
        case_ids("limit_cases"),
        [
            "request.at",
            "request.above",
            "output.at",
            "output.above",
            "request.counter_overflow",
            "output.counter_overflow",
        ]
    );
    for (case, expected) in cases.iter().take(4).zip([
        (REQUEST_TRANSPORT_MAX - 1, 1),
        (REQUEST_TRANSPORT_MAX, 1),
        (OUTPUT_TRANSPORT_MAX - 1, 1),
        (OUTPUT_TRANSPORT_MAX, 1),
    ]) {
        let construction = case.as_object().unwrap()["construction"]
            .as_object()
            .unwrap();
        assert_eq!(construction["jcs_bytes"].integer(), Some(expected.0 as i64));
        assert_eq!(construction["lf_bytes"].integer(), Some(expected.1));
    }
    assert_eq!(
        validate_transport_size(REQUEST_TRANSPORT_MAX - 1, false),
        Ok(REQUEST_TRANSPORT_MAX)
    );
    assert_code(
        validate_transport_size(REQUEST_TRANSPORT_MAX, false)
            .unwrap_err()
            .code,
        DriverProtocolCode::Transport,
    );
    assert_eq!(
        validate_transport_size(OUTPUT_TRANSPORT_MAX - 1, true),
        Ok(OUTPUT_TRANSPORT_MAX)
    );
    assert_code(
        validate_transport_size(OUTPUT_TRANSPORT_MAX, true)
            .unwrap_err()
            .code,
        DriverProtocolCode::OutputLimit,
    );
    assert_code(
        validate_transport_size(usize::MAX, false).unwrap_err().code,
        DriverProtocolCode::Transport,
    );
    assert_code(
        validate_transport_size(usize::MAX, true).unwrap_err().code,
        DriverProtocolCode::OutputLimit,
    );
}

fn vector() -> JsonValue {
    json::parse(VECTOR_BYTES, VECTOR_BYTES.len()).unwrap()
}

fn case_ids(name: &str) -> Vec<String> {
    vector().as_object().unwrap()[name]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case.as_object().unwrap()["id"].as_str().unwrap().to_owned())
        .collect()
}

fn valid_request_bytes() -> Vec<u8> {
    let vector = vector();
    transport(
        vector.as_object().unwrap()["valid_request"]
            .as_object()
            .unwrap(),
    )
}

fn valid_lowered_bytes() -> Vec<u8> {
    let vector = vector();
    transport(
        vector.as_object().unwrap()["valid_lowered"]
            .as_object()
            .unwrap(),
    )
}

fn output_value() -> JsonValue {
    let vector = vector();
    vector.as_object().unwrap()["valid_lowered"]
        .as_object()
        .unwrap()["value"]
        .clone()
}

fn non_success_value(index: usize) -> JsonValue {
    let vector = vector();
    vector.as_object().unwrap()["non_success"]
        .as_array()
        .unwrap()[index]
        .as_object()
        .unwrap()["value"]
        .clone()
}

fn transport(fixture: &BTreeMap<String, JsonValue>) -> Vec<u8> {
    let encoded = fixture["transport"].as_object().unwrap()["base64"]
        .as_str()
        .unwrap();
    decode_base64(encoded)
}

fn assert_transport_metadata(fixture: &BTreeMap<String, JsonValue>, bytes: &[u8]) {
    let transport = fixture["transport"].as_object().unwrap();
    assert_eq!(
        bytes.len() as i64,
        transport["utf8_length"].integer().unwrap()
    );
    assert_eq!(hex(&digest(bytes)), transport["sha256"].as_str().unwrap());
}

fn decode_base64(value: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(value.len() / 4 * 3);
    for chunk in value.as_bytes().chunks_exact(4) {
        let values = [
            base64_value(chunk[0]),
            base64_value(chunk[1]),
            base64_value(chunk[2]),
            base64_value(chunk[3]),
        ];
        output.push((values[0] << 2) | (values[1] >> 4));
        if chunk[2] != b'=' {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if chunk[3] != b'=' {
            output.push((values[2] << 6) | values[3]);
        }
    }
    output
}

fn base64_value(byte: u8) -> u8 {
    match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        b'=' => 0,
        _ => panic!("invalid base64 vector"),
    }
}

fn canonical_transport(value: &JsonValue) -> Vec<u8> {
    let mut bytes = json::canonical(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn duplicate_root_member(bytes: &[u8], name: &str, value: &str) -> Vec<u8> {
    let mut duplicate = format!("{{\"{name}\":\"{value}\",").into_bytes();
    duplicate.extend_from_slice(&bytes[1..]);
    duplicate
}

fn replace_hash_nibble(value: &mut JsonValue, field: &str) {
    let hash = value.as_object_mut().unwrap()[field].as_str().unwrap();
    let replacement = format!(
        "{}{}",
        if &hash[..1] == "0" { "1" } else { "0" },
        &hash[1..]
    );
    value
        .as_object_mut()
        .unwrap()
        .insert(field.to_owned(), JsonValue::String(replacement));
}

fn recompute_request_fingerprint(value: &mut JsonValue) {
    value.as_object_mut().unwrap().remove("request_fingerprint");
    let mut hasher = rust2vir_internal::sha256::Sha256::new();
    hasher.update(b"MPK-RUST-DRIVER-REQUEST-0.1");
    hasher.update(&[0]);
    hasher.update(&json::canonical(value).unwrap());
    value.as_object_mut().unwrap().insert(
        "request_fingerprint".to_owned(),
        JsonValue::String(hex(&hasher.finish())),
    );
}

fn recompute_request_hashes(value: &mut JsonValue) {
    for (field, payload, domain) in [
        ("input_set_hash", "inputs", b"MPK-INPUT-SET-0.1".as_slice()),
        (
            "source_inventory_hash",
            "source_inventory",
            b"MPK-RUST-SOURCE-INVENTORY-0.1".as_slice(),
        ),
    ] {
        let canonical = {
            let root = value.as_object().unwrap();
            json::canonical(&root[payload]).unwrap()
        };
        value.as_object_mut().unwrap().insert(
            field.to_owned(),
            JsonValue::String(domain_hash(domain, &canonical)),
        );
    }
    recompute_request_fingerprint(value);
}

fn recompute_payload_hash(value: &mut JsonValue) {
    value.as_object_mut().unwrap().remove("payload_hash");
    let mut hasher = rust2vir_internal::sha256::Sha256::new();
    hasher.update(b"MPK-RUST-DRIVER-PAYLOAD-0.1");
    hasher.update(&[0]);
    hasher.update(&json::canonical(value).unwrap());
    value.as_object_mut().unwrap().insert(
        "payload_hash".to_owned(),
        JsonValue::String(hex(&hasher.finish())),
    );
}

fn add_instruction_and_mapping(value: &mut JsonValue) {
    let vir = value
        .as_object_mut()
        .unwrap()
        .get_mut("raw_lowering")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .get_mut("vir")
        .unwrap()
        .as_object_mut()
        .unwrap();
    vir.get_mut("units").unwrap().as_array_mut().unwrap()[0]
        .as_object_mut()
        .unwrap()
        .get_mut("functions")
        .unwrap()
        .as_array_mut()
        .unwrap()[0]
        .as_object_mut()
        .unwrap()
        .get_mut("blocks")
        .unwrap()
        .as_array_mut()
        .unwrap()[0]
        .as_object_mut()
        .unwrap()
        .get_mut("instructions")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(JsonValue::Object(BTreeMap::from([(
            "id".to_owned(),
            JsonValue::String("t0".to_owned()),
        )])));
    vir.remove("vir_hash");
    let hash = domain_hash(
        b"MPK-VIR-0.1",
        &json::canonical(&JsonValue::Object(vir.clone())).unwrap(),
    );
    vir.insert("vir_hash".to_owned(), JsonValue::String(hash.clone()));

    value
        .as_object_mut()
        .unwrap()
        .get_mut("raw_source_map")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("source_ir_hash".to_owned(), JsonValue::String(hash));
    let source_origin = raw_map_entries(value)[0].as_object().unwrap()["origin"].clone();
    raw_map_entries(value).insert(
        1,
        JsonValue::Object(BTreeMap::from([
            ("origin".to_owned(), source_origin),
            (
                "reference".to_owned(),
                JsonValue::Object(BTreeMap::from([
                    ("block".to_owned(), JsonValue::String("bb0".to_owned())),
                    (
                        "function_id".to_owned(),
                        JsonValue::String("vector::identity".to_owned()),
                    ),
                    ("instruction".to_owned(), JsonValue::String("t0".to_owned())),
                    (
                        "kind".to_owned(),
                        JsonValue::String("instruction".to_owned()),
                    ),
                    ("unit_id".to_owned(), JsonValue::String("vector".to_owned())),
                ])),
            ),
        ])),
    );
    recompute_payload_hash(value);
}

fn domain_hash(domain: &[u8], payload: &[u8]) -> String {
    let mut hasher = rust2vir_internal::sha256::Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(payload);
    hex(&hasher.finish())
}

fn raw_map_entries(value: &mut JsonValue) -> &mut Vec<JsonValue> {
    value
        .as_object_mut()
        .unwrap()
        .get_mut("raw_source_map")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .get_mut("entries")
        .unwrap()
        .as_array_mut()
        .unwrap()
}

fn raw_map_origin(value: &mut JsonValue, index: usize) -> &mut BTreeMap<String, JsonValue> {
    raw_map_entries(value)[index]
        .as_object_mut()
        .unwrap()
        .get_mut("origin")
        .unwrap()
        .as_object_mut()
        .unwrap()
}

fn temporary(label: &str) -> PathBuf {
    let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("rust2vir-{label}-{}-{serial}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn primary_argv() -> Vec<String> {
    [
        "/mpk/toolchain/bin/rustc",
        "--crate-name",
        "vector",
        "--edition=2021",
        "src/lib.rs",
        "--error-format=json",
        "--json=diagnostic-rendered-ansi,artifacts,future-incompat",
        "--crate-type",
        "lib",
        "--emit=dep-info,metadata",
        "-C",
        "embed-bitcode=no",
        "-C",
        "debuginfo=2",
        "--check-cfg",
        "cfg(docsrs,test)",
        "--check-cfg",
        "cfg(feature, values(\"default\"))",
        "-C",
        "metadata=0123456789abcdef",
        "-C",
        "extra-filename=-fedcba9876543210",
        "--out-dir",
        "/mpk/target/x86_64-unknown-linux-gnu/debug/deps",
        "--target",
        "x86_64-unknown-linux-gnu",
        "-L",
        "dependency=/mpk/target/x86_64-unknown-linux-gnu/debug/deps",
        "-L",
        "dependency=/mpk/target/debug/deps",
        "-C",
        "overflow-checks=yes",
        "-C",
        "panic=abort",
        "-C",
        "debug-assertions=no",
        "-C",
        "opt-level=0",
        "-Z",
        "mir-opt-level=0",
        "--remap-path-prefix=/mpk/input=.",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn assert_code(actual: DriverProtocolCode, expected: DriverProtocolCode) {
    assert_eq!(actual, expected, "{}", actual.as_str());
}

trait JsonArrayMut {
    fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>>;
}

impl JsonArrayMut for JsonValue {
    fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(values) => Some(values),
            _ => None,
        }
    }
}
mod common;

use common::Fixture;
