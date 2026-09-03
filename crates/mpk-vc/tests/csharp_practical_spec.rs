//! Private executable-specification owner: CSHARP-03-T01-W08.
//! These tests do not install/dispatch any practical production profile.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SPEC: &str = "develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md";
const DESCRIPTOR: &str = "develop/migrations/csharp-03/foundation/foundation-descriptor.json";
const DEFINITIONS: &str = "develop/migrations/csharp-03/foundation/foundation-definitions.json";
const VECTORS: &str = "develop/specs/vectors/csharp-practical-foundation-v1.json";
const RUNTIME: &str = "develop/migrations/csharp-03/probes/runtime-foundation-data.json";
const OWNER: &str = "crates/mpk-vc/tests/csharp_practical_spec.rs";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Vec<u8> {
    fs::read(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn document(path: &str) -> Value {
    serde_json::from_slice(&read(path)).expect("valid JSON")
}

fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical JSON")
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn domain_hash(domain: &str, value: &Value) -> String {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend(canonical(value));
    sha(&bytes)
}

fn text(value: &Value) -> &str {
    value.as_str().expect("string")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("array")
}

fn assert_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(actual, expected.iter().copied().collect());
}

fn type_id(value: &Value) -> String {
    match text(&value["kind"]) {
        "primitive" => format!("mpk.csharp.value.{}.v1", text(&value["id"])),
        "source" => text(&value["id"]).to_owned(),
        "instance" => {
            let preimage = json!({
                "template": format!("mpk.csharp.semantic.{}.v1", text(&value["template"])),
                "version": 1,
                "arguments": array(&value["arguments"]).iter().map(type_id).collect::<Vec<_>>()
            });
            format!(
                "mpk.csharp.instance.{}",
                domain_hash("MPK-CSHARP-SEMANTIC-INSTANCE-1.0", &preimage)
            )
        }
        _ => panic!("unclosed type"),
    }
}

fn substitute(value: &Value, args: &[Value]) -> Value {
    match value {
        Value::Object(fields) if fields.get("kind") == Some(&json!("parameter")) => {
            args[value["index"].as_u64().expect("parameter index") as usize].clone()
        }
        Value::Object(fields) => fields
            .iter()
            .map(|(k, v)| (k.clone(), substitute(v, args)))
            .collect(),
        Value::Array(values) => values.iter().map(|v| substitute(v, args)).collect(),
        _ => value.clone(),
    }
}

fn node_count(value: &Value) -> usize {
    1 + match value {
        Value::Object(fields) => fields.values().map(node_count).sum(),
        Value::Array(values) => values.iter().map(node_count).sum(),
        _ => 0,
    }
}

fn row<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    array(&vectors["vectors"])
        .iter()
        .find(|row| row["id"] == id)
        .expect("vector ID")
}

