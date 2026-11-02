use mpk_vc::go_practical_context::GoPrivateCandidateRegistry;
use mpk_vc::{
    hash_canonical_json, parse_strict_json, sha256_raw_file_bytes, HashDomain, StrictJsonLimits,
    StrictJsonValue,
};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

const LIMITS: StrictJsonLimits = StrictJsonLimits::new(4_194_304, 1_000_000, 64, 1_000_000);

fn strict(value: &Value) -> StrictJsonValue {
    parse_strict_json(&serde_json::to_vec(value).unwrap(), LIMITS).unwrap()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn candidate_descriptor_binds_the_unchanged_frozen_recipe_and_limits() {
    let limits = include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/foundation-limits/limits.json"
    );
    let definitions = include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/foundation-limits/definitions.json"
    );
    let descriptor = include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/foundation-limits/descriptor.json"
    );
    let rules = include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/foundation-limits/semantic-rules.md"
    );
    let records: [(&[u8], &str); 3] = [
        (limits, "MPK-GO-PRACTICAL-LIMITS-1.0"),
        (definitions, "MPK-GO-FOUNDATION-DEFINITIONS-1.0"),
        (descriptor, "MPK-GO-PRACTICAL-FOUNDATION-1.0"),
    ];
    let mut parsed = Vec::new();
    for (bytes, domain) in records {
        let value = parse_strict_json(bytes, LIMITS).unwrap();
        let claimed = value.get("content_sha256").unwrap().as_str().unwrap();
        let preimage = value.clone_without_fields(&["content_sha256"]).unwrap();
        assert_eq!(
            hash_canonical_json(HashDomain::new(domain), &preimage)
                .unwrap()
                .to_hex(),
            claimed
        );
        parsed.push(serde_json::from_slice::<Value>(bytes).unwrap());
    }
    let reference = |value: &Value| json!({"schema":value["schema"], "id":value["id"], "content_sha256":value["content_sha256"]});
    assert_eq!(parsed[1]["limits_ref"], reference(&parsed[0]));
    assert_eq!(parsed[2]["limits_ref"], reference(&parsed[0]));
    assert_eq!(parsed[2]["definitions_ref"], reference(&parsed[1]));
    assert_eq!(parsed[2]["activation"], "candidate_only");
    assert_eq!(
        parsed[1]["representation"],
        "operation_recipe_inventory_draft"
    );
    let original_members: [(&str, &[u8]); 3] = [
        (
            "develop/migrations/go-practical/t01-w04/foundation-limits/definitions.json",
            definitions,
        ),
        (
            "develop/migrations/go-practical/t01-w04/foundation-limits/limits.json",
            limits,
        ),
        (
            "develop/migrations/go-practical/t01-w04/foundation-limits/semantic-rules.md",
            rules,
        ),
    ];
    let members = parsed[2]["members"].as_array().unwrap();
    assert_eq!(members.len(), original_members.len());
    for (member, (path, raw)) in members.iter().zip(original_members) {
        assert_eq!(member["path"], path);
        assert_eq!(member["size_bytes"], raw.len());
        assert_eq!(member["sha256"], sha256_raw_file_bytes(raw).to_hex());
    }
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let registry: Value = serde_json::from_slice(registry.canonical_bytes()).unwrap();
    assert_eq!(
        registry["profiles"][1]["foundation_descriptor"],
        reference(&parsed[2])
    );
}

#[test]
fn frozen_candidate_corpus_uses_go_rust_scope_and_actual_retained_policy() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generated = Command::new("python3")
        .arg("-B")
        .arg(repo.join("crates/mpk-vc/tests/support/go_practical_context_vectors.py"))
        .env("PYTHONPATH", repo.join("develop/probes/go-practical"))
        .current_dir(&repo)
        .output()
        .expect("run the frozen independent candidate generator");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let rows: Vec<Value> = serde_json::from_slice(&generated.stdout).unwrap();
    assert_eq!(rows.len(), 133);
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let mut accepted = 0;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let value = &row["value"];
        let result = match row["kind"].as_str().unwrap() {
            "registry" => registry.import_registry(&strict(value)).is_ok(),
            "context" => registry.import_context(&strict(value)).is_ok(),
            "request" => registry.import_request(&strict(value)).is_ok(),
            "linkage" => {
                let pair = value.as_array().unwrap();
                assert_eq!(pair.len(), 2);
                match (
                    registry.import_context(&strict(&pair[0])),
                    registry.import_context(&strict(&pair[1])),
                ) {
                    (Ok(left), Ok(right)) => left.require_same(&right).is_ok(),
                    _ => false,
                }
            }
            "wire" => parse_strict_json(&unhex(value.as_str().unwrap()), LIMITS).is_ok(),
            kind => panic!("unexpected frozen context case kind {kind}"),
        };
        let expected = row["expected_accepted"].as_bool().unwrap();
        assert_eq!(result, expected, "independent frozen case {id}");
        accepted += usize::from(result);
    }
    assert_eq!(accepted, 18);
}
