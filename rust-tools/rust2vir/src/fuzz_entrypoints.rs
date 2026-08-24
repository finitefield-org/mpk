//! Bounded parser-only entry points for the isolated nightly fuzz package.

use std::sync::OnceLock;

use crate::contract::parse_contract_for_fuzz;
use crate::driver_protocol::{
    parse_output_artifact, parse_output_transport, parse_request_transport, DriverOutput,
    DriverProtocolError, DriverRequest, REQUEST_TRANSPORT_MAX,
};
use crate::json::{self, JsonValue};

const DRIVER_VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");

/// Exercises the private request, output-stream, and result-artifact parsers
/// without starting a compiler, reading a path, or retaining arbitrary bytes.
pub fn exercise_driver_protocol(input: &[u8]) {
    let input = &input[..input.len().min(REQUEST_TRANSPORT_MAX)];

    let first_request = parse_request_transport(input);
    let second_request = parse_request_transport(input);
    assert_eq!(
        request_signature(&first_request),
        request_signature(&second_request)
    );

    let request = fixed_request();
    for exit_code in [0, 1, 3, 4, 64] {
        let first = parse_output_transport(input, request, exit_code, false);
        let second = parse_output_transport(input, request, exit_code, false);
        assert_eq!(output_signature(&first), output_signature(&second));
    }
    let first_signaled = parse_output_transport(input, request, 1, true);
    let second_signaled = parse_output_transport(input, request, 1, true);
    assert_eq!(
        output_signature(&first_signaled),
        output_signature(&second_signaled)
    );

    let first_artifact = parse_output_artifact(input, request);
    let second_artifact = parse_output_artifact(input, request);
    assert_eq!(
        output_signature(&first_artifact),
        output_signature(&second_artifact)
    );
}

/// Exercises strict contract parsing with the parser's own byte, node, and
/// depth limits. No path supplied by the fuzz input is consulted.
pub fn exercise_rust_contract(input: &[u8]) {
    let first = parse_contract_for_fuzz(input);
    let second = parse_contract_for_fuzz(input);
    assert_eq!(first, second);
}

fn fixed_request() -> &'static DriverRequest {
    static REQUEST: OnceLock<DriverRequest> = OnceLock::new();
    REQUEST.get_or_init(|| {
        let vector = json::parse(DRIVER_VECTOR, DRIVER_VECTOR.len())
            .expect("tracked driver vector is strict JSON");
        let value = vector
            .as_object()
            .and_then(|root| root.get("valid_request"))
            .and_then(JsonValue::as_object)
            .and_then(|fixture| fixture.get("value"))
            .expect("tracked driver vector has a valid request");
        let mut transport = json::canonical(value).expect("valid request canonicalizes");
        transport.push(b'\n');
        parse_request_transport(&transport).expect("tracked request validates")
    })
}

fn request_signature(
    result: &Result<DriverRequest, DriverProtocolError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(request) => (
            true,
            request.request_fingerprint().to_owned(),
            request.source_inventory_hash().to_owned(),
            request.transport().len(),
        ),
        Err(error) => (false, error.code.as_str().to_owned(), String::new(), 0),
    }
}

fn output_signature(
    result: &Result<DriverOutput, DriverProtocolError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(output) => (
            true,
            output.status().as_str().to_owned(),
            output.phase().to_owned(),
            output.transport().len(),
        ),
        Err(error) => (false, error.code.as_str().to_owned(), String::new(), 0),
    }
}