#[test]
fn csharp_03_t01_w08_descriptor_members_domains_and_closed_registry() {
    let descriptor = document(DESCRIPTOR);
    assert_keys(
        &descriptor,
        &[
            "schema",
            "id",
            "version",
            "semantic_profile",
            "members",
            "template_ids",
            "non_template_ids",
            "hash_domains",
            "structural_limits",
            "value_bounds",
            "source_callable_members",
            "caller_extension_points",
            "activation",
            "content_sha256",
        ],
    );
    assert_eq!(descriptor["schema"], "mpk.csharp.foundation_descriptor.v1");
    assert_eq!(descriptor["activation"], "candidate_only");
    assert_eq!(descriptor["source_callable_members"], json!([]));
    assert_eq!(descriptor["caller_extension_points"], json!([]));
    let mut preimage = descriptor.clone();
    preimage.as_object_mut().unwrap().remove("content_sha256");
    assert_eq!(
        text(&descriptor["content_sha256"]),
        domain_hash("MPK-CSHARP-PRACTICAL-FOUNDATION-1.0", &preimage)
    );
    let members = array(&descriptor["members"]);
    assert_eq!(
        members.iter().map(|m| text(&m["path"])).collect::<Vec<_>>(),
        vec![DEFINITIONS, SPEC]
    );
    for member in members {
        assert_keys(member, &["path", "schema", "sha256", "size_bytes"]);
        let path = root().join(text(&member["path"]));
        assert!(!fs::symlink_metadata(path).unwrap().file_type().is_symlink());
        let bytes = read(text(&member["path"]));
        assert_eq!(text(&member["sha256"]), sha(&bytes));
        assert_eq!(member["size_bytes"].as_u64().unwrap(), bytes.len() as u64);
    }
    let definitions = document(DEFINITIONS);
    let templates = array(&definitions["templates"]);
    let expected = [
        ("boundary_field", 1, 0),
        ("bounded_sequence", 1, 0),
        ("lookup", 1, 0),
        ("money", 1, 0),
        ("option", 1, 0),
        ("ordered_entry", 2, 0),
        ("ordered_map", 2, 3),
        ("ordered_set", 1, 1),
        ("result", 2, 0),
        ("sequence_construction", 1, 1),
        ("transition", 3, 1),
        ("validation", 2, 1),
    ];
    assert_eq!(templates.len(), expected.len());
    for (template, (name, arity, dependencies)) in templates.iter().zip(expected) {
        assert_eq!(template["name"], name);
        assert_eq!(template["id"], format!("mpk.csharp.semantic.{name}.v1"));
        assert_eq!(template["arity"], arity);
        assert_eq!(array(&template["dependencies"]).len(), dependencies);
        assert_eq!(template["source_callable"], false);
        let operations = array(&template["operations"]);
        let names: BTreeSet<_> = operations.iter().map(|op| text(&op["name"])).collect();
        assert_eq!(names.len(), operations.len());
        assert!(!operations.is_empty());
        for op in operations {
            assert_keys(
                op,
                &[
                    "name",
                    "arguments",
                    "result",
                    "equation",
                    "error_precedence",
                ],
            );
            assert!(!text(&op["equation"]).is_empty());
        }
    }
    let non_templates: BTreeSet<_> = array(&definitions["non_templates"])
        .iter()
        .map(|r| text(&r["name"]))
        .collect();
    assert_eq!(
        non_templates,
        ["unit", "parse_error", "instant", "exception"]
            .into_iter()
            .collect()
    );
    assert_eq!(
        definitions["ordinary_core"]["value_carrier"],
        "C(0)=Pi(Nat,Bool); C(d+1)=Pi(Nat,C(d))"
    );
    assert!(text(&definitions["ordinary_core"]["proofs"]).contains("no new inductive shape"));
}

