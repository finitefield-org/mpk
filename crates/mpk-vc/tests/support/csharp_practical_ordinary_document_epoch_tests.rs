//! Historical source IDs select archived definitions, never current evidence.
use mpk_vc::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, parse_canonical_practical_json, PracticalArtifactKind,
    PracticalJsonValue,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, sync::OnceLock};

const PAIRS: [(&str, &str); 3] = [
    (
        "document-41dc96a4a1a5d49fa3ad6bb9e9408db0403bb5ac9ac63f7b6b6977745f0375f6",
        "document-7a157feb27ed416752b5dde8aa2bf373b9a2c82755d2caa89e955057176ee57b",
    ),
    (
        "document-7d781ce1b08e716daf1b4a733544c5a6750b27a35b938ca49d4c5b77c0619f19",
        "document-decb5507362a7672c711a5befd63efc1038c2c0a86ee20137a1c4c472e0d1684",
    ),
    (
        "document-265f119153aca465971812118cea36c2be23c80779917cc99817c9fa1f972edf",
        "document-6b96fc4eed95a1d33fa5784b9062f02fa07ac65b6aef2b4100bb91e9c2f1a613",
    ),
];

fn inputs() -> [Value; 4] {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    [
        ("develop/migrations/go-practical/t01-w05/retained-csharp/json-boundary-epochs/source-requests.json", "70146f8048068f5c3674e048133c933f5d687963227f9f1e52b50ca4a75f89c3"),
        ("develop/migrations/csharp-03/boundary-output/source-requests.json", "5109bc2080b918dd5142c87ad674472f5febf9d6754d74ec8f80dd0fa3bac415"),
        ("develop/migrations/go-practical/t01-w05/retained-csharp/json-boundary-epochs/source-responses.json", "d3fffb239e9fe2d4dd287d3ca974f8d28e47a1f36a09a30c12ee48cc8a6a4260"),
        ("develop/migrations/csharp-03/boundary-output/source-responses.json", "1fdc9ee087f9be53df58a0c6d36f5f2e95e0c36103f2124ed2198eed9c9bf5b8"),
    ].map(|(path, expected)| {
        let bytes = fs::read(root.join(path)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected, "{path}");
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 3);
        value
    })
}

fn row<'a>(rows: &'a Value, id: &str) -> &'a Value {
    let id = id.strip_prefix("document-").unwrap();
    let matches = rows
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["id"] == id)
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    matches[0]
}

fn request_body(request: &Value) -> Value {
    let mut value = request.clone();
    value.as_object_mut().unwrap().remove("id").unwrap();
    let mut sidecars = 0;
    for input in value["inputs"].as_array_mut().unwrap() {
        if input["kind"] == "sidecar" {
            let mut body = sidecar(input);
            let PracticalJsonValue::Object(fields) = &mut body else {
                panic!("sidecar must be an object");
            };
            let before = fields.len();
            fields.retain(|(name, _)| {
                !["semantic_context", "contract_sha256"].contains(&name.as_str())
            });
            assert_eq!(before - fields.len(), 2);
            set_sidecar(input, &body);
            sidecars += 1;
        }
    }
    assert_eq!(sidecars, 2);
    value
}

fn sidecar(input: &Value) -> PracticalJsonValue {
    parse_canonical_practical_json(
        PracticalArtifactKind::SourceArtifacts,
        input["utf8"].as_str().unwrap().as_bytes(),
    )
    .unwrap()
}

fn set_sidecar(input: &mut Value, body: &PracticalJsonValue) {
    input["utf8"] =
        Value::String(String::from_utf8(canonical_practical_json_bytes(body).unwrap()).unwrap());
}

fn replace_lone_surrogate(value: &mut PracticalJsonValue) -> bool {
    match value {
        PracticalJsonValue::Utf16String(_) => {
            *value = PracticalJsonValue::String("\u{fffd}".into());
            true
        }
        PracticalJsonValue::Array(values) => values.iter_mut().any(replace_lone_surrogate),
        PracticalJsonValue::Object(fields) => fields
            .iter_mut()
            .any(|(_, value)| replace_lone_surrogate(value)),
        _ => false,
    }
}

fn response_body(response: &Value, request: &Value) -> Value {
    let mut value = response.clone();
    value.as_object_mut().unwrap().remove("id").unwrap();
    let files = value["facts"]["input_files"].as_array_mut().unwrap();
    let inputs = request["inputs"].as_array().unwrap();
    assert_eq!(files.len(), inputs.len());
    for (file, input) in files.iter_mut().zip(inputs) {
        assert_eq!(file["kind"], input["kind"]);
        assert_eq!(file["path"], input["path"]);
        let bytes = input["utf8"].as_str().unwrap().as_bytes();
        assert_eq!(file["size_bytes"], bytes.len());
        assert_eq!(file["raw_sha256"], format!("{:x}", Sha256::digest(bytes)));
        if file["kind"] == "sidecar" {
            file.as_object_mut().unwrap().remove("raw_sha256").unwrap();
        }
    }
    value
}

fn verify() {
    static VERIFIED: OnceLock<()> = OnceLock::new();
    VERIFIED.get_or_init(|| {
        let [old_requests, current_requests, old_responses, current_responses] = inputs();
        for (current, old) in PAIRS {
            let left = row(&old_requests, old);
            let right = row(&current_requests, current);
            assert_eq!(request_body(left), request_body(right));
            assert_eq!(
                response_body(row(&old_responses, old), left),
                response_body(row(&current_responses, current), right),
            );
        }
    });
}

pub(super) fn previous_document_id(id: &str) -> &'static str {
    verify();
    PAIRS
        .iter()
        .find(|(current, _)| *current == id)
        .map(|(_, old)| *old)
        .unwrap_or_else(|| panic!("unrecorded current document source {id}"))
}

#[test]
fn retained_document_sources_and_native_facts_preserve_both_epochs() {
    verify();
}

#[test]
fn retained_document_source_correspondence_rejects_changed_bodies() {
    let [old_requests, current_requests, _, _] = inputs();
    let mut surrogate_controls = 0;
    for (current, old) in PAIRS {
        let left = request_body(row(&old_requests, old));
        let right = row(&current_requests, current);
        let mut changed = right.clone();
        changed["inputs"][2]["utf8"] = Value::String(format!(
            "{}// altered\n",
            changed["inputs"][2]["utf8"].as_str().unwrap()
        ));
        assert_ne!(left, request_body(&changed));
        let mut changed = right.clone();
        let mut contract = sidecar(&changed["inputs"][1]);
        let PracticalJsonValue::Object(fields) = &mut contract else {
            unreachable!()
        };
        fields
            .iter_mut()
            .find(|(name, _)| name == "termination")
            .unwrap()
            .1 = PracticalJsonValue::String("altered".into());
        set_sidecar(&mut changed["inputs"][1], &contract);
        assert_ne!(left, request_body(&changed));
        let mut changed = right.clone();
        changed["roots"][0] = Value::String("altered".into());
        assert_ne!(left, request_body(&changed));
        for at in 0..2 {
            let mut changed = right.clone();
            let mut payload = sidecar(&changed["inputs"][at]);
            if replace_lone_surrogate(&mut payload) {
                set_sidecar(&mut changed["inputs"][at], &payload);
                assert_ne!(left, request_body(&changed));
                surrogate_controls += 1;
            }
        }
    }
    assert!(surrogate_controls > 0);
    eprintln!(
        "retained source controls: 9 source/contract/root changes and {surrogate_controls} lone-surrogate substitutions rejected"
    );
}
