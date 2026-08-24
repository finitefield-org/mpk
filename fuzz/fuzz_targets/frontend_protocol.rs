#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mpk_cli::frontend_protocol::{
    validate_frontend_process, AcceptedFrontendEnvelope, FrontendProcessFacts,
    FrontendProtocolError, FrontendProtocolRequest,
};
use mpk_vc::{CapturedInput, InputKind};
use serde_json::{json, Value};

const MAX_FUZZ_INPUT: usize = 1_048_576;
const CAPTURED_INPUTS: &[CapturedInput<'static>] = &[CapturedInput {
    kind: InputKind::Source,
    normalized_path: "src/lib.rs",
    bytes: b"pub fn identity(value: i8) -> i8 { value }\n",
}];

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    for exit_code in [0, 1, 3, 4, 64] {
        exercise(data, Some(exit_code), false);
    }
    exercise(data, None, true);
});

fn exercise(data: &[u8], exit_code: Option<i32>, signaled: bool) {
    let request = FrontendProtocolRequest {
        source_language: "rust",
        semantic_profile: "mpk.rust.checked.v0",
        semantic_parameters: semantic_parameters(),
        selection: selection(),
        release_registry: None,
        captured_inputs: CAPTURED_INPUTS,
    };
    let process = FrontendProcessFacts {
        exit_code,
        signaled,
        stdout: data,
        stderr_observed_bytes: 0,
    };
    let first = validate_frontend_process(request, process);
    let second = validate_frontend_process(request, process);
    assert_eq!(signature(&first), signature(&second));
    if let Ok(accepted) = first {
        assert_eq!(accepted.canonical_bytes, data);
    }
}

fn semantic_parameters() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        json!({
            "target_id": "x86_64-unknown-linux-gnu",
            "pointer_width": 64,
            "overflow_mode": "checked",
            "panic_mode": "abort"
        })
    })
}

fn selection() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        json!({
            "package": "fuzz",
            "crate": "fuzz",
            "kind": "lib",
            "function": "fuzz::identity"
        })
    })
}

fn signature(
    result: &Result<AcceptedFrontendEnvelope, FrontendProtocolError>,
) -> (bool, String, String, usize, bool) {
    match result {
        Ok(value) => (
            true,
            value.status.clone(),
            value.phase.clone(),
            value.canonical_bytes.len(),
            value.artifacts.is_some(),
        ),
        Err(error) => (
            false,
            error.code().as_str().to_owned(),
            String::new(),
            0,
            false,
        ),
    }
}