#[test]
fn csharp_03_t01_w08_independent_instance_closure_substitution_provenance_and_counts() {
    let vectors = document(VECTORS);
    let definitions = document(DEFINITIONS);
    let templates: BTreeMap<_, _> = array(&definitions["templates"])
        .iter()
        .map(|t| (text(&t["name"]), t))
        .collect();
    let specimen = row(&vectors, "specialization.all_templates");
    let roots = array(&specimen["inputs"]["roots"]);
    let closed = &specimen["expected"];
    let mut pending: Vec<(Value, String)> = roots
        .iter()
        .map(|r| (r["type"].clone(), text(&r["provenance_id"]).to_owned()))
        .collect();
    let mut actual: BTreeMap<String, (Value, BTreeSet<String>)> = BTreeMap::new();
    while let Some((ty, provenance)) = pending.pop() {
        if ty["kind"] != "instance" {
            continue;
        }
        let identity = type_id(&ty);
        let entry = actual
            .entry(identity)
            .or_insert_with(|| (ty.clone(), BTreeSet::new()));
        assert_eq!(entry.0, ty);
        if !entry.1.insert(provenance.clone()) {
            continue;
        }
        let args = array(&ty["arguments"]);
        pending.extend(args.iter().cloned().map(|arg| (arg, provenance.clone())));
        for dependency in array(&templates[text(&ty["template"])]["dependencies"]) {
            pending.push((substitute(dependency, args), provenance.clone()));
        }
    }
    let entries = array(&closed["entries"]);
    assert_eq!(entries.len(), actual.len());
    let mut operation_count = 0;
    let mut recipe_nodes = 0;
    for (entry, (identity, (ty, provenance))) in entries.iter().zip(&actual) {
        assert_eq!(text(&entry["instance_id"]), identity);
        assert_eq!(
            array(&entry["provenance_ids"])
                .iter()
                .map(|v| text(v).to_owned())
                .collect::<BTreeSet<_>>(),
            *provenance
        );
        let template = templates[text(&ty["template"])];
        let args = array(&ty["arguments"]);
        assert_eq!(
            entry["argument_ids"],
            json!(args.iter().map(type_id).collect::<Vec<_>>())
        );
        let dependencies: BTreeSet<_> = array(&template["dependencies"])
            .iter()
            .map(|d| type_id(&substitute(d, args)))
            .collect();
        assert_eq!(
            entry["dependency_ids"],
            json!(dependencies.iter().collect::<Vec<_>>())
        );
        assert!(dependencies.iter().all(|id| actual.contains_key(id)));
        assert!(!String::from_utf8(canonical(&entry["type_definition"]))
            .unwrap()
            .contains("parameter"));
        let operations = array(&entry["operation_definitions"]);
        assert_eq!(operations.len(), array(&template["operations"]).len());
        for (operation, recipe) in operations.iter().zip(array(&template["operations"])) {
            assert_eq!(
                operation["id"],
                format!("{identity}.{}", text(&recipe["name"]))
            );
            assert_eq!(operation["equation"], recipe["equation"]);
            assert_eq!(operation["error_precedence"], recipe["error_precedence"]);
            for ty in array(&operation["argument_type_ids"])
                .iter()
                .chain(std::iter::once(&operation["normal_result_type_id"]))
            {
                let name = text(ty);
                assert!(name.starts_with("mpk.csharp.value.") || actual.contains_key(name));
            }
        }
        let count =
            node_count(&entry["type_definition"]) + node_count(&entry["operation_definitions"]);
        assert_eq!(
            entry["counters"],
            json!({"declarations":1,"operations":operations.len(),"recipe_nodes":count})
        );
        operation_count += operations.len();
        recipe_nodes += count;
    }
    assert_eq!(
        closed["counters"],
        json!({"declarations":entries.len(),"operations":operation_count,"recipe_nodes":recipe_nodes})
    );
    let mut preimage = closed.clone();
    preimage
        .as_object_mut()
        .unwrap()
        .remove("closed_set_sha256");
    assert_eq!(
        text(&closed["closed_set_sha256"]),
        domain_hash("MPK-CSHARP-CLOSED-INSTANCES-1.0", &preimage)
    );
}

#[test]
fn csharp_03_t01_w08_binding_schema_and_projection_obligations_are_not_proofs() {
    let vectors = document(VECTORS);
    for role in [
        "option",
        "lookup",
        "result",
        "validation",
        "boundary_field",
        "transition",
        "instant",
        "money",
        "bounded_sequence",
        "ordered_entry",
        "ordered_map",
        "ordered_set",
    ] {
        let specimen = row(&vectors, &format!("binding.{role}"));
        let binding = &specimen["inputs"]["binding"];
        assert_keys(
            binding,
            &[
                "schema",
                "source_type_id",
                "source_content_sha256",
                "role",
                "member_map",
                "tag_arms",
                "inferred_argument_ids",
                "default_arm",
                "bounds",
                "operation_map",
                "binding_sha256",
            ],
        );
        let mut preimage = binding.clone();
        preimage.as_object_mut().unwrap().remove("binding_sha256");
        assert_eq!(
            text(&binding["binding_sha256"]),
            domain_hash("MPK-CSHARP-SEMANTIC-BINDING-1.0", &preimage)
        );
        let source = &specimen["inputs"]["sources"][text(&binding["source_type_id"])];
        assert_eq!(binding["source_content_sha256"], source["source_sha256"]);
        let obligations = array(&specimen["expected"]["obligations"]);
        for kind in [
            "source_round_trip_all_observable_members",
            "semantic_round_trip_all_arms",
            "distinct_arms_disjoint",
            "identity_unobservable",
        ] {
            assert!(obligations.iter().any(|o| o["kind"] == kind));
        }
        for member in array(&source["members"]) {
            assert!(obligations
                .iter()
                .any(|o| o["kind"] == "field_complete_reconstruction"
                    && o["subject"] == member["id"]));
        }
        assert!(obligations.len() <= 64);
    }
    for (id, rejection) in [
        (
            "projection.inactive_payload_observable",
            "source_round_trip",
        ),
        ("projection.arm_collapse", "source_round_trip"),
        ("projection.operation_mismatch", "operation_commutation"),
        ("ownership.read_after_transfer", "ownership"),
        ("collections.duplicate_before_capacity", "duplicate_key"),
    ] {
        assert_eq!(row(&vectors, id)["expected"]["reject"], rejection);
    }
}

