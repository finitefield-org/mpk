#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;
#[path = "support/rustc_harness.rs"]
mod rustc_harness;

use rust2vir_internal::cli::{LowerRequest, ReleaseArguments, RustSelection, RustTarget};
use rust2vir_internal::driver_protocol::{parse_output_transport, parse_request_transport};
use rust2vir_internal::emit::driver_non_success_envelope;
use rust2vir_internal::json::{self, JsonValue};
use std::collections::BTreeSet;
use std::path::PathBuf;

const SUBSET_VECTOR: &[u8] = include_bytes!("../testdata/rust-subset-v0.json");
const DRIVER_VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");

#[test]
fn normative_rust_subset_rejections_have_exact_outcomes() {
    let vector = parse(SUBSET_VECTOR);
    let cases = vector.as_object().unwrap()["rejected_cases"]
        .as_array()
        .unwrap();
    let mut ids = BTreeSet::new();
    let mut executed = 0_usize;

    for case in cases {
        let case = case.as_object().unwrap();
        let id = text(case, "id");
        assert!(ids.insert(id), "duplicate normative rejection {id}");
        let expect = case["expect"].as_object().unwrap();
        let expected = (
            text(expect, "status"),
            text(expect, "phase"),
            text(expect, "code"),
        );
        assert_frozen_outcome(expected, id);

        let Some(source) = case.get("source").and_then(JsonValue::as_str) else {
            continue;
        };
        if !matches!(text(case, "stage"), "source" | "subset") {
            continue;
        }
        let error = rustc_harness::analyze(source.as_bytes(), "vector::f")
            .expect_err("negative source must not enter the accepted subset");
        assert_eq!(driver_error_outcome(&error), expected, "vector case {id}");
        executed += 1;
    }

    assert_eq!(ids.len(), 73, "the v0 rejection inventory is frozen");
    assert_eq!(
        executed, 24,
        "source/subset corpus unexpectedly changed executable fixtures"
    );
}

#[test]
fn mixed_failures_stop_at_the_frozen_primary_and_emit_no_artifacts() {
    use rust2vir_internal::source_gate::{validate_source, SourceGateCode, SourceRole};

    let cases = [
        (
            b"macro_rules! m {()=>{1}} pub fn f( -> u8 { m!() }".as_slice(),
            SourceGateCode::SourceParse,
        ),
        (
            b"m!(); #[cfg(unix)] pub fn f(x:u8)->u8{x}".as_slice(),
            SourceGateCode::SubsetCfg,
        ),
        (
            b"#[inline] use core::cmp; pub fn f(x:u8)->u8{x}".as_slice(),
            SourceGateCode::SubsetAttribute,
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(
            validate_source(source, SourceRole::CrateRoot).map_err(|error| error.code),
            Err(expected)
        );
    }

    let vector = parse(DRIVER_VECTOR);
    let root = vector.as_object().unwrap();
    let request = parse_request_transport(&transport(&root["valid_request"])).unwrap();
    let lower = lower_request(&request);
    for fixture in root["non_success"].as_array().unwrap() {
        let fixture = fixture.as_object().unwrap();
        let exit = integer(fixture, "exit") as i32;
        let output = parse_output_transport(
            &transport(&JsonValue::Object(fixture.clone())),
            &request,
            exit,
            false,
        )
        .unwrap();
        let public = driver_non_success_envelope(&lower, &output).unwrap();
        let public = parse(&public[..public.len() - 1]);
        let public = public.as_object().unwrap();
        assert_eq!(output.status().exit_code(), exit);
        for forbidden in ["ir", "source_map", "source_manifest"] {
            assert!(
                !public.contains_key(forbidden),
                "non-success result leaked {forbidden}"
            );
        }
    }
}

#[test]
fn normative_negative_and_adversarial_catalogs_are_closed_and_complete() {
    let vector = parse(SUBSET_VECTOR);
    let root = vector.as_object().unwrap();
    assert_eq!(
        root.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "accepted_cases",
            "case_harness",
            "compiler",
            "limit_boundaries",
            "mir_adapter_patterns",
            "owner_test",
            "profiles",
            "rejected_cases",
            "rustc_wrapper_argv",
            "same_phase_precedence",
            "schema",
            "spec_schema",
            "target_cfg",
        ])
    );
    assert_eq!(text(root, "schema"), "mpk.rust.subset.conformance.v0");
    assert_eq!(root["rejected_cases"].as_array().unwrap().len(), 73);
    assert_eq!(root["same_phase_precedence"].as_array().unwrap().len(), 5);
    assert_eq!(root["limit_boundaries"].as_array().unwrap().len(), 35);

    let mut ids = BTreeSet::new();
    for group in [
        "rejected_cases",
        "same_phase_precedence",
        "limit_boundaries",
    ] {
        for case in root[group].as_array().unwrap() {
            let id = text(case.as_object().unwrap(), "id");
            assert!(ids.insert(id), "duplicate negative catalog ID {id}");
        }
    }
    assert_eq!(ids.len(), 113);
}

