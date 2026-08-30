use rust2vir_internal::driver_process::{classify_invocation, WrapperInvocation};
use rust2vir_internal::driver_protocol::{parse_request_transport, DriverRequest};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::session::{target_cfg, EffectiveSession, SessionField};
use std::collections::BTreeMap;

const VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");
type SessionMutation = (SessionField, fn(&mut EffectiveSession));

#[test]
fn exact_effective_session_is_required_field_by_field() {
    let request = request();
    let accepted = effective_session(&request);
    assert_eq!(accepted.validate(&request), Ok(()));

    let mutations: &[SessionMutation] = &[
        (SessionField::Edition, |value| {
            value.edition = "2024".to_owned()
        }),
        (SessionField::Target, |value| {
            value.target_id = "i686-unknown-linux-gnu".to_owned()
        }),
        (SessionField::PointerWidth, |value| value.pointer_width = 32),
        (SessionField::PanicStrategy, |value| {
            value.panic_strategy = "unwind".to_owned()
        }),
        (SessionField::OverflowChecks, |value| {
            value.overflow_checks = false
        }),
        (SessionField::DebugAssertions, |value| {
            value.debug_assertions = true
        }),
        (SessionField::RustcOptLevel, |value| {
            value.rustc_opt_level = 1
        }),
        (SessionField::MirOptLevel, |value| value.mir_opt_level = 1),
        (SessionField::Features, |value| {
            value.enabled_features.push("hostile".to_owned())
        }),
        (SessionField::Cfg, |value| value.cfg.swap(0, 1)),
        (SessionField::Cfg, |value| value.cfg.push("unix".to_owned())),
    ];
    for (field, mutate) in mutations {
        let mut changed = accepted.clone();
        mutate(&mut changed);
        assert_eq!(changed.validate(&request).unwrap_err().field, *field);
    }
}

#[test]
fn both_target_cfg_sets_are_complete_sorted_and_unique() {
    for (target, width) in [
        ("i686-unknown-linux-gnu", "target_pointer_width=\"32\""),
        ("x86_64-unknown-linux-gnu", "target_pointer_width=\"64\""),
    ] {
        let values = target_cfg(target).unwrap();
        assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(values.contains(&"overflow_checks"));
        assert!(values.contains(&"panic=\"abort\""));
        assert!(values.contains(&width));
    }
    assert!(target_cfg("x86_64-apple-darwin").is_none());
}

#[test]
fn exact_primary_arguments_reject_every_unapproved_lint_control() {
    let request = request();
    let primary = primary_argv();
    assert_eq!(
        classify_invocation(&request, &primary),
        Ok(WrapperInvocation::Primary)
    );
    for arguments in [
        vec!["-Adead_code"],
        vec!["-W", "unused"],
        vec!["-Dwarnings"],
        vec!["-F", "unsafe_code"],
        vec!["--cap-lints", "allow"],
    ] {
        let mut changed = primary.clone();
        changed.extend(arguments.into_iter().map(str::to_owned));
        assert_eq!(
            classify_invocation(&request, &changed),
            Ok(WrapperInvocation::PrimaryArgumentMismatch)
        );
    }
}

fn effective_session(request: &DriverRequest) -> EffectiveSession {
    EffectiveSession {
        edition: "2021".to_owned(),
        target_id: request.target().to_owned(),
        pointer_width: request.pointer_width(),
        panic_strategy: "abort".to_owned(),
        overflow_checks: true,
        debug_assertions: false,
        rustc_opt_level: 0,
        mir_opt_level: 0,
        enabled_features: Vec::new(),
        cfg: target_cfg(request.target())
            .unwrap()
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

fn request() -> DriverRequest {
    let vector = json::parse(VECTOR, VECTOR.len()).unwrap();
    let fixture = vector.as_object().unwrap()["valid_request"]
        .as_object()
        .unwrap();
    parse_request_transport(&transport(fixture)).unwrap()
}

fn transport(fixture: &BTreeMap<String, JsonValue>) -> Vec<u8> {
    decode_base64(
        fixture["transport"].as_object().unwrap()["base64"]
            .as_str()
            .unwrap(),
    )
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
