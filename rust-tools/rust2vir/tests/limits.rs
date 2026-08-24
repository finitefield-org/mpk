use rust2vir_internal::cli::{LowerRequest, ReleaseArguments, RustSelection, RustTarget};
use rust2vir_internal::driver_protocol::{
    encode_non_success, parse_output_transport, parse_request_transport, DriverOutput,
    DriverRequest, DriverStatus, PrivateDiagnostic,
};
use rust2vir_internal::emit::driver_non_success_envelope;
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::limits::{
    checked_add, validate_limit, validate_rust_limit, RustLimitError, RustLimitId,
};
use rust2vir_internal::sandbox::{
    validate_resource_filesystem_observation, ResourceFilesystemObservation, SandboxError,
    SandboxLimits,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const DRIVER_VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
const FUNCTION_ID: &str = "vector::identity";

#[test]
fn resource_filesystem_configuration_is_exact_below_at_above() {
    let limits = SandboxLimits::FROZEN;
    let at = ResourceFilesystemObservation {
        filesystem_type: 0x0102_1994,
        allocated_capacity_bytes: limits.writable_allocated_bytes,
        inode_capacity: limits.writable_inodes,
        same_device: true,
        nosuid: true,
        nodev: true,
        noswap: true,
    };
    assert_eq!(validate_resource_filesystem_observation(at, limits), Ok(()));

    for allocated_capacity_bytes in [
        limits.writable_allocated_bytes - 1,
        limits.writable_allocated_bytes + 1,
    ] {
        assert_eq!(
            validate_resource_filesystem_observation(
                ResourceFilesystemObservation {
                    allocated_capacity_bytes,
                    ..at
                },
                limits,
            ),
            Err(SandboxError::SandboxUnavailable)
        );
    }
    for inode_capacity in [limits.writable_inodes - 1, limits.writable_inodes + 1] {
        assert_eq!(
            validate_resource_filesystem_observation(
                ResourceFilesystemObservation {
                    inode_capacity,
                    ..at
                },
                limits,
            ),
            Err(SandboxError::SandboxUnavailable)
        );
    }
}

#[test]
fn every_frozen_counter_accepts_below_and_at_and_rejects_above_and_overflow() {
    assert_eq!(RustLimitId::ALL.len(), 35);
    let mut ids = BTreeSet::new();

    for limit in RustLimitId::ALL {
        assert!(ids.insert(limit.as_str()), "duplicate limit ID {limit:?}");
        let at = limit.maximum();
        let below = at.checked_sub(1).expect("all frozen maxima are positive");
        let above = at.checked_add(1).expect("frozen maxima fit in u64");

        assert_eq!(
            validate_limit(limit, below),
            Ok(()),
            "{} below",
            limit.as_str()
        );
        assert_eq!(validate_limit(limit, at), Ok(()), "{} at", limit.as_str());
        assert_eq!(
            validate_rust_limit(limit.as_str(), at),
            Ok(()),
            "{} string lookup",
            limit.as_str()
        );
        let exceeded = RustLimitError::Exceeded {
            limit,
            observed: above,
        };
        assert_eq!(validate_limit(limit, above), Err(exceeded.clone()));
        assert_eq!(exceeded.action(), Some(limit.above_action()));
        assert_eq!(checked_add(limit, below, 1), Ok(at));
        assert_eq!(checked_add(limit, at, 1), Err(exceeded));

        let overflow = RustLimitError::CounterOverflow { limit };
        assert_eq!(checked_add(limit, u64::MAX, 1), Err(overflow.clone()));
        assert_eq!(overflow.action(), Some(limit.above_action()));
    }

    assert_eq!(ids.len(), RustLimitId::ALL.len());
    assert_eq!(
        validate_rust_limit("not_a_registered_rust_limit", 0),
        Err(RustLimitError::Unknown(
            "not_a_registered_rust_limit".to_owned()
        ))
    );
}

#[test]
fn diagnostic_message_boundary_is_scalar_exact_and_preserves_source_status() {
    let (request, lower) = request_fixture();
    let exact = "é".repeat(2_048);
    assert_eq!(
        exact.len(),
        RustLimitId::NormalizedIssueMessage.maximum() as usize
    );

    let exact_output = encode_and_parse(
        &request,
        DriverStatus::SourceError,
        "typecheck",
        vec![diagnostic("RUST_SOURCE_TYPE", exact.clone())],
    );
    assert_eq!(exact_output.status(), DriverStatus::SourceError);
    assert_eq!(private_messages(&exact_output), [exact]);

    let above = "é".repeat(2_049);
    let expected = format!("{} [truncated]", "é".repeat(2_042));
    assert_eq!(above.len(), 4_098);
    assert_eq!(expected.len(), 4_096);
    let above_output = encode_and_parse(
        &request,
        DriverStatus::SourceError,
        "typecheck",
        vec![diagnostic("RUST_SOURCE_TYPE", above)],
    );
    assert_eq!(above_output.status(), DriverStatus::SourceError);
    assert_eq!(
        private_messages(&above_output),
        std::slice::from_ref(&expected)
    );

    let public = public_result(&lower, &above_output);
    assert_eq!(text(&public, "status"), "source-error");
    assert_eq!(text(&public, "phase"), "typecheck");
    assert!(array(&public, "rejected_features").is_empty());
    let diagnostics = array(&public, "diagnostics");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(issue_text(&diagnostics[0], "message"), expected);
}

#[test]
fn diagnostic_entry_boundary_sorts_then_reserves_the_final_public_marker() {
    let (request, lower) = request_fixture();
    let maximum = RustLimitId::NormalizedIssueEntries.maximum() as usize;
    let exact = (0..maximum)
        .rev()
        .map(|index| diagnostic("RUST_SUBSET_ITEM", format!("issue-{index:04}")))
        .collect::<Vec<_>>();
    let exact_output = encode_and_parse(&request, DriverStatus::Rejected, "subset", exact);
    assert_eq!(exact_output.status(), DriverStatus::Rejected);
    let exact_private = private_issues(&exact_output);
    assert_eq!(exact_private.len(), maximum);
    assert_eq!(issue_text(&exact_private[0], "message"), "issue-0000");
    assert_eq!(
        issue_text(exact_private.last().unwrap(), "message"),
        "issue-1023"
    );
    assert!(exact_private
        .iter()
        .all(|issue| issue_text(issue, "code") != "RUST_LIMIT_DIAGNOSTICS_TRUNCATED"));

    let exact_public = public_result(&lower, &exact_output);
    assert_eq!(text(&exact_public, "status"), "rejected");
    assert_eq!(array(&exact_public, "rejected_features").len(), maximum);
    assert!(array(&exact_public, "diagnostics").is_empty());

    let above = (0..=maximum)
        .rev()
        .map(|index| diagnostic("RUST_SUBSET_ITEM", format!("issue-{index:04}")))
        .collect::<Vec<_>>();
    let above_output = encode_and_parse(&request, DriverStatus::Rejected, "subset", above);
    assert_eq!(above_output.status(), DriverStatus::Rejected);
    let above_private = private_issues(&above_output);
    assert_eq!(above_private.len(), maximum);
    assert_eq!(issue_text(&above_private[0], "message"), "issue-0000");
    assert_eq!(
        issue_text(&above_private[maximum - 2], "message"),
        "issue-1022"
    );
    assert_marker(&above_private[maximum - 1], "2 normalized issues omitted");

    let above_public = public_result(&lower, &above_output);
    assert_eq!(text(&above_public, "status"), "rejected");
    assert_eq!(text(&above_public, "phase"), "subset");
    let rejected = array(&above_public, "rejected_features");
    let diagnostics = array(&above_public, "diagnostics");
    assert_eq!(rejected.len(), maximum - 1);
    assert_eq!(issue_text(&rejected[0], "message"), "issue-0000");
    assert_eq!(
        issue_text(rejected.last().unwrap(), "message"),
        "issue-1022"
    );
    assert_eq!(diagnostics.len(), 1);
    assert_marker(&diagnostics[0], "2 normalized issues omitted");
    for artifact in ["ir", "source_map", "source_manifest"] {
        assert!(!above_public.contains_key(artifact));
    }
}

#[test]
fn diagnostic_total_boundary_keeps_the_longest_prefix_without_reclassification() {
    let (request, lower) = request_fixture();
    let message_max = RustLimitId::NormalizedIssueMessage.maximum() as usize;
    let total_max = RustLimitId::NormalizedIssueMessageTotal.maximum() as usize;
    let exact_count = total_max / message_max;
    assert_eq!(exact_count, 512);

    let exact = (0..exact_count)
        .rev()
        .map(|index| diagnostic("RUST_SOURCE_TYPE", full_message(index, message_max)))
        .collect::<Vec<_>>();
    let exact_output = encode_and_parse(&request, DriverStatus::SourceError, "typecheck", exact);
    let exact_issues = private_issues(&exact_output);
    assert_eq!(exact_issues.len(), exact_count);
    assert_eq!(
        exact_issues
            .iter()
            .map(|issue| issue_text(issue, "message").len())
            .sum::<usize>(),
        total_max
    );
    assert!(exact_issues
        .iter()
        .all(|issue| issue_text(issue, "code") != "RUST_LIMIT_DIAGNOSTICS_TRUNCATED"));

    let above = (0..=exact_count)
        .rev()
        .map(|index| diagnostic("RUST_SOURCE_TYPE", full_message(index, message_max)))
        .collect::<Vec<_>>();
    let above_output = encode_and_parse(&request, DriverStatus::SourceError, "typecheck", above);
    assert_eq!(above_output.status(), DriverStatus::SourceError);
    let above_issues = private_issues(&above_output);
    assert_eq!(above_issues.len(), exact_count);
    assert_marker(above_issues.last().unwrap(), "2 normalized issues omitted");
    assert!(
        above_issues
            .iter()
            .map(|issue| issue_text(issue, "message").len())
            .sum::<usize>()
            <= total_max
    );

    let public = public_result(&lower, &above_output);
    assert_eq!(text(&public, "status"), "source-error");
    assert_eq!(text(&public, "phase"), "typecheck");
    assert!(array(&public, "rejected_features").is_empty());
    let diagnostics = array(&public, "diagnostics");
    assert_eq!(diagnostics.len(), exact_count);
    assert_marker(diagnostics.last().unwrap(), "2 normalized issues omitted");
}

#[test]
fn bounded_canonical_encoder_counts_escaped_bytes_before_emission() {
    let value = JsonValue::Object(BTreeMap::from([
        (
            "array".to_owned(),
            JsonValue::Array(vec![
                JsonValue::Null,
                JsonValue::Bool(true),
                JsonValue::Number("-17".to_owned()),
            ]),
        ),
        (
            "escaped".to_owned(),
            JsonValue::String("\0\"\\\né".to_owned()),
        ),
    ]));
    let canonical = json::canonical(&value).unwrap();
    let at = canonical.len();
    let below = at - 1;

    assert_eq!(json::canonical_size(&value, at), Ok(at));
    assert_eq!(json::canonical_bounded(&value, at), Ok(canonical));
    assert_eq!(json::canonical_size(&value, below), Err(json::JsonError));
    assert_eq!(json::canonical_bounded(&value, below), Err(json::JsonError));

    let exact_4k = JsonValue::String("x".repeat(4_094));
    assert_eq!(json::canonical_size(&exact_4k, 4_096), Ok(4_096));
    assert_eq!(
        json::canonical_bounded(&exact_4k, 4_096).unwrap().len(),
        4_096
    );
    assert_eq!(
        json::canonical_bounded(&exact_4k, 4_095),
        Err(json::JsonError)
    );
}

fn request_fixture() -> (DriverRequest, LowerRequest) {
    let vector = json::parse(DRIVER_VECTOR, DRIVER_VECTOR.len()).unwrap();
    let fixture = &vector.as_object().unwrap()["valid_request"];
    let request = parse_request_transport(&transport(fixture)).unwrap();
    let lower = lower_request(&request);
    (request, lower)
}

fn lower_request(request: &DriverRequest) -> LowerRequest {
    let root = request.value().as_object().unwrap();
    let selection = root["selection"].as_object().unwrap();
    let frontend = root["frontend"].as_object().unwrap();
    let toolchain = root["toolchain"].as_object().unwrap();
    let registry = root["release_registry"].as_object().unwrap();
    LowerRequest {
        source_root: PathBuf::from("/source-not-emitted"),
        selection: RustSelection {
            package: selection["package"].as_str().unwrap().to_owned(),
            crate_name: selection["crate"].as_str().unwrap().to_owned(),
            kind: "lib",
            function: selection["function"].as_str().unwrap().to_owned(),
        },
        semantic_profile: "mpk.rust.checked.v0",
        target: RustTarget::X86_64UnknownLinuxGnu,
        release: ReleaseArguments {
            frontend_bundle_id: frontend["bundle_id"].as_str().unwrap().to_owned(),
            frontend_sha256: frontend["binary_sha256"].as_str().unwrap().to_owned(),
            release_registry_id: registry["id"].as_str().unwrap().to_owned(),
            release_registry_sha256: registry["registry_sha256"].as_str().unwrap().to_owned(),
            toolchain_bundle_id: toolchain["bundle_id"].as_str().unwrap().to_owned(),
            toolchain_root: PathBuf::from("/toolchain-not-emitted"),
            toolchain_distribution_sha256: toolchain["distribution_sha256"]
                .as_str()
                .unwrap()
                .to_owned(),
            driver: PathBuf::from("/driver-not-emitted"),
            driver_sha256: "0".repeat(64),
        },
        contracts: vec!["contracts/vector.json".to_owned()],
    }
}

fn encode_and_parse(
    request: &DriverRequest,
    status: DriverStatus,
    phase: &str,
    diagnostics: Vec<PrivateDiagnostic>,
) -> DriverOutput {
    let bytes = encode_non_success(request, status, phase, &diagnostics).unwrap();
    parse_output_transport(&bytes, request, status.exit_code(), false).unwrap()
}

fn diagnostic(code: &str, message: String) -> PrivateDiagnostic {
    PrivateDiagnostic {
        code: code.to_owned(),
        message,
        function_id: Some(FUNCTION_ID.to_owned()),
    }
}

fn full_message(index: usize, maximum: usize) -> String {
    let prefix = format!("{index:04}:");
    assert!(prefix.len() < maximum);
    format!("{prefix}{}", "x".repeat(maximum - prefix.len()))
}

fn private_issues(output: &DriverOutput) -> &[JsonValue] {
    output.value().as_object().unwrap()["diagnostics"]
        .as_array()
        .unwrap()
}

fn private_messages(output: &DriverOutput) -> Vec<String> {
    private_issues(output)
        .iter()
        .map(|issue| issue_text(issue, "message").to_owned())
        .collect()
}

fn public_result(request: &LowerRequest, output: &DriverOutput) -> BTreeMap<String, JsonValue> {
    let bytes = driver_non_success_envelope(request, output).unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    json::parse(&bytes[..bytes.len() - 1], bytes.len() - 1)
        .unwrap()
        .as_object()
        .unwrap()
        .clone()
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().unwrap()["value"].clone();
    let mut bytes = json::canonical(&value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn text<'a>(object: &'a BTreeMap<String, JsonValue>, field: &str) -> &'a str {
    object[field].as_str().unwrap()
}

fn array<'a>(object: &'a BTreeMap<String, JsonValue>, field: &str) -> &'a [JsonValue] {
    object[field].as_array().unwrap()
}

fn issue_text<'a>(issue: &'a JsonValue, field: &str) -> &'a str {
    issue.as_object().unwrap()[field].as_str().unwrap()
}

fn assert_marker(issue: &JsonValue, message: &str) {
    let issue = issue.as_object().unwrap();
    assert_eq!(
        issue["code"].as_str(),
        Some("RUST_LIMIT_DIAGNOSTICS_TRUNCATED")
    );
    assert_eq!(issue["message"].as_str(), Some(message));
    assert!(!issue.contains_key("function_id"));
    assert!(!issue.contains_key("span"));
}