#[test]
fn csharp_03_t01_w08_vectors_runtime_evidence_and_owner_closure() {
    let vectors = document(VECTORS);
    assert_eq!(vectors["owner_test"], OWNER);
    assert_eq!(
        vectors["schema"],
        "mpk.csharp.practical.foundation.conformance.v1"
    );
    let rows = array(&vectors["vectors"]);
    let ids: Vec<_> = rows.iter().map(|r| text(&r["id"])).collect();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(vectors["vector_count"].as_u64().unwrap(), rows.len() as u64);
    assert_eq!(
        text(&vectors["vector_ids_sha256"]),
        sha(&canonical(&json!(ids)))
    );
    let mut families = BTreeSet::new();
    for row in rows {
        assert_keys(
            row,
            &[
                "id",
                "family",
                "inputs",
                "expected",
                "implementation_owner",
                "production_test_owner",
            ],
        );
        let family = text(&row["family"]);
        families.insert(family);
        assert_eq!(
            row["implementation_owner"],
            vectors["family_owners"][family]["work_item"]
        );
        assert_eq!(
            row["production_test_owner"],
            vectors["family_owners"][family]["test"]
        );
        assert!(text(&row["production_test_owner"]).ends_with(text(&row["implementation_owner"])));
    }
    assert_eq!(families.len(), 13);
    let runtime = document(RUNTIME);
    assert_eq!(
        runtime["schema"],
        "mpk.csharp_practical.t01_w08.runtime_foundation.v0"
    );
    assert_eq!(runtime["clean_builds"], 2);
    assert_eq!(runtime["executions_per_build"], 4);
    assert_eq!(array(&runtime["observations"]).len(), 2);
    assert_eq!(
        text(&runtime["observations_sha256"]),
        sha(&[canonical(&runtime["observations"]), vec![b'\n']].concat())
    );
    for member in array(&runtime["inputs"]) {
        assert_eq!(text(&member["sha256"]), sha(&read(text(&member["path"]))));
    }
    let mut observed_operations = BTreeSet::new();
    for run in array(&runtime["observations"]) {
        assert_eq!(run["runtime"], "10.0.11");
        assert_eq!(
            array(&run["vectors"]).len() as u64,
            runtime["vector_count"].as_u64().unwrap()
        );
        for runtime_row in array(&run["vectors"]) {
            let operation = text(&runtime_row["operation"]);
            observed_operations.insert(operation);
            let family = if operation.starts_with("nullable.")
                || operation.starts_with("lifted.")
                || operation == "source.null_short_circuit"
            {
                "nullable"
            } else if operation == "source.array_two_pass" {
                "loops"
            } else if operation.starts_with("source.array_") {
                "ownership"
            } else if operation == "source.struct_default" {
                "default"
            } else if operation == "source.null_call_order" {
                "calls"
            } else if operation.starts_with("source.") {
                "construction"
            } else {
                "business"
            };
            let specimen = row(
                &vectors,
                &format!("{family}.runtime_{}", text(&runtime_row["id"])),
            );
            assert_eq!(runtime_row["observed"], specimen["expected"]);
            assert_eq!(runtime_row["inputs"], specimen["inputs"]["inputs"]);
        }
    }
    assert_eq!(runtime["operation_ids"], json!(observed_operations));
    // Predecessor evidence is immutable, and neither active descriptor is changed.
    assert_eq!(
        sha(&read(
            "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json"
        )),
        "0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769"
    );
    assert_eq!(
        sha(&read("release/build-inputs/csharp/build-inputs.json")),
        "0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc"
    );
}

#[test]
fn csharp_03_t01_w08_executable_specification_and_independent_runtime_oracle() {
    for (script, argument) in [
        ("foundation_package.py", "--check"),
        ("run-foundation-data-probe.py", "--check-record"),
    ] {
        let output = Command::new("python3")
            .arg(root().join("develop/probes/csharp-03").join(script))
            .arg(argument)
            .current_dir(root())
            .output()
            .expect("run Python executable specification");
        assert!(
            output.status.success(),
            "{script}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
