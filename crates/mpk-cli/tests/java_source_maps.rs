//! T06: original UTF-8 origins, exact captured inputs, and cross-artifact identities.

#[path = "support/java_lowering.rs"]
mod harness;

use serde_json::{json, Value};

#[test]
fn all_original_source_boundary_vectors_have_private_fixtures() {
    harness::check_fixtures();
    let profile = harness::profile();
    assert_eq!(profile["source_map_cases"].as_array().unwrap().len(), 7);
    assert_eq!(
        profile["source_map_cases"][3]["expected_utf8_range"],
        json!([8, 9])
    );
    assert_eq!(
        profile["source_map_cases"][4]["expected_code"],
        "JAVA_SOURCE_MAP_UTF16"
    );
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn pinned_maps_cover_every_node_with_exact_original_byte_origins_and_no_partial_failure() {
    let report = harness::run();
    let case = harness::case(report, "extra/map-unicode");
    let envelope = harness::envelope(case);
    let inputs = harness::captured(case);
    let source = inputs
        .iter()
        .find(|input| input.normalized_path.ends_with(".java"))
        .unwrap()
        .bytes;
    let entries = envelope["source_map"]["entries"].as_array().unwrap();
    let instructions = envelope["ir"]["value"]["units"][0]["functions"][0]["blocks"][0]
        ["instructions"]
        .as_array()
        .unwrap();
    let spelling = |origin: &Value| {
        let first = origin["start"].as_u64().unwrap() as usize;
        let last = origin["end"].as_u64().unwrap() as usize;
        std::str::from_utf8(&source[first..last]).unwrap()
    };
    for instruction in instructions {
        let entry = entries
            .iter()
            .find(|entry| entry["reference"]["instruction"] == instruction["id"])
            .unwrap();
        let expected = if instruction["kind"] == "Copy" {
            if instruction["id"] == "t0" {
                "int y = x;"
            } else {
                "y = (y + 1) >>> k;"
            }
        } else if instruction["op"] == "bv_add" {
            "y + 1"
        } else if instruction["value"]
            .get("int")
            .is_some_and(|value| value["value"] == "1")
        {
            "1"
        } else {
            "(y + 1) >>> k"
        };
        assert_eq!(spelling(&entry["origin"]), expected);
    }
    let returned = entries
        .iter()
        .find(|entry| entry["reference"]["kind"] == "terminator")
        .unwrap();
    assert_eq!(spelling(&returned["origin"]), "return y;");
    let failed = report["mutations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "precedence/map_failure_prevents_partial_output")
        .unwrap();
    assert_eq!(failed["code"], "JAVA_SOURCE_MAP_RANGE");
    assert_eq!(failed["published_bytes"], 0);
}

#[test]
#[ignore = "requires the provisioned pinned JDK cache and local Linux amd64 image; runs offline"]
fn shared_validators_reject_rehashed_map_and_manifest_linkage_mutations() {
    let report = harness::run();
    let case = harness::case(report, "extra/map-unicode");
    let original = harness::envelope(case);
    let request = harness::Request::new(&original["selection"]);
    let inputs = harness::captured(case);
    for mutation in 0..8 {
        let mut value = original.clone();
        match mutation {
            0 => {
                value["source_map"]["entries"].as_array_mut().unwrap().pop();
            }
            1 => {
                value["source_map"]["entries"][0]["origin"]["normalized_path"] =
                    json!("/private/source.java")
            }
            2 => value["source_map"]["entries"][0]["origin"]["end"] = json!(999999),
            3 => {
                let source = inputs
                    .iter()
                    .find(|input| input.normalized_path.ends_with(".java"))
                    .unwrap()
                    .bytes;
                let first = source
                    .windows("😀".len())
                    .position(|window| window == "😀".as_bytes())
                    .unwrap()
                    + 1;
                value["source_map"]["entries"][0]["origin"]["start"] = json!(first);
            }
            4 => {
                value["source_manifest"]["release_registry"]["registry_sha256"] =
                    json!("f".repeat(64))
            }
            5 => value["source_manifest"]["inputs"][0]["sha256"] = json!("f".repeat(64)),
            6 => value["source_manifest"]["selection"]["value"]["compilation"] = json!("different"),
            _ => value["source_manifest"]["vc_hash"] = json!("f".repeat(64)),
        }
        harness::refresh(&mut value);
        assert!(
            request
                .validate(&harness::canonical_line(&value), 0, &inputs)
                .is_err(),
            "mutation {mutation}"
        );
    }
    for kind in ["source", "contract"] {
        let mut changed = case.clone();
        let input = changed["captured_inputs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|input| input["kind"] == kind)
            .unwrap();
        input["text"] = format!("{}\n", input["text"].as_str().unwrap()).into();
        assert!(
            request
                .validate(
                    &harness::canonical_line(&original),
                    0,
                    &harness::captured(&changed)
                )
                .is_err(),
            "raw {kind} bytes"
        );
    }
    let mut extra_lf = harness::canonical_line(&original);
    extra_lf.push(b'\n');
    assert!(request.validate(&extra_lf, 0, &inputs).is_err());
}
