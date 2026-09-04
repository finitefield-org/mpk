//! W08 foundation, W09 freeze/capacity evidence, and the W10 publication owner.
//! These tests do not install or dispatch any practical production profile.

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
const W09_FREEZE: &str = "develop/migrations/csharp-03/freeze/profile-freeze.json";
const W09_VECTORS: &str = "develop/migrations/csharp-03/freeze/profile-freeze-vectors.json";
const W09_CAPACITY: &str = "develop/migrations/csharp-03/probes/checker-capacity.json";
const W09_INVENTORY: &str = "develop/migrations/csharp-03/artifact-consumer-inventory.json";
const W10_PROFILE_SPEC: &str = "develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md";
const W10_SHARED_SPEC: &str = "develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md";
const W10_VECTORS: &str = "develop/specs/vectors/csharp-practical-profile-v1.json";
const W10_MANIFEST: &str = "develop/specs/vectors/manifest.json";
const W10_GENERATOR: &str = "develop/probes/csharp-03/profile_package.py";
const W10_DESIGN: &str = "develop/docs/08_csharp_practical_subset_design.md";
const W10_PLAN: &str = "develop/docs/08_csharp_practical_subset_design-todo.md";
const W10_LEDGER: &str = "develop/docs/csharp-03-implementation-traceability-ledger.md";
const ACTIVE_SEMANTIC_REGISTRY: &str = "release/bundles/semantic-profile-registry.json";
const ACTIVE_BUNDLE_REGISTRY: &str = "release/bundles/bundle-registry.json";
const JAVA_GATE: &str = "scripts/check-java-frontend.sh";
const AGGREGATE_GATE: &str = "scripts/check-all.sh";
const PRACTICAL_GATE: &str = "scripts/check-csharp-practical-release.sh";
const OWNER: &str = "crates/mpk-vc/tests/csharp_practical_spec.rs";

#[path = "../../../develop/probes/csharp-03/recursor_feasibility.rs"]
mod recursor_feasibility;

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

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    array(&vectors["vectors"])
        .iter()
        .find(|row| row["id"] == id)
        .expect("W09 vector ID")
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
        "C(0)=Bool; C(d+1)=Pi(Bool,C(d))"
    );
    assert!(text(&definitions["ordinary_core"]["conditionals"])
        .contains("every Bool.rec has Bool cases, major, and result"));
    assert!(text(&definitions["ordinary_core"]["folds"]).contains("concrete S->S transformers"));
    assert!(text(&definitions["ordinary_core"]["folds"]).contains("no Nat.rec"));
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

