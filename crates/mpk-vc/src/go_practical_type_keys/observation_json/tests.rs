use super::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/observation-json")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(name)).unwrap()).unwrap()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_OBSERVATION_JSON_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    schema: String,
    metadata_hex: String,
    diagnostics_hex: String,
    original_bytes: u64,
    content_sha256: String,
}
fn domain(schema: &str) -> HashDomain {
    match schema {
        "mpk.go.practical.source_export_inputs.v1" => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0")
        }
        "mpk.go.practical.source_module_observations.v1" => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0")
        }
        "mpk.go.practical.source_module_graph_inputs.v2" => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0")
        }
        _ => panic!("unknown original schema"),
    }
}
#[test]
fn original_go_observation_byte_boundaries_keep_same_bytes_and_hashes() {
    let directory = root().join("native/native-originals/observation-json");
    let corpus =
        read("native/native-originals/observation-json/complete-observation-json-budgets.json");
    let rows = corpus["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 30);
    let mut observed = vec![];
    for row in rows {
        let raw = std::fs::read(directory.join(row["canonical_file"].as_str().unwrap())).unwrap();
        assert_eq!(
            sha256_raw_file_bytes(&raw).to_hex(),
            row["canonical_sha256"]
        );
        let original =
            std::fs::read(directory.join(row["original_file"].as_str().unwrap())).unwrap();
        assert_eq!(sha256_raw_file_bytes(&original).to_hex(), row["raw_sha256"]);
        let wire: Probe = decode(&raw).unwrap();
        assert_eq!(wire.original_bytes as usize, original.len());
        assert_eq!(
            identity(&wire, domain(&wire.schema)).unwrap(),
            wire.content_sha256
        );
        assert_eq!(wire.content_sha256, row["content_sha256"]);
        let encoded = if row["field"] == "metadata_hex" {
            &wire.metadata_hex
        } else {
            &wire.diagnostics_hex
        };
        let restored = encoded
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(original, restored);
        if encoded.len() as u64 > limits().max_string_bytes {
            assert!(super::super::decode::<Probe>(&raw).is_err());
        }
        assert_eq!(canonical_json_bytes(&parse(&raw).unwrap()).unwrap(), raw);
        assert!(identity(&wire, HashDomain::new("MPK-CALLER-OBSERVATION-1.0")).is_err());
        observed.push(json!({"case":row["case"],"original_bytes":original.len(),"raw_sha256":row["raw_sha256"],"canonical_sha256":row["canonical_sha256"],"content_sha256":wire.content_sha256,"same_original_bytes":true}));
    }
    output(
        "original-byte-boundary-comparison.json",
        &json!({"status":"passed","cases":observed,"shared_json_limits_unchanged":true,"native_stdout_bytes":STDOUT_BYTES,"native_stderr_bytes":STDERR_BYTES,"source_or_context_authority":false}),
    );
}
#[test]
fn scoped_observation_parser_retains_common_rules_and_raw_limits() {
    let directory = root().join("native/native-originals/observation-json-controls");
    let corpus = read(
        "native/native-originals/observation-json-controls/complete-observation-json-controls.json",
    );
    let mut observed = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        if row["serialized_input_retained"] != true {
            continue;
        }
        let raw =
            std::fs::read(directory.join(row["complete_input_file"].as_str().unwrap())).unwrap();
        let name = row["case"].as_str().unwrap();
        if matches!(name, "ordinary-over" | "nested-metadata-over") {
            // Preserve each predecessor's ordinary limit: Go is 1MiB, this
            // Rust private source family was already 16MiB. Neither changes.
            assert!(parse(&raw).is_ok());
            assert!(parse_strict_json(&raw, limits()).is_ok());
        } else {
            assert!(parse(&raw).is_err(), "{name}");
        }
        observed.push(json!({"case":name,"bytes":raw.len(),"raw_sha256":sha256_raw_file_bytes(&raw).to_hex(),"rejected":!matches!(name,"ordinary-over"|"nested-metadata-over")}));
    }
    let base = json!({"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"","diagnostics_hex":""});
    for nested in [false, true] {
        let mut over = base.clone();
        let value = Value::String("a".repeat(limits().max_string_bytes as usize + 1));
        over["ordinary"] = if nested {
            json!({"metadata_hex":value})
        } else {
            value
        };
        let raw = serde_json::to_vec(&over).unwrap();
        assert!(parse(&raw).is_err());
    }
    for missing in ["metadata_hex", "diagnostics_hex"] {
        let mut value = base.clone();
        value.as_object_mut().unwrap().remove(missing);
        assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut wrong = base.clone();
    wrong["metadata_hex"] = json!(false);
    assert!(parse(&serde_json::to_vec(&wrong).unwrap()).is_err());
    assert_eq!(
        limits(),
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216)
    );
    output(
        "original-byte-control-comparison.json",
        &json!({"status":"passed","cases":observed,"ordinary_rust_limit_unchanged":limits().max_string_bytes,"ordinary_go_limit_unchanged":1048576,"nested_root_name_does_not_widen_limit":true}),
    );
}
