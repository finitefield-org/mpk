use rust2vir_internal::cli::{
    LowerRequest, NonSuccessStatus, ReleaseArguments, RustSelection, RustTarget,
};
use rust2vir_internal::driver_protocol::{
    encode_non_success, parse_output_transport, parse_request_transport, DriverStatus,
    PrivateDiagnostic,
};
use rust2vir_internal::emit::{
    driver_non_success_envelope, local_non_success_envelope, success_envelope,
};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::sha256::{hex, Sha256};
use std::path::PathBuf;

const VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");

#[test]
fn validated_private_lowering_emits_one_canonical_public_envelope() {
    let (request, output, lower) = lowered_fixture();
    let first = success_envelope(&lower, &request, &output, false).unwrap();
    let second = success_envelope(&lower, &request, &output, false).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.last(), Some(&b'\n'));
    assert!(!first[..first.len() - 1].contains(&b'\n'));
    let envelope = json::parse(&first[..first.len() - 1], first.len()).unwrap();
    assert_eq!(
        json::canonical(&envelope).unwrap(),
        first[..first.len() - 1]
    );
    let root = envelope.as_object().unwrap();
    assert_eq!(root["status"].as_str(), Some("ir-lowered"));
    assert_eq!(root["phase"].as_str(), Some("emission"));
    assert!(root["diagnostics"].as_array().unwrap().is_empty());
    assert!(root["rejected_features"].as_array().unwrap().is_empty());
    let vir = root["ir"].as_object().unwrap();
    assert_eq!(vir["sha256"], vir["value"].as_object().unwrap()["vir_hash"]);
    let manifest = root["source_manifest"].as_object().unwrap();
    assert_eq!(
        manifest["limit_profile"].as_str(),
        Some("mpk.vir.limits.v0")
    );
    assert_eq!(
        manifest["units"].as_array().unwrap()[0]
            .as_object()
            .unwrap()["name"]
            .as_str(),
        Some("vector")
    );
    let target = manifest["target"].as_object().unwrap();
    assert_eq!(target["id"].as_str(), Some("x86_64-unknown-linux-gnu"));
    assert_eq!(target["pointer_width"].integer(), Some(64));
    assert_eq!(manifest["semantic_context"], root["semantic_context"]);
    assert_eq!(
        manifest["source_map_hash"],
        root["source_map"].as_object().unwrap()["source_map_hash"]
    );
    assert_self_hash(
        &root["source_map"],
        "source_map_hash",
        b"MPK-SOURCE-MAP-1.0",
    );
    assert_self_hash(
        &root["source_manifest"],
        "source_manifest_hash",
        b"MPK-SOURCE-MANIFEST-1.0",
    );
    let text = std::str::from_utf8(&first).unwrap();
    for forbidden in ["/tmp/", "/root/", "/mpk/toolchain", "rustc --"] {
        assert!(!text.contains(forbidden));
    }
}

#[test]
fn non_success_statuses_route_normalized_issues_without_artifacts() {
    let vector = vector();
    let root = vector.as_object().unwrap();
    let request = parse_request_transport(&transport(&root["valid_request"])).unwrap();
    let lower = lower_request(&request);
    for fixture in root["non_success"].as_array().unwrap() {
        let fixture = fixture.as_object().unwrap();
        let exit = fixture["exit"].integer().unwrap() as i32;
        let output = parse_output_transport(
            &transport(&JsonValue::Object(fixture.clone())),
            &request,
            exit,
            false,
        )
        .unwrap();
        let public = driver_non_success_envelope(&lower, &output).unwrap();
        let envelope = json::parse(&public[..public.len() - 1], public.len()).unwrap();
        let envelope = envelope.as_object().unwrap();
        assert!(!envelope.contains_key("ir"));
        assert!(!envelope.contains_key("source_map"));
        assert!(!envelope.contains_key("source_manifest"));
        if envelope["status"].as_str() == Some("rejected") {
            assert!(!envelope["rejected_features"].as_array().unwrap().is_empty());
            assert!(envelope["diagnostics"].as_array().unwrap().is_empty());
        } else {
            assert!(envelope["rejected_features"].as_array().unwrap().is_empty());
            assert!(!envelope["diagnostics"].as_array().unwrap().is_empty());
        }
    }
    let local = local_non_success_envelope(
        &lower,
        NonSuccessStatus::FrontendError,
        "lowering",
        "RUST_FRONTEND_DRIVER_PROTOCOL_HASH",
        "public artifact hash validation failed",
    )
    .unwrap();
    let local = json::parse(&local[..local.len() - 1], local.len()).unwrap();
    assert_eq!(
        local.as_object().unwrap()["diagnostics"]
            .as_array()
            .unwrap()[0]
            .as_object()
            .unwrap()["function_id"]
            .as_str(),
        Some("vector::identity")
    );
}

