use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::{hash_domain_separated_raw, HashDomain};
use serde_json::{json, Value};
use std::{env, fs, path::Path};

fn preimage(value: &PracticalJsonValue, field: &str) -> PracticalJsonValue {
    let entries = value.as_object().unwrap();
    assert_eq!(entries.last().unwrap().0, field);
    PracticalJsonValue::Object(entries[..entries.len() - 1].to_vec())
}

fn digest(value: &PracticalJsonValue, field: &str, domain: HashDomain) -> String {
    hash_domain_separated_raw(
        domain,
        &canonical_practical_json_bytes(&preimage(value, field)).unwrap(),
    )
    .unwrap()
    .to_hex()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 5);
    let plan: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let identity_map: Value = serde_json::from_slice(
        &fs::read(
            Path::new(&args[1])
                .parent()
                .unwrap()
                .join("candidate-identity-map.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for corpus in plan.as_array().unwrap() {
        let relative = corpus["path"].as_str().unwrap();
        let old: Value =
            serde_json::from_slice(&fs::read(Path::new(&args[2]).join(relative)).unwrap()).unwrap();
        let mut new: Value =
            serde_json::from_slice(&fs::read(Path::new(&args[3]).join(relative)).unwrap()).unwrap();
        assert_eq!(old.as_array().unwrap().len(), new.as_array().unwrap().len());
        for (row_index, (old_row, new_row)) in old
            .as_array()
            .unwrap()
            .iter()
            .zip(new.as_array_mut().unwrap())
            .enumerate()
        {
            for (old_input, new_input) in old_row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .zip(new_row["inputs"].as_array_mut().unwrap())
            {
                assert_eq!(old_input["kind"], new_input["kind"]);
                if old_input["kind"] == "source" {
                    assert_eq!(old_input, new_input);
                    continue;
                }
                let mut rebound = old_input["utf8"].as_str().unwrap().to_owned();
                for (before, after) in identity_map.as_object().unwrap() {
                    rebound = rebound.replace(before, after.as_str().unwrap());
                }
                rebound = rebound.replace("\"revision\":4", "\"revision\":5");
                let old_parsed = parse_canonical_practical_json(
                    PracticalArtifactKind::MethodContract,
                    old_input["utf8"].as_str().unwrap().as_bytes(),
                );
                let old_value = match old_parsed {
                    Ok(value) => value,
                    Err(old_error) => {
                        // Preserve the original invalid transport, including duplicate
                        // fields and noncanonical escapes. Rebind only known identities.
                        let new_error = parse_canonical_practical_json(
                            PracticalArtifactKind::MethodContract,
                            rebound.as_bytes(),
                        )
                        .unwrap_err();
                        assert_eq!(format!("{old_error:?}"), format!("{new_error:?}"));
                        new_input["utf8"] = json!(rebound);
                        println!(
                            "{}",
                            json!({"request_path":relative,"row_index":row_index,
                            "path":old_input["path"], "original_transport_error":format!("{old_error:?}"),
                            "invalid_transport_structure_preserved":true})
                        );
                        continue;
                    }
                };
                let mut new_value = parse_canonical_practical_json(
                    PracticalArtifactKind::MethodContract,
                    rebound.as_bytes(),
                )
                .unwrap();
                let schema = old_value.get("schema").unwrap().as_str().unwrap();
                let (field, domain) = match schema {
                    SEMANTIC_BINDINGS_SCHEMA => {
                        ("binding_set_sha256", SEMANTIC_BINDING_SET_HASH_DOMAIN)
                    }
                    TYPE_CONTRACT_SCHEMA => ("contract_sha256", TYPE_CONTRACT_HASH_DOMAIN),
                    METHOD_CONTRACT_SCHEMA => ("contract_sha256", METHOD_CONTRACT_HASH_DOMAIN),
                    BOUNDARY_CONTRACT_SCHEMA | "mpk.csharp.boundary.v2" => {
                        ("contract_sha256", BOUNDARY_CONTRACT_HASH_DOMAIN)
                    }
                    TRANSITION_CONTRACT_SCHEMA => {
                        ("contract_sha256", TRANSITION_CONTRACT_HASH_DOMAIN)
                    }
                    _ => panic!("unknown sidecar {schema}"),
                };
                let old_hash = old_value.get(field).unwrap().as_str().unwrap();
                let valid_old_hash = digest(&old_value, field, domain) == old_hash;
                let new_hash = if valid_old_hash {
                    digest(&new_value, field, domain)
                } else {
                    old_hash.to_owned()
                };
                let PracticalJsonValue::Object(entries) = &mut new_value else {
                    panic!("sidecar object");
                };
                entries.last_mut().unwrap().1 = PracticalJsonValue::string(new_hash.clone());
                new_input["utf8"] = json!(String::from_utf8(
                    canonical_practical_json_bytes(&new_value).unwrap()
                )
                .unwrap());
                println!(
                    "{}",
                    json!({"request_path":relative,"row_index":row_index,"path":old_input["path"],
                    "schema":schema,"hash_field":field,"old_hash":old_hash,"new_hash":new_hash,
                    "original_hash_valid":valid_old_hash,"intentional_invalid_hash_preserved":!valid_old_hash})
                );
            }
        }
        let destination = Path::new(&args[4]).join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        let mut bytes = serde_json::to_vec(&new).unwrap();
        bytes.push(b'\n');
        fs::write(destination, bytes).unwrap();
    }
}