#[test]
fn adversarial_driver_results_are_status_exact_and_artifact_free() {
    let vector = parse(DRIVER_VECTOR);
    let root = vector.as_object().unwrap();
    let request = parse_request_transport(&transport(&root["valid_request"])).unwrap();
    let mut statuses = BTreeSet::new();
    for fixture in root["non_success"].as_array().unwrap() {
        let fixture = fixture.as_object().unwrap();
        let exit = integer(fixture, "exit") as i32;
        let output = parse_output_transport(
            &transport(&JsonValue::Object(fixture.clone())),
            &request,
            exit,
            false,
        )
        .unwrap();
        assert_eq!(output.status().exit_code(), exit);
        statuses.insert(output.status().as_str());
        let output = output.value().as_object().unwrap();
        for forbidden in [
            "payload_hash",
            "raw_lowering",
            "raw_source_map",
            "source_inventory",
        ] {
            assert!(!output.contains_key(forbidden));
        }
    }
    assert_eq!(
        statuses,
        BTreeSet::from(["frontend-error", "rejected", "source-error"])
    );
}

fn driver_error_outcome(error: &rustc_driver_adapter::RustcDriverError) -> (&str, &str, &str) {
    use rust2vir_internal::file_loader::SourceLoaderStatus;
    use rustc_driver_adapter::RustcDriverError;

    match error {
        RustcDriverError::Source(error) => (
            match error.code.status() {
                SourceLoaderStatus::Rejected => "rejected",
                SourceLoaderStatus::SourceError => "source-error",
                SourceLoaderStatus::FrontendError => "frontend-error",
            },
            error.code.phase(),
            error.code.as_str(),
        ),
        RustcDriverError::Subset(code) => ("rejected", "subset", code.as_str()),
        RustcDriverError::Contract(error) => ("rejected", "subset", error.code.as_str()),
        RustcDriverError::Mir(error) if error.is_frontend_error() => {
            ("frontend-error", "emission", error.code.as_str())
        }
        RustcDriverError::Mir(error) => ("rejected", "lowering", error.code.as_str()),
        RustcDriverError::Session => ("frontend-error", "typecheck", "RUST_TOOLCHAIN_OPTIONS"),
        RustcDriverError::MirAdapter => {
            ("frontend-error", "lowering", "RUST_TOOLCHAIN_MIR_ADAPTER")
        }
        RustcDriverError::Compiler => ("source-error", "typecheck", "RUST_SOURCE_TYPE"),
    }
}

fn assert_frozen_outcome((status, phase, code): (&str, &str, &str), id: &str) {
    assert!(
        matches!(status, "rejected" | "source-error" | "frontend-error"),
        "invalid status for {id}"
    );
    assert!(
        matches!(
            phase,
            "capture" | "source" | "metadata" | "typecheck" | "subset" | "lowering" | "emission"
        ),
        "invalid phase for {id}"
    );
    assert!(code.starts_with("RUST_"), "invalid stable code for {id}");
    assert!(
        code.bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'),
        "invalid stable code grammar for {id}"
    );
}

fn parse(bytes: &[u8]) -> JsonValue {
    json::parse(bytes, bytes.len()).expect("strict JSON fixture")
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().unwrap()["value"].clone();
    let mut bytes = json::canonical(&value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn text<'a>(object: &'a std::collections::BTreeMap<String, JsonValue>, key: &str) -> &'a str {
    object[key].as_str().unwrap()
}

fn integer(object: &std::collections::BTreeMap<String, JsonValue>, key: &str) -> i64 {
    object[key].integer().unwrap()
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
            package: text(selection, "package").to_owned(),
            crate_name: text(selection, "crate").to_owned(),
            kind: "lib",
            function: text(selection, "function").to_owned(),
        },
        semantic_profile: "mpk.rust.checked.v0",
        target: RustTarget::X86_64UnknownLinuxGnu,
        release: ReleaseArguments {
            frontend_bundle_id: text(frontend, "bundle_id").to_owned(),
            frontend_sha256: text(frontend, "binary_sha256").to_owned(),
            release_registry_id: text(registry, "id").to_owned(),
            release_registry_sha256: text(registry, "registry_sha256").to_owned(),
            toolchain_bundle_id: text(toolchain, "bundle_id").to_owned(),
            toolchain_root: PathBuf::from("/toolchain-not-emitted"),
            toolchain_distribution_sha256: text(toolchain, "distribution_sha256").to_owned(),
            driver: PathBuf::from("/driver-not-emitted"),
            driver_sha256: "0".repeat(64),
        },
        contracts: vec!["contracts/vector.json".to_owned()],
    }
}