#[test]
fn sandbox_unavailable_remains_shared_frontend_error_at_metadata_and_typecheck() {
    let (_, _, lower) = lowered_fixture();
    for phase in ["metadata", "typecheck"] {
        let transport = local_non_success_envelope(
            &lower,
            NonSuccessStatus::FrontendError,
            phase,
            "FRONTEND_SANDBOX_UNAVAILABLE",
            "required isolated execution is unavailable",
        )
        .unwrap();
        let envelope = json::parse(&transport[..transport.len() - 1], transport.len()).unwrap();
        let envelope = envelope.as_object().unwrap();
        assert_eq!(envelope["status"].as_str(), Some("frontend-error"));
        assert_eq!(envelope["phase"].as_str(), Some(phase));
        assert_eq!(
            envelope["diagnostics"].as_array().unwrap()[0]
                .as_object()
                .unwrap()["code"]
                .as_str(),
            Some("FRONTEND_SANDBOX_UNAVAILABLE")
        );
        assert!(envelope["rejected_features"].as_array().unwrap().is_empty());
        for artifact in ["ir", "source_map", "source_manifest"] {
            assert!(!envelope.contains_key(artifact));
        }
    }
}

#[test]
fn diagnostics_truncate_messages_and_append_the_exact_final_marker() {
    let (private_request, _, lower) = lowered_fixture();
    let long_message = format!("{}é", "x".repeat(4_095));
    let transport = encode_non_success(
        &private_request,
        DriverStatus::SourceError,
        "typecheck",
        &[PrivateDiagnostic {
            code: "RUST_SOURCE_TYPE".to_owned(),
            message: long_message,
            function_id: Some("vector::identity".to_owned()),
        }],
    )
    .unwrap();
    let output = parse_output_transport(&transport, &private_request, 4, false).unwrap();
    let public = driver_non_success_envelope(&lower, &output).unwrap();
    let public = json::parse(&public[..public.len() - 1], public.len()).unwrap();
    let message = public.as_object().unwrap()["diagnostics"]
        .as_array()
        .unwrap()[0]
        .as_object()
        .unwrap()["message"]
        .as_str()
        .unwrap();
    assert_eq!(message.len(), 4_096);
    assert!(message.ends_with(" [truncated]"));
    assert_eq!(
        public.as_object().unwrap()["status"].as_str(),
        Some("source-error")
    );

    let diagnostics = (0..1_025)
        .map(|_| PrivateDiagnostic {
            code: "RUST_SUBSET_ITEM".to_owned(),
            message: "unsupported item".to_owned(),
            function_id: Some("vector::identity".to_owned()),
        })
        .collect::<Vec<_>>();
    let transport = encode_non_success(
        &private_request,
        DriverStatus::Rejected,
        "subset",
        &diagnostics,
    )
    .unwrap();
    let output = parse_output_transport(&transport, &private_request, 3, false).unwrap();
    let public = driver_non_success_envelope(&lower, &output).unwrap();
    let public = json::parse(&public[..public.len() - 1], public.len()).unwrap();
    let root = public.as_object().unwrap();
    assert_eq!(root["rejected_features"].as_array().unwrap().len(), 1_023);
    let marker = root["diagnostics"].as_array().unwrap();
    assert_eq!(marker.len(), 1);
    assert_eq!(
        marker[0].as_object().unwrap()["code"].as_str(),
        Some("RUST_LIMIT_DIAGNOSTICS_TRUNCATED")
    );
    assert_eq!(
        marker[0].as_object().unwrap()["message"].as_str(),
        Some("2 normalized issues omitted")
    );
    assert!(!marker[0].as_object().unwrap().contains_key("function_id"));

    let diagnostics = (0..513)
        .map(|_| PrivateDiagnostic {
            code: "RUST_SOURCE_TYPE".to_owned(),
            message: "x".repeat(4_096),
            function_id: Some("vector::identity".to_owned()),
        })
        .collect::<Vec<_>>();
    let transport = encode_non_success(
        &private_request,
        DriverStatus::SourceError,
        "typecheck",
        &diagnostics,
    )
    .unwrap();
    let output = parse_output_transport(&transport, &private_request, 4, false).unwrap();
    let public = driver_non_success_envelope(&lower, &output).unwrap();
    let public = json::parse(&public[..public.len() - 1], public.len()).unwrap();
    let diagnostics = public.as_object().unwrap()["diagnostics"]
        .as_array()
        .unwrap();
    assert_eq!(diagnostics.len(), 512);
    assert_eq!(
        diagnostics.last().unwrap().as_object().unwrap()["message"].as_str(),
        Some("2 normalized issues omitted")
    );
}

fn lowered_fixture() -> (
    rust2vir_internal::driver_protocol::DriverRequest,
    rust2vir_internal::driver_protocol::DriverOutput,
    LowerRequest,
) {
    let vector = vector();
    let root = vector.as_object().unwrap();
    let request = parse_request_transport(&transport(&root["valid_request"])).unwrap();
    let output =
        parse_output_transport(&transport(&root["valid_lowered"]), &request, 0, false).unwrap();
    let lower = lower_request(&request);
    (request, output, lower)
}

fn lower_request(request: &rust2vir_internal::driver_protocol::DriverRequest) -> LowerRequest {
    let root = request.value().as_object().unwrap();
    let selection = root["selection"].as_object().unwrap()["value"]
        .as_object()
        .unwrap();
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

fn vector() -> JsonValue {
    json::parse(VECTOR, VECTOR.len()).unwrap()
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().unwrap()["value"].clone();
    let mut bytes = json::canonical(&value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn assert_self_hash(value: &JsonValue, field: &str, domain: &[u8]) {
    let claimed = value.as_object().unwrap()[field]
        .as_str()
        .unwrap()
        .to_owned();
    let mut preimage = value.clone();
    preimage.as_object_mut().unwrap().remove(field);
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(&json::canonical(&preimage).unwrap());
    assert_eq!(claimed, hex(&hasher.finish()));
}
