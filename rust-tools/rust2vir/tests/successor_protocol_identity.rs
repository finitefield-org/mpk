use rust2vir_internal::driver_protocol::{
    parse_output_transport, parse_request_transport, DriverStatus,
};
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::sha256::{hex, Sha256};

const PREDECESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
const SUCCESSOR: &[u8] = include_bytes!("../testdata/rust-driver-v1.json");

fn parse(bytes: &[u8]) -> JsonValue {
    json::parse(bytes, bytes.len()).expect("strict driver vector")
}

fn transport(fixture: &JsonValue) -> Vec<u8> {
    let value = fixture.as_object().expect("fixture")["value"].clone();
    canonical_transport(&value)
}

fn canonical_transport(value: &JsonValue) -> Vec<u8> {
    let mut bytes = json::canonical(value).expect("canonical fixture");
    bytes.push(b'\n');
    bytes
}

fn rehash(value: &mut JsonValue, field: &str, domain: &[u8]) {
    value.as_object_mut().expect("hashed object").remove(field);
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(&json::canonical(value).expect("canonical hash preimage"));
    value
        .as_object_mut()
        .expect("hashed object")
        .insert(field.to_owned(), JsonValue::String(hex(&hasher.finish())));
}

fn replace_schema(value: &mut JsonValue, schema: &str) {
    value
        .as_object_mut()
        .expect("schema object")
        .insert("schema".to_owned(), JsonValue::String(schema.to_owned()));
}

fn replace_nested_schema(value: &mut JsonValue, field: &str, schema: &str) {
    let nested = value
        .as_object_mut()
        .expect("result object")
        .get_mut(field)
        .expect("nested protocol artifact")
        .as_object_mut()
        .expect("nested protocol object");
    nested.insert("schema".to_owned(), JsonValue::String(schema.to_owned()));
}

#[test]
fn subordinate_protocol_accepts_only_successor_identities() {
    let old = parse(PREDECESSOR);
    let old = old.as_object().expect("predecessor vector");
    let new = parse(SUCCESSOR);
    let new = new.as_object().expect("successor vector");

    assert!(parse_request_transport(&transport(&old["valid_request"])).is_err());
    let request =
        parse_request_transport(&transport(&new["valid_request"])).expect("successor request");
    assert!(parse_output_transport(
        &transport(&old["valid_lowered"]),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .is_err());
    parse_output_transport(
        &transport(&new["valid_lowered"]),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .expect("successor result");

    let valid_request =
        new["valid_request"].as_object().expect("request envelope")["value"].clone();
    let mut predecessor_request_schema = valid_request.clone();
    replace_schema(
        &mut predecessor_request_schema,
        "mpk.rust.driver.request.v0",
    );
    rehash(
        &mut predecessor_request_schema,
        "request_fingerprint",
        b"MPK-RUST-DRIVER-REQUEST-1.0",
    );
    assert!(parse_request_transport(&canonical_transport(&predecessor_request_schema)).is_err());

    let mut predecessor_request_domain = valid_request;
    rehash(
        &mut predecessor_request_domain,
        "request_fingerprint",
        b"MPK-RUST-DRIVER-REQUEST-0.1",
    );
    assert!(parse_request_transport(&canonical_transport(&predecessor_request_domain)).is_err());

    let valid_result = new["valid_lowered"].as_object().expect("lowered envelope")["value"].clone();
    for (field, schema) in [
        (None, "mpk.rust.driver.v0"),
        (Some("raw_lowering"), "mpk.rust.driver.lowering.v0"),
        (Some("raw_source_map"), "mpk.rust.driver.raw_source_map.v0"),
    ] {
        let mut predecessor_schema = valid_result.clone();
        match field {
            Some(field) => replace_nested_schema(&mut predecessor_schema, field, schema),
            None => replace_schema(&mut predecessor_schema, schema),
        }
        rehash(
            &mut predecessor_schema,
            "payload_hash",
            b"MPK-RUST-DRIVER-PAYLOAD-1.0",
        );
        assert!(parse_output_transport(
            &canonical_transport(&predecessor_schema),
            &request,
            DriverStatus::Lowered.exit_code(),
            false,
        )
        .is_err());
    }

    let mut predecessor_payload_domain = valid_result;
    rehash(
        &mut predecessor_payload_domain,
        "payload_hash",
        b"MPK-RUST-DRIVER-PAYLOAD-0.1",
    );
    assert!(parse_output_transport(
        &canonical_transport(&predecessor_payload_domain),
        &request,
        DriverStatus::Lowered.exit_code(),
        false,
    )
    .is_err());

    let lowered_envelope = new["valid_lowered"].as_object().expect("lowered envelope");
    let lowered = lowered_envelope["value"]
        .as_object()
        .expect("lowered result");
    assert_eq!(lowered["schema"].as_str(), Some("mpk.rust.driver.v1"));
    let raw_lowering = lowered["raw_lowering"].as_object().expect("raw lowering");
    let raw_source_map = lowered["raw_source_map"]
        .as_object()
        .expect("raw source map");
    assert_eq!(
        raw_lowering["schema"].as_str(),
        Some("mpk.rust.driver.lowering.v1")
    );
    assert_eq!(
        raw_source_map["schema"].as_str(),
        Some("mpk.rust.driver.raw_source_map.v1")
    );
}