#[test]
fn csharp_03_t01_w09_successor_identities_schemas_and_owners_are_frozen() {
    let freeze = document(W09_FREEZE);
    assert_keys(
        &freeze,
        &[
            "schema",
            "work_item",
            "status",
            "semantic_profile",
            "activation",
            "predecessor_commit",
            "identity_families",
            "schemas",
            "schema_type_system",
            "expression_union",
            "semantic_context_binding",
            "canonical_json",
            "boundary_handoff",
            "transition",
            "diagnostics",
            "frontend_linkage",
            "limits",
            "executable_dispatch",
            "termination",
            "ownership",
            "evidence",
            "publication_owner",
            "content_hash_domain",
            "content_sha256",
        ],
    );
    assert_eq!(freeze["schema"], "mpk.csharp_practical.t01_w09.freeze.v1");
    assert_eq!(freeze["work_item"], "CSHARP-03-T01-W09");
    assert_eq!(freeze["status"], "frozen_candidate_only");
    assert_eq!(freeze["activation"], "candidate_only");
    assert_eq!(freeze["semantic_profile"], "mpk.csharp.practical.v1");
    assert_eq!(freeze["publication_owner"], "CSHARP-03-T01-W10");
    let mut preimage = freeze.clone();
    preimage.as_object_mut().unwrap().remove("content_sha256");
    assert_eq!(
        text(&freeze["content_sha256"]),
        domain_hash("MPK-CSHARP-PRACTICAL-FREEZE-1.0", &preimage)
    );

    let inventory = document(W09_INVENTORY);
    assert_eq!(
        freeze["ownership"]["inventory_raw_sha256"],
        sha(&read(W09_INVENTORY))
    );
    assert_eq!(
        freeze["ownership"]["migration_set"],
        inventory["atomic_migration_set"]
    );
    assert_eq!(
        freeze["ownership"]["rollback_set"],
        inventory["whole_image_rollback_set"]
    );
    let frozen_families = array(&freeze["identity_families"]);
    let inventory_families = array(&inventory["identity_families"]);
    assert_eq!(frozen_families.len(), 17);
    assert_eq!(frozen_families.len(), inventory_families.len());
    let mut identities = BTreeMap::new();
    let mut domains = BTreeMap::new();
    let mut retained_identities = BTreeSet::new();
    let mut retained_domains = BTreeSet::new();
    for (frozen, inventoried) in frozen_families.iter().zip(inventory_families) {
        assert_keys(
            frozen,
            &[
                "family",
                "successor_identities",
                "successor_hash_domains",
                "retained_identities",
                "retained_hash_domains",
                "implementation_owners",
                "migration_set",
            ],
        );
        assert_eq!(frozen["family"], inventoried["id"]);
        assert_eq!(
            frozen["implementation_owners"],
            inventoried["implementation_owners"]
        );
        assert_eq!(
            frozen["retained_identities"],
            inventoried["current_identities"]
        );
        retained_identities.extend(array(&frozen["retained_identities"]).iter().map(text));
        retained_domains.extend(
            array(&frozen["retained_hash_domains"])
                .iter()
                .map(|row| text(&row["id"])),
        );
        for identity in array(&frozen["successor_identities"]) {
            assert!(identities
                .insert(text(identity), text(&frozen["family"]))
                .is_none());
        }
        for domain in array(&frozen["successor_hash_domains"]) {
            assert!(domains
                .insert(text(domain), text(&frozen["family"]))
                .is_none());
        }
    }
    assert_eq!(
        frozen_families[14]["successor_identities"],
        json!(["mpk.program_certificate.ordinary_context.v2"])
    );
    let certificate_domain = format!("MPK-{}-0.1", "CERT");
    assert!(array(&frozen_families[14]["retained_hash_domains"])
        .iter()
        .any(|row| row["id"] == certificate_domain
            && row["decision"] == "retain_certificate_v0_preimage"));
    assert!(identities
        .keys()
        .all(|identity| !retained_identities.contains(identity)));
    assert!(domains
        .keys()
        .all(|domain| !retained_domains.contains(domain)));

    let declared: BTreeSet<_> = identities.keys().copied().collect();
    let schemas = array(&freeze["schemas"]);
    assert_eq!(schemas.len(), 15);
    for schema in schemas {
        assert_keys(
            schema,
            &[
                "id",
                "version",
                "root",
                "ordered_fields",
                "field_types",
                "required_fields",
                "optional_fields",
                "hash_field",
                "hash_domain",
                "hash_preimage_fields",
                "unknown_fields",
                "duplicate_keys",
                "later_versions",
                "producer",
                "consumers",
            ],
        );
        assert!(declared.contains(text(&schema["id"])));
        assert_eq!(schema["root"], "object");
        assert_eq!(
            array(&schema["ordered_fields"])
                .iter()
                .map(text)
                .collect::<BTreeSet<_>>(),
            schema["field_types"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect()
        );
        assert_eq!(schema["ordered_fields"], schema["required_fields"]);
        assert_eq!(schema["optional_fields"], json!([]));
        assert_eq!(schema["unknown_fields"], "reject");
        assert_eq!(
            schema["duplicate_keys"],
            "reject_before_object_construction"
        );
        assert_eq!(schema["later_versions"], "reject");
        assert!(!array(&schema["ordered_fields"]).is_empty());
        if !schema["hash_field"].is_null() {
            assert_eq!(
                array(&schema["ordered_fields"]).last(),
                Some(&schema["hash_field"])
            );
            assert!(domains.contains_key(text(&schema["hash_domain"])));
        }
        if !schema["hash_domain"].is_null() {
            assert!(domains.contains_key(text(&schema["hash_domain"])));
            assert_eq!(
                array(&schema["hash_preimage_fields"])
                    .iter()
                    .collect::<Vec<_>>(),
                array(&schema["ordered_fields"])
                    .iter()
                    .filter(|field| **field != schema["hash_field"])
                    .collect::<Vec<_>>()
            );
        } else {
            assert!(schema["hash_preimage_fields"].is_null());
        }
    }
    let type_system = &freeze["schema_type_system"];
    let records = array(&type_system["nested_records"]);
    assert_eq!(records.len(), 20);
    let record_ids: BTreeSet<_> = records.iter().map(|row| text(&row["id"])).collect();
    assert_eq!(record_ids.len(), records.len());
    for required in [
        "boundary_field",
        "csharp_practical_parameter_values_v1",
        "boundary_evidence_linkage",
        "exceptional_case",
        "loop_contract",
        "transition_version_rule",
        "idempotency_complete_snapshot",
        "diagnostic_entry_v2",
    ] {
        assert!(record_ids.contains(required));
    }
    for record in records {
        assert_eq!(record["required_fields"], record["ordered_fields"]);
        assert_eq!(record["optional_fields"], json!([]));
        assert_eq!(record["unknown_fields"], "reject");
        assert_eq!(
            record["duplicate_keys"],
            "reject_before_object_construction"
        );
        assert_eq!(
            array(&record["ordered_fields"])
                .iter()
                .map(text)
                .collect::<BTreeSet<_>>(),
            record["field_types"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect()
        );
        assert!(text(&record["producer"]).starts_with("CSHARP-03-"));
    }
    let practical_parameters = records
        .iter()
        .find(|row| row["id"] == "csharp_practical_parameter_values_v1")
        .unwrap();
    assert_eq!(
        practical_parameters["field_types"]["check_overflow_default"],
        "literal<true>"
    );
    assert_eq!(
        practical_parameters["field_types"]["nullable_context"],
        "literal<enable>"
    );
    let diagnostic_entry = records
        .iter()
        .find(|row| row["id"] == "diagnostic_entry_v2")
        .unwrap();
    assert_eq!(
        diagnostic_entry["field_types"]["message"],
        "sanitized_public_message_literal"
    );
    let type_contract = schemas
        .iter()
        .find(|row| text(&row["id"]).ends_with("csharp.type_contract.v1"))
        .unwrap();
    assert_eq!(
        type_contract["field_types"]["construction_invariant"],
        "contract_expression_bool_or_null"
    );
    assert_eq!(
        type_contract["field_types"]["invariants"],
        "ordered_array<contract_expression_bool>"
    );
    assert_eq!(
        type_contract["field_types"]["source_type_id"],
        "closed_source_type_id"
    );
    assert_eq!(
        type_contract["field_types"]["ordered_member_ids"],
        "ordered_unique_array<canonical_source_member_id>"
    );
    let unions = array(&type_system["tagged_unions"]);
    assert_eq!(unions.len(), 3);
    let idempotency_union = unions
        .iter()
        .find(|row| row["id"] == "transition_idempotency")
        .unwrap();
    assert_eq!(idempotency_union["id"], "transition_idempotency");
    assert_eq!(idempotency_union["unknown_tags"], "reject");
    assert_eq!(idempotency_union["unknown_fields"], "reject");
    assert_eq!(
        idempotency_union["duplicate_keys"],
        "reject_before_object_construction"
    );
    assert!(unions
        .iter()
        .any(|row| row["id"] == "boundary_missing_rule"));
    let diagnostic_linkage = unions
        .iter()
        .find(|row| row["id"] == "frontend_diagnostic_request_linkage")
        .unwrap();
    assert_eq!(diagnostic_linkage["tag_field"], "state");
    let linkage_variants = array(&diagnostic_linkage["variants"]);
    assert_eq!(
        linkage_variants
            .iter()
            .map(|row| text(&row["tag"]))
            .collect::<Vec<_>>(),
        vec!["unvalidated", "validated"]
    );
    assert_eq!(linkage_variants[0]["ordered_fields"], json!(["state"]));
    assert_eq!(
        linkage_variants[1]["ordered_fields"],
        json!(["state", "request_sha256", "semantic_context"])
    );
    let frontend_success = schemas
        .iter()
        .find(|row| text(&row["id"]).ends_with("frontend.success.v2"))
        .unwrap();
    assert_eq!(
        frontend_success["ordered_fields"],
        json!([
            "schema",
            "request_sha256",
            "semantic_context",
            "artifacts",
            "success_sha256"
        ])
    );
    assert_eq!(
        frontend_success["field_types"]["artifacts"],
        "mpk.frontend.source_artifacts.v2"
    );
    let frontend_diagnostic = schemas
        .iter()
        .find(|row| text(&row["id"]).ends_with("frontend.diagnostic.v2"))
        .unwrap();
    assert_eq!(
        frontend_diagnostic["ordered_fields"],
        json!([
            "schema",
            "raw_request_sha256",
            "raw_request_size_bytes",
            "request_linkage",
            "status",
            "phase",
            "diagnostics",
            "diagnostic_sha256"
        ])
    );
    assert_eq!(
        frontend_diagnostic["field_types"]["request_linkage"],
        "frontend_diagnostic_request_linkage"
    );
    assert_eq!(
        frontend_diagnostic["field_types"]["phase"],
        "diagnostic_phase"
    );
    let validated_request = schemas
        .iter()
        .find(|row| text(&row["id"]).ends_with("validated_semantic_request.v2"))
        .unwrap();
    assert_eq!(
        validated_request["field_types"]["selection"],
        "selection_envelope"
    );
    let context_binding = &freeze["semantic_context_binding"];
    assert_eq!(array(&context_binding["required_equalities"]).len(), 4);
    assert_eq!(context_binding["hash_only_or_projected_context"], "reject");
    assert!(text(&context_binding["validated_request_selection"])
        .contains("profile_entry.selection_schema"));
}

#[test]
fn csharp_03_t01_w09_contract_boundary_transition_and_dispatch_are_closed() {
    let freeze = document(W09_FREEZE);
    let expressions = &freeze["expression_union"];
    assert_eq!(expressions["schema"], "mpk.csharp.contract_expression.v1");
    assert_eq!(expressions["unknown_tags"], "reject");
    assert_eq!(expressions["unknown_fields"], "reject");
    assert_eq!(
        expressions["duplicate_keys"],
        "reject_before_object_construction"
    );
    assert_eq!(expressions["calls_source_methods"], false);
    let variants = array(&expressions["variants"]);
    let tags: BTreeSet<_> = variants.iter().map(|row| text(&row["tag"])).collect();
    assert_eq!(tags.len(), variants.len());
    for required in [
        "field",
        "sequence_index",
        "map_lookup",
        "tagged_make",
        "source_project",
        "structural_equal",
        "codec_parse",
        "exception_payload",
        "transition_events",
        "bounded_forall",
        "bounded_exists",
    ] {
        assert!(tags.contains(required));
    }
    for variant in variants {
        let fields = array(&variant["ordered_fields"]);
        assert_eq!(fields[0], "tag");
        assert_eq!(fields[1], "type_id");
        assert_eq!(variant["required_fields"], variant["ordered_fields"]);
        assert_eq!(variant["optional_fields"], json!([]));
        assert_eq!(
            fields.iter().map(text).collect::<BTreeSet<_>>(),
            variant["field_types"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect()
        );
    }
    let by_tag: BTreeMap<_, _> = variants
        .iter()
        .map(|variant| (text(&variant["tag"]), variant))
        .collect();
    assert_eq!(
        by_tag["codec_parse"]["field_types"]["codec_id"],
        "registered_codec_id"
    );
    assert_eq!(
        by_tag["tagged_make"]["field_types"]["arm"],
        "registered_sum_arm_id"
    );
    assert_eq!(
        by_tag["source_project"]["field_types"]["binding_id"],
        "registered_semantic_binding_id"
    );

    let json = &freeze["canonical_json"];
    assert_eq!(json["encoding"], "UTF-8 without BOM");
    assert_eq!(json["outside_string_whitespace"], "forbidden");
    assert_eq!(
        json["duplicate_members"],
        "reject before object construction"
    );
    assert_eq!(json["unknown_members"], "reject");
    assert_eq!(
        json["missing_null_value"],
        "missing omits the field; null emits JSON null; value emits exactly one canonical payload"
    );
    assert_eq!(
        json["map"],
        "ordered array of typed key/value entries, never a JSON object"
    );

    let boundary = &freeze["boundary_handoff"];
    assert_eq!(
        boundary["classification"],
        "MPK verification-overlay transport"
    );
    assert_eq!(boundary["application_protocol"], false);
    assert_eq!(boundary["bypass"], "reject");
    assert_eq!(boundary["hash_only_equivalence"], "forbidden");
    assert_eq!(array(&boundary["input_order"]).len(), 6);
    assert_eq!(array(&boundary["output_order"]).len(), 5);

    let transition = &freeze["transition"];
    assert_eq!(
        transition["precedence"],
        json!([
            "ordinary_boundary_preconditions",
            "retained_key_lookup",
            "equal_snapshot_replay_or_idempotency_conflict",
            "expected_version_conflict",
            "idempotency_history_capacity",
            "version_exhausted",
            "accepted_command_case_and_declared_business_errors",
            "new_success"
        ])
    );
    assert_eq!(transition["version_rule"]["carrier"], "u64");
    assert_eq!(transition["version_rule"]["replay"], "unchanged");
    assert_eq!(transition["version_rule"]["error"], "unchanged");
    let idempotency = &transition["idempotency"];
    assert_eq!(
        idempotency["modes"],
        json!(["disabled", "complete_snapshot"])
    );
    assert_eq!(idempotency["unavailable_when_incomplete"], true);
    assert_eq!(idempotency["digest_substitution"], "forbidden");
    assert!(array(&idempotency["ineligible_fields"]).contains(&json!("float")));
    assert!(array(&idempotency["ineligible_fields"]).contains(&json!("double")));

    let dispatch = &freeze["executable_dispatch"];
    assert_eq!(dispatch["binary_name"], "csharp2vir");
    assert_eq!(dispatch["binary_path"], "csharp2vir.dll");
    assert_eq!(
        dispatch["successor_bundle_id"],
        "frontend.csharp.csharp2vir.candidate.v2"
    );
    let scalar_profile_id = ["mpk.csharp.", "scalar.v0"].concat();
    assert_eq!(
        dispatch["profiles"],
        json!(["mpk.csharp.practical.v1", scalar_profile_id])
    );
    assert_eq!(dispatch["ambient_flag"], "forbidden");
    assert_eq!(dispatch["fallback"], "forbidden");
    let frontend_linkage = &freeze["frontend_linkage"];
    assert_eq!(array(&frontend_linkage["success_equalities"]).len(), 4);
    assert_eq!(
        array(&frontend_linkage["diagnostic_validated_equalities"]).len(),
        2
    );
    assert_eq!(
        frontend_linkage["partial_artifacts_on_failure"],
        "forbidden"
    );
    assert!(text(&frontend_linkage["comparison"]).contains("field-complete typed equality"));
    assert_eq!(
        freeze["termination"]["total_required_routes"],
        json!([
            "boundary",
            "transition",
            "example",
            "public practical profile"
        ])
    );
    assert_eq!(
        freeze["termination"]["partial_callee_on_total_path"],
        "reject"
    );
}

#[test]
fn csharp_03_t01_w09_limits_capacity_and_diagnostics_have_total_boundaries() {
    let freeze = document(W09_FREEZE);
    let limits = array(&freeze["limits"]["practical"]);
    assert_eq!(limits.len(), 35);
    let mut by_id = BTreeMap::new();
    let mut classifications = BTreeSet::new();
    for limit in limits {
        assert_keys(
            limit,
            &[
                "id",
                "inclusive_maximum",
                "unit",
                "classification",
                "increment_site",
                "increment_rule",
                "comparison_rule",
                "overflow_rule",
                "diagnostic",
                "implementation_owner",
            ],
        );
        assert!(limit["inclusive_maximum"].as_u64().unwrap() > 0);
        assert_eq!(limit["diagnostic"], "CSHARP_PRACTICAL_LIMIT");
        assert_eq!(
            limit["increment_rule"],
            "checked_add(counter,1) exactly once at the increment site"
        );
        classifications.insert(text(&limit["classification"]));
        assert!(by_id.insert(text(&limit["id"]), limit).is_none());
    }
    assert_eq!(
        classifications,
        BTreeSet::from(["pre_invocation_structural", "runtime_value_predicate_vc"])
    );
    assert_eq!(array(&freeze["limits"]["retained_scalar_v0"]).len(), 32);

    let capacity = document(W09_CAPACITY);
    assert_eq!(
        freeze["evidence"]["freeze_generator_raw_sha256"],
        sha(&read("develop/probes/csharp-03/profile_freeze.py"))
    );
    assert_eq!(
        freeze["evidence"]["retained_limit_source_raw_sha256"],
        sha(&read(text(
            &freeze["evidence"]["retained_limit_source_path"]
        )))
    );
    assert_eq!(
        freeze["evidence"]["capacity_evidence_raw_sha256"],
        sha(&read(W09_CAPACITY))
    );
    assert_eq!(
        freeze["evidence"]["capacity_source_inventory_sha256"],
        capacity["source_inventory_sha256"]
    );
    assert_eq!(freeze["evidence"]["checker_invocations"], 48);
    assert_eq!(freeze["evidence"]["checker_acceptances"], 48);
    for family in [
        "binder_depth",
        "generated_declarations",
        "ordinary_term_nodes",
        "static_transformers",
    ] {
        assert_eq!(
            by_id[family]["inclusive_maximum"],
            capacity["probe"]["limits"][family]
        );
        let cases: Vec<_> = array(&capacity["probe"]["cases"])
            .iter()
            .filter(|row| row["family"] == family)
            .collect();
        assert_eq!(cases.len(), 3);
        let maximum = by_id[family]["inclusive_maximum"].as_u64().unwrap();
        assert_eq!(
            cases
                .iter()
                .map(|row| row["counter_value"].as_u64().unwrap())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([maximum - 1, maximum, maximum + 1])
        );
        assert_eq!(
            cases
                .iter()
                .filter(|row| row["profile_expected"] == "limit_exceeded")
                .count(),
            1
        );
        assert!(cases.iter().all(|row| row["checker_expected"] == "accepted"
            && row["proof_nodes"] == 0
            && row["theory_certificates"] == 0
            && row["axioms"] == 0));
    }
    for run in array(&capacity["runs"]) {
        assert_eq!(array(&run["observations"]).len(), 12);
        assert!(array(&run["observations"]).iter().all(|row| {
            row["rust"]["result"] == "accepted" && row["reference"]["result"] == "accepted"
        }));
    }

    let diagnostics = &freeze["diagnostics"];
    let families = array(&diagnostics["families"]);
    assert_eq!(families.len(), 29);
    assert_eq!(
        families.iter().map(text).collect::<BTreeSet<_>>().len(),
        families.len()
    );
    let flattened: Vec<_> = array(&diagnostics["phase_precedence"])
        .iter()
        .flat_map(|phase| {
            array(&phase["families_in_precedence_order"])
                .iter()
                .map(text)
        })
        .collect();
    assert_eq!(flattened.len(), families.len());
    assert_eq!(
        flattened.iter().copied().collect::<BTreeSet<_>>(),
        families.iter().map(text).collect()
    );
    assert!(
        array(&diagnostics["forbidden_public_data"]).contains(&json!("customer member spelling"))
    );
    assert!(text(&diagnostics["request_linkage"]).contains("raw request hash and size"));
    assert!(text(&diagnostics["phase_rule"]).contains("exactly 0..8"));
    assert!(text(&diagnostics["location_rule"]).contains("start_byte < end_byte"));
}

#[test]
fn csharp_03_t01_w09_vectors_cover_all_strict_mutations_limits_and_precedence() {
    let freeze = document(W09_FREEZE);
    let vectors = document(W09_VECTORS);
    assert_keys(
        &vectors,
        &[
            "schema",
            "work_item",
            "freeze_content_sha256",
            "owner_test",
            "publication_owner",
            "vector_count",
            "vector_ids_sha256",
            "vectors",
        ],
    );
    assert_eq!(vectors["work_item"], "CSHARP-03-T01-W09");
    assert_eq!(vectors["freeze_content_sha256"], freeze["content_sha256"]);
    assert_eq!(vectors["owner_test"], format!("{OWNER}#CSHARP-03-T01-W09"));
    assert_eq!(vectors["publication_owner"], "CSHARP-03-T01-W10");
    let rows = array(&vectors["vectors"]);
    let ids: Vec<_> = rows.iter().map(|row| text(&row["id"])).collect();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(vectors["vector_count"].as_u64().unwrap(), rows.len() as u64);
    assert_eq!(vectors["vector_ids_sha256"], sha(&canonical(&json!(ids))));
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
        assert!(text(&row["production_test_owner"]).ends_with(text(&row["implementation_owner"])));
    }

    for schema in array(&freeze["schemas"]) {
        let key = text(&schema["id"])
            .strip_prefix("mpk.")
            .unwrap()
            .replace('.', "_");
        for suffix in [
            "valid",
            "later_version",
            "unknown_field",
            "missing_field",
            "wrong_field_type",
            "duplicate_key",
        ] {
            let item = vector(&vectors, &format!("schema.{key}.{suffix}"));
            if suffix == "valid" {
                assert_eq!(item["expected"], json!({"accept":true}));
            } else {
                assert!(item["expected"].get("reject").is_some());
            }
            assert_eq!(item["implementation_owner"], schema["producer"]);
        }
    }
    for record in array(&freeze["schema_type_system"]["nested_records"]) {
        let key = text(&record["id"]).replace('.', "_");
        for suffix in [
            "valid",
            "unknown_field",
            "missing_field",
            "wrong_field_type",
            "duplicate_key",
        ] {
            let item = vector(&vectors, &format!("schema.nested_{key}.{suffix}"));
            if suffix == "valid" {
                assert_eq!(item["expected"], json!({"accept":true}));
            } else {
                assert!(item["expected"].get("reject").is_some());
            }
            assert_eq!(item["implementation_owner"], record["producer"]);
        }
    }
    for variant in array(&freeze["expression_union"]["variants"]) {
        let tag = text(&variant["tag"]);
        for suffix in [
            "valid",
            "unknown_field",
            "missing_field",
            "wrong_field_type",
            "duplicate_key",
        ] {
            let item = vector(&vectors, &format!("schema.expression.{tag}.{suffix}"));
            if suffix == "valid" {
                assert_eq!(item["expected"], json!({"accept":true}));
            } else {
                assert!(item["expected"].get("reject").is_some());
            }
            assert_eq!(item["implementation_owner"], "CSHARP-03-T06-W01");
        }
    }
    for union in array(&freeze["schema_type_system"]["tagged_unions"]) {
        let union_id = text(&union["id"]);
        for variant in array(&union["variants"]) {
            let tag = text(&variant["tag"]);
            for suffix in [
                "valid",
                "unknown_field",
                "missing_field",
                "wrong_field_type",
                "duplicate_key",
            ] {
                let item = vector(&vectors, &format!("schema.union_{union_id}.{tag}.{suffix}"));
                if suffix == "valid" {
                    assert_eq!(item["expected"], json!({"accept":true}));
                } else {
                    assert!(item["expected"].get("reject").is_some());
                }
                assert_eq!(item["implementation_owner"], union["producer"]);
            }
        }
        assert!(
            vector(&vectors, &format!("schema.union_{union_id}.unknown_tag"))["expected"]
                .get("reject")
                .is_some()
        );
    }
    assert_eq!(
        vector(&vectors, "schema.union_transition_idempotency.unknown_tag")["expected"]["reject"],
        "unknown_transition_idempotency_tag"
    );
    for group in ["practical", "retained"] {
        let source = if group == "practical" {
            &freeze["limits"]["practical"]
        } else {
            &freeze["limits"]["retained_scalar_v0"]
        };
        for limit in array(source) {
            let maximum = limit["inclusive_maximum"].as_u64().unwrap();
            for (suffix, expected) in [
                ("below", maximum - 1),
                ("at", maximum),
                ("above", maximum + 1),
            ] {
                let item = vector(
                    &vectors,
                    &format!("limit.{group}.{}.{suffix}", text(&limit["id"])),
                );
                assert_eq!(item["inputs"]["value"], expected);
                assert_eq!(item["inputs"]["inclusive_maximum"], maximum);
                if group == "practical" {
                    assert_eq!(item["implementation_owner"], limit["implementation_owner"]);
                }
            }
        }
    }
    let precedence: Vec<_> = array(&freeze["diagnostics"]["phase_precedence"])
        .iter()
        .flat_map(|phase| {
            array(&phase["families_in_precedence_order"])
                .iter()
                .map(text)
        })
        .collect();
    for (index, family) in precedence.iter().take(precedence.len() - 1).enumerate() {
        assert_eq!(
            vector(&vectors, &format!("diagnostic.precedence_{index:02}"))["expected"]["primary"],
            *family
        );
    }
    assert_eq!(
        vector(&vectors, "idempotency.float_field")["expected"]["mode"],
        "unavailable"
    );
    assert_eq!(
        vector(&vectors, "boundary.input_bypass")["expected"]["reject"],
        "boundary_bypass"
    );
    assert_eq!(
        vector(&vectors, "dispatch.scalar")["expected"]["bundle"],
        vector(&vectors, "dispatch.practical")["expected"]["bundle"]
    );
    assert_eq!(
        vector(&vectors, "context.selection_schema_mismatch")["expected"]["reject"],
        "selection_schema_mismatch"
    );
    assert_eq!(
        vector(&vectors, "context.projected_context")["expected"]["reject"],
        "projected_semantic_context"
    );
    assert_eq!(
        vector(
            &vectors,
            "frontend_linkage.diagnostic_unvalidated_after_validation"
        )["expected"]["reject"],
        "diagnostic_request_linkage"
    );
    assert_eq!(
        vector(&vectors, "frontend_linkage.diagnostic_partial_artifacts")["expected"]["reject"],
        "partial_frontend_artifacts"
    );
    assert_eq!(
        vector(&vectors, "diagnostic.phase_above")["expected"]["reject"],
        "diagnostic_phase"
    );
    assert_eq!(
        vector(&vectors, "diagnostic.mixed_phase")["expected"]["reject"],
        "diagnostic_phase_code"
    );
    assert_eq!(
        vector(&vectors, "diagnostic.invalid_location")["expected"]["reject"],
        "diagnostic_location"
    );
}

#[test]
fn csharp_03_t01_w09_freeze_generator_is_reproducible() {
    let output = Command::new("python3")
        .arg(root().join("develop/probes/csharp-03/profile_freeze.py"))
        .arg("--check")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root())
        .output()
        .expect("run W09 freeze generator");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn csharp_03_t01_w10_publication_reproduces_the_complete_private_freeze() {
    let package = document(W10_VECTORS);
    assert_keys(
        &package,
        &[
            "canonical_evidence",
            "downstream_work_item_owners",
            "freeze_requirement_owners",
            "frozen_contract",
            "historical_inventory_extension",
            "incorporated_design",
            "name_owner_inventory",
            "owner_test",
            "publication_generator",
            "release_gate",
            "schema",
            "semantic_profile",
            "source_w09",
            "specification_members",
            "status",
            "upgrade_matrix",
            "vector_count",
            "vector_ids_sha256",
            "vectors",
            "work_item",
        ],
    );
    assert_eq!(
        package["schema"],
        "mpk.csharp.practical.profile.conformance.v1"
    );
    assert_eq!(package["work_item"], "CSHARP-03-T01-W10");
    assert_eq!(package["status"], "normative_frozen_inactive");
    assert_eq!(package["semantic_profile"], "mpk.csharp.practical.v1");
    assert_eq!(package["owner_test"], OWNER);

    let freeze = document(W09_FREEZE);
    let private_vectors = document(W09_VECTORS);
    assert_eq!(package["frozen_contract"], freeze);
    assert_eq!(package["vectors"], private_vectors["vectors"]);
    assert_eq!(package["vector_count"], private_vectors["vector_count"]);
    assert_eq!(
        package["vector_ids_sha256"],
        private_vectors["vector_ids_sha256"]
    );
    let ids = array(&package["vectors"])
        .iter()
        .map(|row| text(&row["id"]))
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(ids.len(), 700);
    assert_eq!(ids.iter().copied().collect::<BTreeSet<_>>().len(), 700);
    assert_eq!(package["vector_ids_sha256"], sha(&canonical(&json!(ids))));

    let source = &package["source_w09"];
    assert_eq!(source["commit"], "17525292755c4e508acd9300cfa72d20cdf9bb92");
    assert_eq!(source["freeze_path"], W09_FREEZE);
    assert_eq!(source["vectors_path"], W09_VECTORS);
    assert_eq!(source["freeze_schema"], freeze["schema"]);
    assert_eq!(source["vectors_schema"], private_vectors["schema"]);
    assert_eq!(source["freeze_content_sha256"], freeze["content_sha256"]);
    assert_eq!(text(&source["freeze_raw_sha256"]), sha(&read(W09_FREEZE)));
    assert_eq!(text(&source["vectors_raw_sha256"]), sha(&read(W09_VECTORS)));

    let expected_evidence = BTreeSet::from([
        "develop/migrations/csharp-03/baseline.json",
        W09_INVENTORY,
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
        "develop/migrations/csharp-03/build-inputs/candidate-inventory.json",
        "develop/migrations/csharp-03/probes/roslyn-data-construction.json",
        "develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json",
        "develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json",
        "develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json",
        DEFINITIONS,
        DESCRIPTOR,
        VECTORS,
        RUNTIME,
        "develop/migrations/csharp-03/probes/recursor-feasibility.json",
        W09_CAPACITY,
        W09_FREEZE,
        W09_VECTORS,
    ]);
    let evidence = array(&package["canonical_evidence"]);
    assert_eq!(evidence.len(), expected_evidence.len());
    assert_eq!(
        evidence
            .iter()
            .map(|record| text(&record["path"]))
            .collect::<BTreeSet<_>>(),
        expected_evidence
    );
    let mut evidence_items = BTreeSet::new();
    for record in evidence {
        let path = text(&record["path"]);
        let bytes = read(path);
        let source_document = document(path);
        assert_eq!(text(&record["raw_sha256"]), sha(&bytes), "{path}");
        assert_eq!(record["size_bytes"], bytes.len() as u64, "{path}");
        assert_eq!(record["schema"], source_document["schema"], "{path}");
        assert!(!text(&record["role"]).is_empty());
        evidence_items.insert(text(&record["work_item"]).to_owned());
    }
    assert_eq!(
        evidence_items,
        (1..=9)
            .map(|index| format!("CSHARP-03-T01-W{index:02}"))
            .collect::<BTreeSet<_>>()
    );

    let expected_members = BTreeMap::from([
        (SPEC, "normative_foundation_specification"),
        (W10_PROFILE_SPEC, "normative_profile_specification"),
        (
            W10_SHARED_SPEC,
            "normative_successor_shared_artifact_specification",
        ),
    ]);
    let members = array(&package["specification_members"]);
    assert_eq!(members.len(), expected_members.len());
    for member in members {
        let path = text(&member["path"]);
        let bytes = read(path);
        assert_eq!(text(&member["role"]), expected_members[path]);
        assert_eq!(text(&member["raw_sha256"]), sha(&bytes));
        assert_eq!(member["size_bytes"], bytes.len() as u64);
    }
    let generator = &package["publication_generator"];
    assert_eq!(generator["path"], W10_GENERATOR);
    assert_eq!(generator["role"], "deterministic_publication_generator");
    assert_eq!(text(&generator["raw_sha256"]), sha(&read(W10_GENERATOR)));

    let design = String::from_utf8(read(W10_DESIGN)).expect("UTF-8 design");
    let start_heading = "## 6. Source closure and declaration model";
    let end_heading = "## 24. Implementation stages and gates";
    let start = design.find(start_heading).expect("design projection start");
    let end = design.find(end_heading).expect("design projection end");
    let projection = &design.as_bytes()[start..end];
    let incorporated = &package["incorporated_design"];
    assert_eq!(incorporated["path"], W10_DESIGN);
    assert_eq!(incorporated["start_heading"], start_heading);
    assert_eq!(incorporated["end_before_heading"], end_heading);
    assert_eq!(
        text(&incorporated["raw_projection_sha256"]),
        sha(projection)
    );
    assert_eq!(incorporated["size_bytes"], projection.len() as u64);
}

#[test]
fn csharp_03_t01_w10_freeze_and_downstream_ownership_is_total_and_exact() {
    let package = document(W10_VECTORS);
    let ledger_owners = published_ledger_owners();
    assert_eq!(ledger_owners.len(), 73);

    let freeze_rows = array(&package["freeze_requirement_owners"]);
    assert_eq!(freeze_rows.len(), 10);
    for (index, row) in freeze_rows.iter().enumerate() {
        let work_item = format!("CSHARP-03-T01-W{:02}", index + 1);
        assert_eq!(text(&row["work_item"]), work_item);
        assert_eq!(text(&row["primary_test_owner"]), ledger_owners[&work_item]);
        assert!(text(&row["primary_test_owner"]).ends_with(&work_item));
        assert!(!array(&row["artifacts"]).is_empty());
        assert!(!text(&row["requirement"]).is_empty());
    }

    let expected_items = [(2, 9), (3, 14), (4, 6), (5, 6), (6, 12), (7, 6), (8, 10)]
        .into_iter()
        .flat_map(|(stage, last)| {
            (1..=last).map(move |work| format!("CSHARP-03-T{stage:02}-W{work:02}"))
        })
        .collect::<BTreeSet<_>>();
    let downstream = array(&package["downstream_work_item_owners"]);
    assert_eq!(downstream.len(), 63);
    assert_eq!(
        downstream
            .iter()
            .map(|row| text(&row["work_item"]).to_owned())
            .collect::<BTreeSet<_>>(),
        expected_items
    );
    let downstream_by_item = downstream
        .iter()
        .map(|row| (text(&row["work_item"]), text(&row["primary_test_owner"])))
        .collect::<BTreeMap<_, _>>();
    for row in downstream {
        assert_keys(
            row,
            &[
                "entry_state_at_publication",
                "exit_gate",
                "owns",
                "primary_test_owner",
                "requirement_anchor",
                "title",
                "verification",
                "work_item",
            ],
        );
        let work_item = text(&row["work_item"]);
        assert_eq!(text(&row["primary_test_owner"]), ledger_owners[work_item]);
        assert!(text(&row["primary_test_owner"]).ends_with(work_item));
        assert_eq!(
            row["entry_state_at_publication"],
            if work_item == "CSHARP-03-T02-W01" {
                json!("ready")
            } else {
                json!("serially_blocked")
            }
        );
        assert_eq!(row["requirement_anchor"], format!("{W10_PLAN}#{work_item}"));
        for field in ["title", "owns", "exit_gate", "verification"] {
            assert!(!text(&row[field]).is_empty(), "{work_item}:{field}");
        }
    }
    for row in array(&package["vectors"]) {
        let work_item = text(&row["implementation_owner"]);
        assert_eq!(
            text(&row["production_test_owner"]),
            downstream_by_item[work_item],
            "{}",
            text(&row["id"])
        );
    }

    let inventory = &package["name_owner_inventory"];
    let names = array(&inventory["names"]);
    assert_eq!(names.len(), 243);
    assert_eq!(
        names
            .iter()
            .map(|row| (text(&row["kind"]), text(&row["name"])))
            .collect::<BTreeSet<_>>()
            .len(),
        names.len()
    );
    assert_eq!(
        names
            .iter()
            .filter(|row| row["kind"] == "identity" && row["disposition"] == "successor")
            .count(),
        102
    );
    assert_eq!(
        names
            .iter()
            .filter(|row| row["kind"] == "hash_domain" && row["disposition"] == "successor")
            .count(),
        42
    );
    assert_eq!(
        names
            .iter()
            .filter(|row| row["kind"] == "identity" && row["disposition"] == "retained")
            .count(),
        88
    );
    assert_eq!(
        names
            .iter()
            .filter(|row| row["kind"] == "hash_domain" && row["disposition"] == "retained")
            .count(),
        11
    );
    for row in names {
        assert!(!array(&row["implementation_owners"]).is_empty());
        assert!(!text(&row["family"]).is_empty());
    }

    let shapes = array(&inventory["shapes"]);
    assert_eq!(shapes.len(), 71);
    assert_eq!(
        shapes
            .iter()
            .map(|row| (text(&row["kind"]), text(&row["id"])))
            .collect::<BTreeSet<_>>()
            .len(),
        shapes.len()
    );
    for row in shapes {
        let work_item = text(&row["primary_owner"]);
        assert_eq!(text(&row["primary_test_owner"]), ledger_owners[work_item]);
    }
    let limits = array(&inventory["limits"]);
    assert_eq!(limits.len(), 67);
    for row in limits {
        let work_item = text(&row["primary_owner"]);
        assert_eq!(text(&row["primary_test_owner"]), ledger_owners[work_item]);
    }
    let diagnostics = array(&inventory["diagnostics"]);
    assert_eq!(diagnostics.len(), 29);
    for row in diagnostics {
        assert_eq!(row["primary_owner"], "CSHARP-03-T02-W08");
        assert_eq!(
            text(&row["primary_test_owner"]),
            ledger_owners["CSHARP-03-T02-W08"]
        );
    }
}

#[test]
fn csharp_03_t01_w10_upgrade_gate_and_nonactivation_decisions_are_closed() {
    let package = document(W10_VECTORS);
    let upgrade = &package["upgrade_matrix"];
    assert!(text(&upgrade["future_change_rule"]).contains("new semantic-profile identity"));
    assert!(text(&upgrade["nullable_exception"]).contains("value-type T?"));
    let evidence_paths = array(&package["canonical_evidence"])
        .iter()
        .map(|row| text(&row["path"]))
        .collect::<BTreeSet<_>>();
    let mut forms = BTreeSet::new();
    let excluded = array(&upgrade["excluded_families"]);
    assert_eq!(excluded.len(), 12);
    for family in excluded {
        assert_eq!(
            family["current_disposition"],
            "reject_before_VIR_without_partial_artifacts"
        );
        assert_eq!(family["future_profile_required"], true);
        assert_eq!(family["positive_vectors"], "forbidden");
        assert!(evidence_paths.contains(text(&family["evidence_path"])));
        let owner = text(&family["rejection_owner"]);
        assert!(text(&family["primary_test_owner"]).ends_with(owner));
        for form in array(&family["source_forms_or_claims"]) {
            assert!(forms.insert(text(form)), "duplicate excluded form");
        }
    }

    let observations = array(&upgrade["observation_sets"]);
    assert_eq!(
        observations
            .iter()
            .map(|row| row["case_count"].as_u64().expect("case count"))
            .collect::<Vec<_>>(),
        vec![181, 65, 144, 154]
    );
    for observation in observations {
        let source = document(text(&observation["path"]));
        let source_field = text(&observation["source_field"]);
        let id_field = if source_field == "shape_index" {
            "shape_id"
        } else {
            "mutation_id"
        };
        let mut ids = array(&source[source_field])
            .iter()
            .map(|row| text(&row[id_field]))
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(
            ids.iter().copied().collect::<BTreeSet<_>>().len(),
            ids.len()
        );
        assert_eq!(observation["case_count"], ids.len() as u64);
        assert_eq!(
            text(&observation["ids_sha256"]),
            sha(&canonical(&json!(ids)))
        );
    }

    let gate = &package["release_gate"];
    assert_eq!(
        gate["candidate_and_release_command"],
        "sudo ./scripts/check-csharp-practical-release.sh"
    );
    assert_eq!(gate["implementation_owner"], "CSHARP-03-T07-W05");
    assert_eq!(gate["receipt_owner"], "CSHARP-03-T07-W06");
    assert_eq!(gate["activation_owner"], "CSHARP-03-T08-W10");
    assert_eq!(
        gate["invocation_owners"],
        json!([
            "CSHARP-03-T07-W05",
            "CSHARP-03-T07-W06",
            "CSHARP-03-T08-W06",
            "CSHARP-03-T08-W09",
            "CSHARP-03-T08-W10"
        ])
    );
    assert_eq!(
        gate["post_activation_relation"],
        "replace_and_retire_java_named_gate_atomically"
    );
    assert_eq!(gate["pre_activation_gate_path"], JAVA_GATE);
    assert_eq!(gate["aggregate_gate_path"], AGGREGATE_GATE);
    assert_eq!(gate["practical_gate_path"], PRACTICAL_GATE);
    assert_eq!(
        text(&gate["pre_activation_gate_raw_sha256"]),
        sha(&read(JAVA_GATE))
    );
    assert_eq!(
        text(&gate["aggregate_gate_predecessor_raw_sha256"]),
        sha(&read(AGGREGATE_GATE))
    );
    assert!(!root().join(PRACTICAL_GATE).exists());
    let aggregate = String::from_utf8(read(AGGREGATE_GATE)).expect("UTF-8 gate");
    assert!(aggregate.contains(JAVA_GATE));
    assert!(!aggregate.contains(PRACTICAL_GATE));

    let active = format!(
        "{}{}",
        String::from_utf8(read(ACTIVE_SEMANTIC_REGISTRY)).expect("semantic registry UTF-8"),
        String::from_utf8(read(ACTIVE_BUNDLE_REGISTRY)).expect("bundle registry UTF-8")
    );
    assert!(!active.contains("mpk.csharp.practical"));
    assert!(!active.contains("csharp-practical"));

    let extension = &package["historical_inventory_extension"];
    assert_eq!(extension["baseline_inventory_path"], W09_INVENTORY);
    assert_eq!(
        text(&extension["baseline_inventory_raw_sha256"]),
        sha(&read(W09_INVENTORY))
    );
    assert_eq!(extension["owner"], "CSHARP-03-T01-W10");
    assert_eq!(
        extension["owner_test"],
        format!("{OWNER}#CSHARP-03-T01-W10")
    );
    assert_eq!(
        extension["publication_paths"],
        json!([W10_PROFILE_SPEC, W10_SHARED_SPEC, W10_VECTORS])
    );
}

#[test]
fn csharp_03_t01_w10_manifest_specs_and_generator_are_reproducible() {
    let package_bytes = read(W10_VECTORS);
    let package = document(W10_VECTORS);
    let manifest = document(W10_MANIFEST);
    let entries = array(&manifest["vectors"])
        .iter()
        .filter(|entry| entry["path"] == W10_VECTORS)
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    let entry = entries[0];
    assert_eq!(entry["schema_id"], package["schema"]);
    assert_eq!(text(&entry["sha256"]), sha(&package_bytes));
    assert_eq!(entry["owning_spec"], W10_PROFILE_SPEC);
    assert_eq!(entry["implementation_test_owners"], json!([OWNER]));

    let profile = String::from_utf8(read(W10_PROFILE_SPEC)).expect("profile spec UTF-8");
    let shared = String::from_utf8(read(W10_SHARED_SPEC)).expect("shared spec UTF-8");
    for contents in [&profile, &shared] {
        assert!(contents.contains(W10_VECTORS));
        assert!(contents.contains("mpk.csharp.practical.profile.conformance.v1"));
        assert!(contents.contains("crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10"));
        assert!(!contents.contains("MetisDoc"));
    }
    assert!(profile.contains(W10_SHARED_SPEC));
    assert!(profile.contains("mpk.csharp.practical.v1"));
    assert!(profile.contains("normative, frozen, and inactive"));
    assert!(shared.contains("17 identity families"));
    assert!(shared.contains("102 globally unique successor"));
    assert!(shared.contains("Atomic migration and rollback"));

    let output = Command::new("python3")
        .arg(root().join(W10_GENERATOR))
        .arg("--check")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root())
        .output()
        .expect("run W10 publication generator");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn published_ledger_owners() -> BTreeMap<String, String> {
    let ledger = String::from_utf8(read(W10_LEDGER)).expect("ledger UTF-8");
    let section = ledger
        .split("<!-- work-item-ledger:start -->")
        .nth(1)
        .expect("ledger start")
        .split("<!-- work-item-ledger:end -->")
        .next()
        .expect("ledger end");
    let mut result = BTreeMap::new();
    for line in section
        .lines()
        .filter(|line| line.starts_with("| `CSHARP-03-T"))
    {
        let fields = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        let work_item = fields[0].trim_matches('`').to_owned();
        let owner = fields[2].trim_matches('`').to_owned();
        assert!(
            result.insert(work_item.clone(), owner).is_none(),
            "{work_item}"
        );
    }
    result
}
