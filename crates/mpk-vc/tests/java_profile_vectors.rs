//! T03's executable contract and artifact owner. Hand-authored VIR fixtures
//! exercise admission, not javac/source lowering (owned by T04-T06).

use mpk_vc::semantic_profile_registry::*;
use mpk_vc::successor_source_artifacts::*;
use mpk_vc::successor_vc::{generate_successor_vc, import_successor_vc_json, SuccessorVcSource};
use mpk_vc::{
    canonical_json_bytes, hash_domain_separated_raw, parse_strict_json, sha256_raw_file_bytes,
    CapturedInput, HashDomain, InputKind, StrictJsonLimits, VcTerm,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const PROFILE: &[u8] = include_bytes!("../../../develop/specs/vectors/java-profile-v0.json");
const REGISTRY: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v3.json");
const ACTIVE: &[u8] = include_bytes!("../../../release/bundles/semantic-profile-registry.json");
const BUNDLES: &[u8] = include_bytes!("../../../release/bundles/bundle-registry.json");
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn load(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}
fn canonical(value: &Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).unwrap();
    canonical_json_bytes(
        &parse_strict_json(
            &encoded,
            StrictJsonLimits::new(16_777_216, 2_000_000, 256, 4_194_304),
        )
        .unwrap(),
    )
    .unwrap()
}
fn registry() -> ValidatedSemanticProfileRegistry {
    validate_semantic_profile_registry(
        &canonical_registry_transport(&load(REGISTRY)["registry"]).unwrap(),
        RegistryRevision::Revision3,
    )
    .unwrap()
}
fn context() -> Value {
    load(PROFILE)["semantic_context_fixture"].clone()
}
fn request(selection: Value) -> Value {
    json!({"semantic_context":context(), "selection":selection})
}
fn base_selection() -> Value {
    load(PROFILE)["selection_fixture"].clone()
}
fn profile_contract(field: &str) -> Value {
    load(PROFILE)["profile_contracts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["field"] == field)
        .unwrap()["envelope"]
        .clone()
}
fn mutate(value: &mut Value, case: &Value) {
    let pointer = case["pointer"].as_str().unwrap();
    match case["operation"].as_str().unwrap() {
        "replace" => *value.pointer_mut(pointer).unwrap() = case["value"].clone(),
        "add" | "remove" => {
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            let object = value.pointer_mut(parent).unwrap().as_object_mut().unwrap();
            if case["operation"] == "add" {
                assert!(object
                    .insert(key.to_owned(), case["value"].clone())
                    .is_none());
            } else {
                assert!(object.remove(key).is_some());
            }
        }
        other => panic!("unowned mutation {other}"),
    }
}
fn rehash_registry(value: &mut Value) {
    for entry in value["profiles"].as_array_mut().unwrap() {
        entry["entry_sha256"] = json!(semantic_profile_entry_hash(entry).unwrap());
    }
    value["registry_sha256"] = json!(semantic_profile_registry_hash(value).unwrap());
}
fn domain(name: &str) -> HashDomain {
    HashDomain::new(match name {
        "MPK-JAVA-SELECTION-0.1" => "MPK-JAVA-SELECTION-0.1",
        "MPK-JAVA-CONTRACT-SIDECAR-0.1" => "MPK-JAVA-CONTRACT-SIDECAR-0.1",
        "MPK-JAVA-TOOLCHAIN-INPUTS-0.1" => "MPK-JAVA-TOOLCHAIN-INPUTS-0.1",
        "MPK-CONTRACT-1.0" => "MPK-CONTRACT-1.0",
        "MPK-SEMANTIC-PROFILE-ENTRY-1.0" => "MPK-SEMANTIC-PROFILE-ENTRY-1.0",
        "MPK-SEMANTIC-PROFILE-REGISTRY-1.0" => "MPK-SEMANTIC-PROFILE-REGISTRY-1.0",
        _ => panic!("unowned hash domain {name}"),
    })
}

#[test]
fn frozen_registry_and_all_nine_contracts_execute_compiled_dispatch() {
    let manifest = load(include_bytes!(
        "../../../develop/specs/vectors/manifest.json"
    ));
    for (path, bytes) in [
        ("develop/specs/vectors/java-profile-v0.json", PROFILE),
        (
            "develop/specs/vectors/semantic-profile-registry-v3.json",
            REGISTRY,
        ),
    ] {
        let entry = manifest["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["path"] == path)
            .unwrap();
        assert_eq!(entry["sha256"], sha256_raw_file_bytes(bytes).to_hex());
        assert!(entry["implementation_test_owners"]
            .as_array()
            .unwrap()
            .contains(&json!("crates/mpk-vc/tests/java_profile_vectors.rs")));
    }
    let vectors = load(REGISTRY);
    assert_eq!(
        vectors["schema"],
        "mpk.semantic_profile.registry.conformance.v3"
    );
    assert_eq!(
        vectors["owner_test"],
        "crates/mpk-vc/tests/java_profile_spec.rs"
    );
    assert_eq!(
        vectors["mechanism_spec"],
        "develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md"
    );
    assert_eq!(vectors["profile_spec"], "develop/specs/JAVA_PROFILE_V0.md");
    let registry = registry();
    let previous = validate_semantic_profile_registry(ACTIVE, RegistryRevision::Revision2).unwrap();
    validate_revision_3_append_only(&previous, &registry).unwrap();
    assert!(validate_revision_3_append_only(&registry, &previous).is_err());
    assert!(validate_revision_3_append_only(&registry, &registry).is_err());
    for case in vectors["append_only_cases"].as_array().unwrap() {
        match case["id"].as_str().unwrap() {
            "append.exact_count" => {
                assert_eq!(registry.entries().len() as u64, case["expected_count"])
            }
            "append.exact_order" => assert_eq!(
                json!(registry
                    .entries()
                    .iter()
                    .map(|e| e.source_language())
                    .collect::<Vec<_>>()),
                case["expected_languages"]
            ),
            "append.csharp_bytes_unchanged"
            | "append.go_bytes_unchanged"
            | "append.rust_bytes_unchanged" => {
                assert_eq!(
                    previous.entries()[case["predecessor_index"].as_u64().unwrap() as usize]
                        .canonical_json(),
                    registry.entries()[case["successor_index"].as_u64().unwrap() as usize]
                        .canonical_json()
                );
            }
            "append.no_later_language" => {
                for language in case["absent_languages"].as_array().unwrap() {
                    assert!(registry
                        .entries()
                        .iter()
                        .all(|e| e.source_language() != language.as_str().unwrap()));
                }
            }
            other => panic!("unowned append case {other}"),
        }
    }
    assert_eq!(
        canonical_registry_transport(&vectors["predecessor"]).unwrap(),
        ACTIVE
    );
    assert!(previous.lookup("java", JAVA_SCALAR_PROFILE).is_none());
    assert_eq!(load(BUNDLES)["tuples"].as_array().unwrap().len(), 4);
    assert!(load(BUNDLES)["tuples"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["semantic_context"]["source_language"] != "java"));
    let profile = load(PROFILE);
    assert_eq!(profile["schema"], "mpk.java.profile.conformance.v0");
    let identity = &profile["profile_identity"];
    let compiled = CompiledSemanticProfile::from_identity(
        identity["source_language"].as_str().unwrap(),
        identity["semantic_profile"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(compiled, CompiledSemanticProfile::JavaScalarV0);
    assert_eq!(
        compiled.semantic_parameters_schema(),
        identity["semantic_parameters_schema"]
    );
    assert_eq!(compiled.selection_schema(), identity["selection_schema"]);
    assert_eq!(compiled.entry_sha256(), identity["profile_entry_sha256"]);
    assert_eq!(
        registry.identity().registry_sha256(),
        identity["registry_sha256"]
    );
    assert_eq!(
        registry.identity().revision(),
        identity["registry_revision"]
    );
    assert_eq!(
        registry
            .lookup("java", JAVA_SCALAR_PROFILE)
            .unwrap()
            .canonical_json(),
        canonical(&vectors["java_entry"])
    );
    for contract in profile["profile_contracts"].as_array().unwrap() {
        let field = ProfileContractField::from_name(contract["field"].as_str().unwrap()).unwrap();
        let envelope = &contract["envelope"];
        let accepted = validate_compiled_profile_envelope(&registry, envelope, field).unwrap();
        assert_eq!(
            accepted.contract_id(),
            registry
                .lookup("java", JAVA_SCALAR_PROFILE)
                .unwrap()
                .contracts()
                .contract_id(field)
        );
        for key in envelope["value"].as_object().unwrap().keys() {
            let mut changed = envelope.clone();
            changed["value"][key] = json!("tampered");
            assert!(
                validate_compiled_profile_envelope(&registry, &changed, field).is_err(),
                "{}.{key}",
                field.as_str()
            );
        }
    }
    let accepted = validate_semantic_request(&registry, &request(base_selection())).unwrap();
    assert_eq!(
        java_selection_hash(accepted.selection()).unwrap(),
        profile["selection_sha256"]
    );
    assert!(validate_semantic_request(&previous, &request(base_selection())).is_err());
    for revision in [RegistryRevision::Revision1, RegistryRevision::Revision2] {
        assert!(validate_semantic_profile_registry(
            &canonical_registry_transport(&vectors["registry"]).unwrap(),
            revision
        )
        .is_err());
    }
    assert!(validate_semantic_profile_registry(ACTIVE, RegistryRevision::Revision3).is_err());
}

#[test]
fn frozen_identity_payload_registry_hash_and_limit_mutations_reject() {
    let registry = registry();
    let profile = load(PROFILE);
    let mut sidecar_owners = BTreeSet::new();
    let mut executed = 0;
    for case in profile["mutation_cases"].as_array().unwrap() {
        let path = case["fixture"].as_str().unwrap();
        if path == "/contract_fixture" {
            sidecar_owners.insert(case["id"].as_str().unwrap());
            continue; // T05 owns raw sidecar parsing; normalized contracts are tested below.
        }
        let mut value = profile.pointer(path).unwrap().clone();
        mutate(&mut value, case);
        let rejected = match path {
            "/semantic_parameters" => {
                let mut ctx = context();
                ctx["semantic_parameters"] = value;
                validate_registry_semantic_context(&registry, &ctx).is_err()
            }
            "/semantic_context_fixture" => {
                validate_registry_semantic_context(&registry, &value).is_err()
            }
            "/selection_fixture" => validate_semantic_request(&registry, &request(value)).is_err(),
            _ if path.starts_with("/profile_contracts/") => {
                let i: usize = path.split('/').nth(2).unwrap().parse().unwrap();
                let field = ProfileContractField::from_name(
                    profile["profile_contracts"][i]["field"].as_str().unwrap(),
                )
                .unwrap();
                validate_compiled_profile_envelope(&registry, &value, field).is_err()
            }
            other => panic!("unowned T03 fixture {other}"),
        };
        assert!(rejected, "mutation admitted: {}", case["id"]);
        executed += 1;
    }
    assert_eq!(
        sidecar_owners,
        BTreeSet::from([
            "sidecar.profile",
            "sidecar.modifies",
            "sidecar.result_in_requires",
            "sidecar.unsigned",
            "sidecar.unknown"
        ])
    );
    assert_eq!(executed, 48);
    let vectors = load(REGISTRY);
    for case in vectors["mutation_cases"].as_array().unwrap() {
        let mut value = vectors["registry"].clone();
        mutate(&mut value, case);
        if case["repair_hashes"] == true {
            rehash_registry(&mut value)
        }
        assert!(
            validate_semantic_profile_registry(
                &canonical_registry_transport(&value).unwrap(),
                RegistryRevision::Revision3
            )
            .is_err(),
            "{}",
            case["id"]
        );
    }
    for case in profile["shared_envelope_limits"].as_array().unwrap() {
        let limit = SemanticRegistryLimit::from_id(case["id"].as_str().unwrap()).unwrap();
        let maximum = case["maximum"].as_u64().unwrap();
        validate_semantic_registry_limit(limit, maximum).unwrap();
        assert!(validate_semantic_registry_limit(limit, maximum + 1).is_err());
    }
    for (root, cases) in [
        (&profile, &profile["hash_cases"]),
        (&vectors["registry"], &vectors["hash_cases"]),
    ] {
        for case in cases.as_array().unwrap() {
            let source = root
                .pointer(case["source_pointer"].as_str().unwrap())
                .unwrap();
            let mut payload = source.clone();
            if let Some(field) = case["excluded_field"].as_str() {
                payload.as_object_mut().unwrap().remove(field).unwrap();
            }
            let bytes = canonical(&payload);
            assert_eq!(bytes.len() as u64, case["expected_payload_utf8_length"]);
            let name = case["domain"].as_str().unwrap();
            assert_eq!(
                (bytes.len() + name.len() + 1) as u64,
                case["expected_preimage_length"]
            );
            assert_eq!(
                hash_domain_separated_raw(domain(name), &bytes)
                    .unwrap()
                    .to_hex(),
                case["expected_sha256"]
            );
            if let Some(length) = case.get("expected_complete_jcs_utf8_length") {
                assert_eq!(canonical(source).len() as u64, length.as_u64().unwrap());
            }
            if let Some(length) = case.get("expected_transport_utf8_length") {
                let transport = canonical_registry_transport(source).unwrap();
                assert_eq!(transport.len() as u64, length.as_u64().unwrap());
                assert_eq!(
                    sha256_raw_file_bytes(&transport).to_hex(),
                    case["expected_transport_sha256"]
                );
            }
        }
    }
    assert_eq!(
        successor_contract_hash_value(&profile["normalized_contract_fixture"])
            .unwrap()
            .as_str(),
        profile["normalized_contract_fixture"]["contract_hash"]
    );
}

#[test]
fn selection_validates_names_paths_order_aliases_and_closed_values() {
    let registry = registry();
    let reject = |field: &str, value: Value| {
        let mut selection = base_selection();
        selection["value"][field] = value.clone();
        assert!(
            validate_semantic_request(&registry, &request(selection)).is_err(),
            "admitted {field}: {value}"
        );
    };
    for value in [
        "", "A", "1a", "a..b", "a--b", "a__b", "a-", "a b", "a/b", "a$b",
    ] {
        reject("compilation", json!(value));
    }
    reject("compilation", json!("a".repeat(65)));
    for value in ["a", "a0.b_c-d", &"a".repeat(64)] {
        let mut selection = base_selection();
        selection["value"]["compilation"] = json!(value);
        validate_semantic_request(&registry, &request(selection)).unwrap();
    }
    for path in [
        "/src/a/A.java",
        "src/A.java",
        "src/a/../A.java",
        "src/a//A.java",
        "src/a/./A.java",
        "src/a/A.java/",
        "src/a/A.JAVA",
        "src/a/A.cs",
        "src/a\\A.java",
        "src/a:b/A.java",
        "src/var/A.java",
        "src/a/_.java",
        "src/a/When$.java",
        "src/é/A.java",
        "src/a/CON.java",
        "src/a/A..java",
        "src/java/A.java",
        "src/java/sub/A.java",
        "src/javax/A.java",
        "src/jdk/A.java",
        "src/sun/A.java",
        "src/com/sun/A.java",
        "src/com/sun/sub/A.java",
    ] {
        reject("sources", json!([path]));
    }
    for path in [
        "src/javaxish/A.java",
        "src/com/sunny/A.java",
        "src/a/_A1.java",
    ] {
        let mut selection = base_selection();
        selection["value"]["sources"] = json!([path]);
        validate_semantic_request(&registry, &request(selection)).unwrap();
    }
    for paths in [
        json!([]),
        json!(["src/a/A.java", "src/a/A.java"]),
        json!(["src/a/Z.java", "src/a/A.java"]),
        json!(["src/A/F.java", "src/a/G.java"]),
        json!(["src/a/F.java", "src/a/f.java"]),
    ] {
        reject("sources", paths);
    }
    for paths in [
        json!(["contracts/../x.json"]),
        json!(["contracts/x.cs"]),
        json!(["contracts/A/x.json", "contracts/a/y.json"]),
        json!(["contracts/a.json", "contracts/a.json/b.json"]),
    ] {
        reject("contracts", paths);
    }
    for method in [
        "Case::run()->int",
        "a.Case.run()->int",
        "a.Case::run()",
        "a.Case::run( int)->int",
        "a.Case::run(i32)->int",
        "a.Case::run(int,)->int",
        "a.Case::run(long)->void",
        "a.Case::run()->unsigned",
        "a.Case::run$()->int",
        "a.Case::var()->int",
        "a.record::run()->int",
        "java.Case::run()->int",
        "com.sun.Case::run()->int",
        "a.Case::run()->int\n",
    ] {
        reject("methods", json!([method]));
    }
    let parameters = vec!["long"; 128].join(",");
    reject(
        "methods",
        json!([format!("a.Case::run({parameters})->int")]),
    );
    for methods in [
        json!([]),
        json!(["a.A::f()->int", "a.A::f()->int"]),
        json!(["a.A::g()->int", "a.A::f()->int"]),
    ] {
        reject("methods", methods);
    }
    let mut selection = base_selection();
    selection["value"]["methods"] = json!([format!(
        "a.Case::_f1({},int)->long",
        vec!["long"; 127].join(",")
    )]);
    validate_semantic_request(&registry, &request(selection)).unwrap();
    for (field, max, pattern) in [
        ("sources", 256, "src/a/S"),
        ("contracts", 128, "contracts/c"),
        ("methods", 32, "a.A::f"),
    ] {
        let make = |n| {
            json!((0..n)
                .map(|i| match field {
                    "sources" => format!("{pattern}{i:03}.java"),
                    "contracts" => format!("{pattern}{i:03}.json"),
                    _ => format!("{pattern}{i:03}()->int"),
                })
                .collect::<Vec<_>>())
        };
        let mut selection = base_selection();
        selection["value"][field] = make(max);
        validate_semantic_request(&registry, &request(selection)).unwrap();
        reject(field, make(max + 1));
    }
}

fn ty(token: &str) -> Value {
    match token {
        "boolean" => json!({"kind":"bool"}),
        "int" => json!({"kind":"bv","width":32,"signed":true}),
        "long" => json!({"kind":"bv","width":64,"signed":true}),
        "u32" => json!({"kind":"bv","width":32,"signed":false}),
        "u64" => json!({"kind":"bv","width":64,"signed":false}),
        _ => panic!("test type {token}"),
    }
}
fn var(id: &str) -> Value {
    json!({"var":id})
}
fn bin(id: &str, op: &str, token: &str, lhs: Value, rhs: Value) -> Value {
    json!({"kind":"BinOp","id":id,"op":op,"type":ty(token),"lhs":lhs,"rhs":rhs,
        "safety_checks":if matches!(op,"bv_sdiv"|"bv_srem") { json!([{"kind":"divisor_nonzero"}]) } else { json!([]) }})
}
fn convert(id: &str, token: &str, value: Value) -> Value {
    json!({"kind":"Convert","id":id,"type":ty(token),"value":value,"safety_checks":[]})
}
fn constant(id: &str, token: &str, value: &str) -> Value {
    json!({"kind":"Const","id":id,"type":ty(token),"value":{"int":{"value":value,"width":ty(token)["width"],"signed":ty(token)["signed"]}},"safety_checks":[]})
}
fn function(
    name: &str,
    parameters: &[&str],
    result: &str,
    instructions: Vec<Value>,
    returned: Value,
) -> Value {
    let id = format!("vector.Case::{name}({})->{result}", parameters.join(","));
    let features = instructions
        .iter()
        .filter_map(|i| match i["kind"].as_str().unwrap() {
            "CallStatic" => Some("call_static"),
            "Convert" => Some("conversion"),
            "Copy" => Some("mutable_local"),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    json!({"id":id,"unit_id":"java-test","name":name,
        "params":parameters.iter().enumerate().map(|(i,t)|json!({"id":format!("arg{i}"),"type":ty(t)})).collect::<Vec<_>>(),
        "results":[{"id":"result0","type":ty(result)}],"locals":[],
        "blocks":[{"label":"bb0","parameters":[],"instructions":instructions,"terminator":{"kind":"Return","values":[returned]}}],
        "contracts":{"semantic_context":context(),"unit_id":"java-test","function_id":id,
            "requires":[],"ensures":[{"bool":true}],"modifies":[],"panic":"forbidden","termination":"total","loops":[],"contract_hash":ZERO},
        "features_used":features})
}
fn rehash_vir(value: &mut Value) {
    let mut contracts = BTreeMap::new();
    for function in value["units"][0]["functions"].as_array_mut().unwrap() {
        let contract = &mut function["contracts"];
        contract["contract_hash"] =
            json!(successor_contract_hash_value(contract).unwrap().as_str());
        contracts.insert(
            function["id"].as_str().unwrap().to_owned(),
            function["contracts"]["contract_hash"].clone(),
        );
    }
    for function in value["units"][0]["functions"].as_array_mut().unwrap() {
        for block in function["blocks"].as_array_mut().unwrap() {
            for instruction in block["instructions"].as_array_mut().unwrap() {
                if instruction["kind"] == "CallStatic" {
                    if let Some(hash) = contracts.get(instruction["function"].as_str().unwrap()) {
                        instruction["contract_hash"] = hash.clone();
                    }
                }
            }
        }
    }
    value["vir_hash"] = json!(successor_vir_hash_value(value).unwrap().as_str());
}
fn module(functions: Vec<Value>) -> Value {
    let mut value = json!({"schema":"mpk.vir.v1","semantic_context":context(),"units":[{"id":"java-test","name":"java-test","type_decls":[],"const_decls":[],"functions":functions}],"vir_hash":ZERO});
    rehash_vir(&mut value);
    value
}
fn accept_vir(value: &Value) -> ValidatedSuccessorVir {
    import_successor_vir_json(&canonical(value), &registry()).unwrap()
}
fn reject_vir(mut value: Value) {
    rehash_vir(&mut value); // Correct helper hashes cannot authorize an excluded operation.
    assert!(
        import_successor_vir_json(&canonical(&value), &registry()).is_err(),
        "admitted: {value}"
    );
}
fn arithmetic(op: &str, token: &str) -> Value {
    module(vec![function(
        "run",
        &[token, token],
        token,
        vec![bin("t0", op, token, var("arg0"), var("arg1"))],
        var("t0"),
    )])
}
fn shifted(op: &str, token: &str) -> Value {
    let mut instructions = vec![
        constant("t0", "int", if token == "int" { "31" } else { "63" }),
        bin("t1", "bv_and", "int", var("arg1"), var("t0")),
    ];
    let result = if op == "bv_lshr" {
        let unsigned = if token == "int" { "u32" } else { "u64" };
        instructions.extend([
            convert("t2", unsigned, var("arg0")),
            bin("t3", op, unsigned, var("t2"), var("t1")),
            convert("t4", token, var("t3")),
        ]);
        "t4"
    } else {
        instructions.push(bin("t2", op, token, var("arg0"), var("t1")));
        "t2"
    };
    module(vec![function(
        "run",
        &[token, "int"],
        token,
        instructions,
        var(result),
    )])
}

#[test]
fn java_vir_admits_exact_scalar_operations_conversions_and_check_sequences() {
    for token in ["int", "long"] {
        for op in [
            "bv_add", "bv_sub", "bv_mul", "bv_and", "bv_or", "bv_xor", "bv_sdiv", "bv_srem",
        ] {
            let base = arithmetic(op, token);
            accept_vir(&base);
            for checks in [
                json!([{"kind":"integer_no_overflow","operation":"add","signed":true}]),
                json!([{"kind":"signed_divrem_representable","operation":"div"}]),
                json!([{"kind":"divisor_nonzero"},{"kind":"divisor_nonzero"}]),
                json!([{"kind":"shift_count_nonnegative"}]),
                json!([{"kind":"divisor_nonzero"},{"kind":"signed_divrem_representable","operation":"div"}]),
                json!([{"kind":"signed_divrem_representable","operation":"div"},{"kind":"divisor_nonzero"}]),
            ] {
                let mut value = base.clone();
                value["units"][0]["functions"][0]["blocks"][0]["instructions"][0]
                    ["safety_checks"] = checks;
                reject_vir(value);
            }
            if matches!(op, "bv_sdiv" | "bv_srem") {
                let mut value = base;
                value["units"][0]["functions"][0]["blocks"][0]["instructions"][0]
                    ["safety_checks"] = json!([]);
                reject_vir(value);
            }
        }
        for op in [
            "signed_lt",
            "signed_le",
            "signed_gt",
            "signed_ge",
            "eq",
            "not_eq",
        ] {
            accept_vir(&module(vec![function(
                "run",
                &[token, token],
                "boolean",
                vec![bin("t0", op, "boolean", var("arg0"), var("arg1"))],
                var("t0"),
            )]));
        }
        for op in ["bv_neg", "bv_not"] {
            accept_vir(&module(vec![function(
                "run",
                &[token],
                token,
                vec![
                    json!({"kind":"UnaryOp","id":"t0","op":op,"type":ty(token),"value":var("arg0"),"safety_checks":[]}),
                ],
                var("t0"),
            )]));
        }
        for op in [
            "bv_udiv",
            "bv_urem",
            "unsigned_lt",
            "unsigned_le",
            "unsigned_gt",
            "unsigned_ge",
        ] {
            reject_vir(arithmetic(op, token));
        }
    }
    for (from, to) in [("int", "long"), ("long", "int")] {
        accept_vir(&module(vec![function(
            "run",
            &[from],
            to,
            vec![convert("t0", to, var("arg0"))],
            var("t0"),
        )]));
    }
    reject_vir(module(vec![function(
        "run",
        &["int"],
        "int",
        vec![convert("t0", "int", var("arg0"))],
        var("t0"),
    )]));
    for op in ["eq", "not_eq"] {
        accept_vir(&module(vec![function(
            "run",
            &["boolean", "boolean"],
            "boolean",
            vec![bin("t0", op, "boolean", var("arg0"), var("arg1"))],
            var("t0"),
        )]));
    }
    accept_vir(&module(vec![function(
        "run",
        &["boolean"],
        "boolean",
        vec![
            json!({"kind":"UnaryOp","id":"t0","op":"not","type":ty("boolean"),"value":var("arg0"),"safety_checks":[]}),
        ],
        var("t0"),
    )]));
    for op in ["bv_and", "bv_or", "bv_xor"] {
        reject_vir(arithmetic(op, "boolean"));
    }
}

#[test]
fn every_shift_requires_its_exact_linked_mask_and_closed_unsigned_pattern() {
    for token in ["int", "long"] {
        for op in ["bv_shl", "bv_ashr", "bv_lshr"] {
            let base = shifted(op, token);
            accept_vir(&base);
            let shift = if op == "bv_lshr" { 3 } else { 2 };
            for (pointer, replacement) in [
                (
                    "/0/value/int/value",
                    json!(if token == "int" { "63" } else { "31" }),
                ),
                ("/0/type/width", json!(64)),
                ("/0/value/int/signed", json!(false)),
                ("/1/lhs", var("t0")),
                ("/1/rhs", var("arg1")),
                ("/1/type/width", json!(64)),
                (
                    "/1/safety_checks",
                    json!([{"kind":"shift_count_nonnegative"}]),
                ),
            ] {
                let mut value = base.clone();
                *value["units"][0]["functions"][0]["blocks"][0]["instructions"]
                    .pointer_mut(pointer)
                    .unwrap() = replacement;
                // Changing only the mask's LHS to its own constant is legal as
                // a constant count. Swap both operands to test the forbidden ordering.
                if pointer == "/1/lhs" {
                    value["units"][0]["functions"][0]["blocks"][0]["instructions"][1]["rhs"] =
                        var("arg1");
                }
                reject_vir(value);
            }
            for count in [
                var("arg1"),
                var("t0"),
                json!({"int":{"value":"1","width":32,"signed":true}}),
            ] {
                let mut value = base.clone();
                value["units"][0]["functions"][0]["blocks"][0]["instructions"][shift]["rhs"] =
                    count;
                reject_vir(value);
            }
            let mut value = base.clone();
            value["units"][0]["functions"][0]["blocks"][0]["instructions"][shift]
                ["safety_checks"] = json!([{"kind":"shift_count_nonnegative"}]);
            reject_vir(value);
            if op == "bv_lshr" {
                for (pointer, replacement) in [
                    ("/2/type/width", json!(if token == "int" { 64 } else { 32 })),
                    ("/3/lhs", var("arg0")),
                    ("/4/value", var("t2")),
                ] {
                    let mut value = base.clone();
                    *value["units"][0]["functions"][0]["blocks"][0]["instructions"]
                        .pointer_mut(pointer)
                        .unwrap() = replacement;
                    reject_vir(value);
                }
                for id in ["t2", "t3"] {
                    let mut value = base.clone();
                    value["units"][0]["functions"][0]["blocks"][0]["terminator"]["values"] =
                        json!([var(id)]);
                    reject_vir(value);
                }
                let mut value = base.clone();
                value["units"][0]["functions"][0]["blocks"][0]["instructions"]
                    .as_array_mut()
                    .unwrap()
                    .push(convert("t5", token, var("t3")));
                reject_vir(value);
                let mut value = base.clone();
                value["units"][0]["functions"][0]["blocks"][0]["instructions"][4] =
                    convert("t4", token, var("arg0"));
                reject_vir(value);
            }
        }
    }
    // Negative/oversized source counts are not rejected or given overflow VCs.
    for count in ["-1", "32", "64", "2147483647", "-2147483648"] {
        let mut value = shifted("bv_shl", "long");
        value["units"][0]["functions"][0]["blocks"][0]["instructions"][1]["lhs"] =
            json!({"int":{"value":count,"width":32,"signed":true}});
        rehash_vir(&mut value);
        accept_vir(&value);
    }
}

#[test]
fn repaired_hashes_cannot_admit_bad_types_flow_names_or_normalized_contracts() {
    let base = arithmetic("bv_add", "int");
    let f = "/units/0/functions/0";
    for (tail, replacement) in [
        ("/name", json!("another")),
        ("/params/0/id", json!("arg1")),
        ("/params/0/type", ty("u32")),
        ("/params/0/type/width", json!(8)),
        ("/results", json!([])),
        ("/results/0/type", ty("long")),
        ("/blocks", json!([])),
        ("/blocks/0/label", json!("bb1")),
        ("/blocks/0/instructions/0/id", json!("t1")),
        ("/blocks/0/instructions/0/lhs", var("t0")),
        ("/blocks/0/instructions/0/rhs", var("unlisted")),
        ("/blocks/0/terminator/values", json!([{"bool":true}])),
        (
            "/blocks/0/terminator",
            json!({"kind":"Jump","label":"bb0","args":[]}),
        ),
        ("/contracts/ensures", json!([])),
        ("/contracts/termination", json!("partial")),
        (
            "/contracts/requires",
            json!([{"op":"eq","lhs":{"result":0},"rhs":{"int":{"value":"0","width":32,"signed":true}}}]),
        ),
        (
            "/contracts/ensures",
            json!([{"op":"eq","lhs":{"int":{"value":"-1","width":32,"signed":false}},"rhs":{"int":{"value":"-1","width":32,"signed":false}}}]),
        ),
        (
            "/contracts/ensures",
            json!([{"op":"eq","lhs":{"op":"convert","type":ty("long"),"value":var("arg0")},"rhs":{"int":{"value":"0","width":64,"signed":true}}}]),
        ),
        (
            "/contracts/ensures",
            json!([{"op":"eq","lhs":{"op":"bv_sdiv","lhs":var("arg0"),"rhs":var("arg1")},"rhs":{"int":{"value":"0","width":32,"signed":true}}}]),
        ),
        ("/features_used", json!(["conversion"])),
    ] {
        let mut value = base.clone();
        *value.pointer_mut(&format!("{f}{tail}")).unwrap() = replacement;
        reject_vir(value);
    }
    for kind in ["Field", "Index", "MakeArray", "MakeStruct"] {
        let mut value = base.clone();
        value["units"][0]["functions"][0]["blocks"][0]["instructions"][0]["kind"] = json!(kind);
        reject_vir(value);
    }
    let mut value = base.clone();
    value["units"][0]["type_decls"] = json!([{"id":"vector.T","name":"T","fields":[]}]);
    reject_vir(value);
    let mut value = base.clone();
    value["units"][0]["name"] = json!("other");
    reject_vir(value);
    for identity in [
        "vector.Case::run(u32,int)->int",
        "vector.Case::run(int,int)->long",
        "vector.Case::run(int)->int",
        "java.Case::run(int,int)->int",
    ] {
        let mut value = base.clone();
        value["units"][0]["functions"][0]["id"] = json!(identity);
        value["units"][0]["functions"][0]["contracts"]["function_id"] = json!(identity);
        reject_vir(value);
    }
    let mut second = function("run", &["long"], "long", vec![], var("arg0"));
    let first = base["units"][0]["functions"][0].clone();
    reject_vir(module(vec![first, second.clone()])); // Even resolvable overloads are excluded.
    second["contracts"]["ensures"] = json!(vec![json!({"bool":true}); 65]);
    reject_vir(module(vec![second]));
    let mut nested = json!({"bool":true});
    for _ in 0..32 {
        nested = json!({"op":"not","value":nested});
    }
    let mut value = base.clone();
    value["units"][0]["functions"][0]["contracts"]["ensures"] = json!([nested]);
    reject_vir(value);
    // An uninitialized local cannot be read on the short arm of a branch.
    let mut value = base.clone();
    let function = &mut value["units"][0]["functions"][0];
    function["locals"] = json!([{"id":"local0","type":ty("int")}]);
    function["blocks"][0]["instructions"][0]["lhs"] = var("local0");
    function["features_used"] = json!(["mutable_local"]);
    reject_vir(value);
}

const SOURCE: &[u8] = b"package vector;\npublic interface Case {\n public static int run(int x, int n) { return x / n; }\n}\n";
const SIDECAR: &[u8] = b"{\"schema\":\"mpk.java.contract.v0\",\"semantic_profile\":\"mpk.java.scalar.v0\",\"method\":\"vector.Case::run(int,int)->int\",\"requires\":[],\"ensures\":[{\"bool\":true}],\"modifies\":[],\"abrupt_completion\":\"forbidden\",\"termination\":\"total\"}\n";
fn captured(source: &[u8]) -> Vec<CapturedInput<'_>> {
    vec![
        CapturedInput {
            kind: InputKind::Contract,
            normalized_path: "contracts/run.json",
            bytes: SIDECAR,
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: "src/vector/Case.java",
            bytes: source,
        },
    ]
}
fn map_value(vir: &ValidatedSuccessorVir) -> Value {
    let mut entries = Vec::new();
    for unit in vir.module().units() {
        for function in unit.functions() {
            let origin = json!({"kind":"source","input_kind":"source","normalized_path":"src/vector/Case.java","start":39,"end":SOURCE.len()-3});
            entries.push(json!({"reference":{"kind":"function","unit_id":unit.id(),"function_id":function.id()},"origin":origin}));
            for block in function.blocks() {
                for instruction in &block.instructions {
                    let instruction = serde_json::to_value(instruction).unwrap();
                    entries.push(json!({"reference":{"kind":"instruction","unit_id":unit.id(),"function_id":function.id(),"block":block.label,"instruction":instruction["id"]},"origin":origin}));
                }
            }
            for block in function.blocks() {
                entries.push(json!({"reference":{"kind":"terminator","unit_id":unit.id(),"function_id":function.id(),"block":block.label},"origin":origin}));
            }
        }
    }
    entries.sort_by(|a, b| {
        let key = |e: &Value| {
            let r = &e["reference"];
            (
                r["function_id"].as_str().unwrap().to_owned(),
                match r["kind"].as_str().unwrap() {
                    "function" => 0,
                    "instruction" => 1,
                    _ => 2,
                },
                r["block"].as_str().unwrap_or("").to_owned(),
                r["instruction"]
                    .as_str()
                    .unwrap_or("t0")
                    .trim_start_matches('t')
                    .parse::<usize>()
                    .unwrap(),
            )
        };
        key(a).cmp(&key(b))
    });
    let mut value = json!({"schema":"mpk.source_map.v1","semantic_context":context(),"source_ir_schema":"mpk.vir.v1","source_ir_hash":vir.hash().as_str(),"entries":entries,"source_map_hash":ZERO});
    rehash_map(&mut value);
    value
}
fn rehash_map(value: &mut Value) {
    value["source_map_hash"] = json!(successor_source_map_hash_value(value).unwrap().as_str());
}
fn import_map(
    value: &Value,
    vir: &ValidatedSuccessorVir,
    captured: &[CapturedInput<'_>],
) -> Result<ValidatedSuccessorSourceMap, SuccessorArtifactError> {
    import_successor_source_map_json(
        &canonical(value),
        SuccessorSourceMapValidationContext {
            registry: &registry(),
            vir,
            captured_inputs: captured,
            synthetic_permissions: &[],
        },
    )
}
fn release() -> mpk_vc::ReleaseRegistryIdentity {
    // Explicit test context, not an installed Java release tuple.
    mpk_vc::ReleaseRegistryIdentity {
        schema: SUCCESSOR_RELEASE_REGISTRY_SCHEMA.to_owned(),
        id: SUCCESSOR_RELEASE_REGISTRY_ID.to_owned(),
        registry_sha256: "b".repeat(64),
    }
}
fn manifest_value(
    vir: &ValidatedSuccessorVir,
    map: &ValidatedSuccessorSourceMap,
    captured: &[CapturedInput<'_>],
) -> Value {
    let inputs=captured.iter().map(|i|json!({"kind":match i.kind {InputKind::Source=>"source",InputKind::Contract=>"contract",_=>"build_manifest"},"normalized_path":i.normalized_path,"size_bytes":i.bytes.len(),"sha256":sha256_raw_file_bytes(i.bytes).to_hex()})).collect::<Vec<_>>();
    let mut value = json!({"schema":"mpk.source_manifest.v1","semantic_context":context(),
        "selection":{"schema":"mpk.selection.java_methods.v0","value":{"compilation":"java-test","sources":["src/vector/Case.java"],"contracts":["contracts/run.json"],"methods":[vir.module().units()[0].functions()[0].id()]}},
        "limit_profile":"mpk.vir.limits.v0","release_registry":release(),
        "toolchain":{"bundle_id":"test.java.toolchain","distribution_sha256":load(PROFILE)["toolchain_inputs"]["archive"]["sha256"],"components":[]},
        "frontend":{"bundle_id":"test.java.frontend","name":"java2vir","version":"0.1.0","binary_sha256":"c".repeat(64),"subordinate_binaries":[]},
        "units":[{"identity":"java-test","name":"java-test","kind":"compilation"}],"target":{"id":"linux-x64","pointer_width":64},
        "inputs":inputs,"input_set_hash":ZERO,"vir_hash":vir.hash().as_str(),"source_map_hash":map.hash().as_str(),"source_manifest_hash":ZERO});
    rehash_manifest(&mut value);
    value
}
fn rehash_manifest(value: &mut Value) {
    let inputs =
        serde_json::from_value::<Vec<mpk_vc::InputEntry>>(value["inputs"].clone()).unwrap();
    value["input_set_hash"] = json!(mpk_vc::input_set_hash(&inputs).unwrap().as_str());
    value["source_manifest_hash"] = json!(successor_source_manifest_hash_value(value)
        .unwrap()
        .as_str());
}
fn import_manifest(
    value: &Value,
    vir: &ValidatedSuccessorVir,
    source_map: &ValidatedSuccessorSourceMap,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<ValidatedSuccessorSourceManifest, SuccessorArtifactError> {
    import_successor_source_manifest_json(
        &canonical(value),
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: &registry(),
            vir,
            source_map,
            captured_inputs,
            expected_release_registry: &release(),
        },
    )
}

#[test]
fn maps_bind_utf8_origins_helpers_and_captured_bytes_to_manifest_selection() {
    let vir = accept_vir(&shifted("bv_lshr", "int"));
    let captured = captured(SOURCE);
    let value = map_value(&vir);
    let map = import_map(&value, &vir, &captured).unwrap();
    let manifest = manifest_value(&vir, &map, &captured);
    import_manifest(&manifest, &vir, &map, &captured).unwrap();
    for (pointer, replacement) in [
        ("/entries/1/origin/start", json!(40)),
        ("/entries/1/origin/end", json!(SOURCE.len())),
        (
            "/entries/1/origin/normalized_path",
            json!("src/vector/Other.java"),
        ),
        (
            "/entries/1/origin",
            json!({"kind":"synthetic","reason":"java.shift.helper"}),
        ),
        ("/source_ir_hash", json!(ZERO)),
        ("/semantic_context/source_language", json!("csharp")),
    ] {
        let mut mutated = value.clone();
        *mutated.pointer_mut(pointer).unwrap() = replacement;
        rehash_map(&mut mutated);
        assert!(import_map(&mutated, &vir, &captured).is_err(), "{pointer}");
    }
    let mut missing = value.clone();
    missing["entries"].as_array_mut().unwrap().remove(1);
    rehash_map(&mut missing);
    assert!(import_map(&missing, &vir, &captured).is_err());
    for (pointer, replacement) in [
        ("/target/pointer_width", json!(32)),
        ("/target/id", json!("linux/amd64")),
        ("/units/0/kind", json!("package")),
        ("/units/0/identity", json!("other")),
        ("/selection/value/sources", json!(["src/vector/Other.java"])),
        (
            "/selection/value/contracts",
            json!(["contracts/other.json"]),
        ),
        (
            "/selection/value/methods",
            json!(["vector.Case::missing(int,int)->int"]),
        ),
        ("/inputs/0/sha256", json!(ZERO)),
        ("/inputs/1/sha256", json!(ZERO)),
        ("/vir_hash", json!(ZERO)),
        ("/source_map_hash", json!(ZERO)),
        ("/release_registry/registry_sha256", json!(ZERO)),
    ] {
        let mut mutated = manifest.clone();
        *mutated.pointer_mut(pointer).unwrap() = replacement;
        rehash_manifest(&mut mutated);
        assert!(
            import_manifest(&mutated, &vir, &map, &captured).is_err(),
            "{pointer}"
        );
    }
    let mut vc = manifest.clone();
    vc["vc_hash"] = json!(ZERO);
    rehash_manifest(&mut vc);
    assert!(import_manifest(&vc, &vir, &map, &captured).is_err());
    // Rehashing a same-sized new snapshot cannot reuse an old validated map.
    let changed = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("x / n", "x + n");
    let changed_inputs = vec![
        captured[0],
        CapturedInput {
            bytes: changed.as_bytes(),
            ..captured[1]
        },
    ];
    let mut mutated = manifest.clone();
    mutated["inputs"][1]["sha256"] = json!(sha256_raw_file_bytes(changed.as_bytes()).to_hex());
    rehash_manifest(&mut mutated);
    assert!(import_manifest(&mutated, &vir, &map, &changed_inputs).is_err());
}

#[test]
fn java_vc_regenerates_only_divisor_obligations_and_preserves_java_context() {
    let registry = registry();
    let profile_contract = profile_contract("vc");
    let captured = captured(SOURCE);
    for token in ["int", "long"] {
        for op in ["bv_sdiv", "bv_srem", "bv_add", "bv_shl", "bv_lshr"] {
            let value = if matches!(op, "bv_shl" | "bv_lshr") {
                shifted(op, token)
            } else {
                arithmetic(op, token)
            };
            let vir = accept_vir(&value);
            let map = import_map(&map_value(&vir), &vir, &captured).unwrap();
            let manifest = import_manifest(
                &manifest_value(&vir, &map, &captured),
                &vir,
                &map,
                &captured,
            )
            .unwrap();
            let source = SuccessorVcSource {
                registry: &registry,
                vir: &vir,
                manifest: &manifest,
                profile_contract: &profile_contract,
            };
            let vc = generate_successor_vc(source).unwrap();
            assert_eq!(
                vc.document().semantic_context(),
                vir.module().semantic_context()
            );
            assert_eq!(
                vc.required_checks().len(),
                usize::from(matches!(op, "bv_sdiv" | "bv_srem"))
            );
            for audit in vc.required_checks() {
                assert_eq!(
                    serde_json::to_value(audit.check()).unwrap(),
                    json!({"kind":"divisor_nonzero"})
                );
                assert_eq!(
                    audit.function_id(),
                    vir.module().units()[0].functions()[0].id()
                );
            }
            assert!(!String::from_utf8_lossy(vc.canonical_bytes())
                .contains("signed_divrem_representable"));
            import_successor_vc_json(vc.canonical_bytes(), source).unwrap();
            let original: Value = load(vc.canonical_bytes());
            for (pointer, replacement) in [
                ("/semantic_context/profile_registry/revision", json!(2)),
                (
                    "/semantic_context/profile_entry_sha256",
                    json!(CSHARP_SCALAR_ENTRY_SHA256),
                ),
                ("/source_ir_hash", json!(ZERO)),
                ("/source_manifest_hash", json!(ZERO)),
                ("/input_set_hash", json!(ZERO)),
                ("/functions", json!([])),
                (
                    "/profile_contract/value/required_check_profile_id",
                    json!("mpk.csharp.required_checks.v0"),
                ),
            ] {
                let mut value = original.clone();
                *value.pointer_mut(pointer).unwrap() = replacement;
                let mut payload = value.clone();
                payload.as_object_mut().unwrap().remove("vc_hash").unwrap();
                value["vc_hash"] = json!(hash_domain_separated_raw(
                    HashDomain::new("MPK-VC-2.0"),
                    &canonical(&payload)
                )
                .unwrap()
                .to_hex());
                assert!(
                    import_successor_vc_json(&canonical(&value), source).is_err(),
                    "{op}: {pointer}"
                );
            }
        }
    }
}

#[test]
fn strict_transport_source_bytes_and_origin_boundaries_fail_closed() {
    let registry = registry();
    let transport = canonical_registry_transport(&load(REGISTRY)["registry"]).unwrap();
    let text = String::from_utf8(transport.clone()).unwrap();
    for bytes in [
        format!(" {text}").into_bytes(),
        format!("{text}\n").into_bytes(),
        text.replace('\n', "\r\n").into_bytes(),
        text.replacen("\"revision\":3", "\"revision\":3,\"revision\":3", 1)
            .into_bytes(),
        text.replacen("\"revision\":3", "\"revision\":3e0", 1)
            .into_bytes(),
        text.replacen(
            "\"source_language\":\"java\"",
            "\"source_language\":\"java\",\"source_language\":\"java\"",
            1,
        )
        .into_bytes(),
        [b"\xef\xbb\xbf".as_slice(), transport.as_slice()].concat(),
    ] {
        assert!(validate_semantic_profile_registry(&bytes, RegistryRevision::Revision3).is_err());
    }
    let base = arithmetic("bv_sdiv", "int");
    let encoded = String::from_utf8(canonical(&base)).unwrap();
    for bytes in [
        encoded
            .replacen("\"width\":32", "\"width\":32,\"width\":32", 1)
            .into_bytes(),
        encoded
            .replacen("\"id\":\"java-test\"", "\"id\":\"\\ud800\"", 1)
            .into_bytes(),
        encoded
            .replacen("\"width\":32", "\"width\":32.0", 1)
            .into_bytes(),
        [b"\xef\xbb\xbf".as_slice(), encoded.as_bytes()].concat(),
    ] {
        assert!(import_successor_vir_json(&bytes, &registry).is_err());
    }
    let vir = accept_vir(&base);
    let map = map_value(&vir);
    for bytes in [
        Vec::new(),
        SOURCE[..SOURCE.len() - 1].to_vec(),
        [b"\xef\xbb\xbf".as_slice(), SOURCE].concat(),
        [SOURCE, b"\r\n"].concat(),
        [SOURCE, b"\0\n"].concat(),
        [SOURCE, b"\xff\n"].concat(),
        [SOURCE, b"// \\u0041\n"].concat(),
        [SOURCE, "// \u{fdd0}\n".as_bytes()].concat(),
        [SOURCE, "// \u{10ffff}\n".as_bytes()].concat(),
    ] {
        assert!(import_map(&map, &vir, &captured(&bytes)).is_err());
    }
    let unicode = [SOURCE, "// 日😀\t\n".as_bytes()].concat();
    import_map(&map, &vir, &captured(&unicode)).unwrap();
    for (start, end) in [(-1, 2), (40, 40), (50, 41), (40, 100000)] {
        let mut value = map.clone();
        value["entries"][1]["origin"]["start"] = json!(start);
        value["entries"][1]["origin"]["end"] = json!(end);
        rehash_map(&mut value);
        assert!(import_map(&value, &vir, &captured(SOURCE)).is_err());
    }
    // A UTF-8 interior byte is never accepted, even inside the method range.
    let unicode = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("return x / n;", "/*😀*/ return x / n;");
    let offset = unicode.find('😀').unwrap();
    let mut value = map.clone();
    value["entries"][0]["origin"]["end"] = json!(unicode.len() - 3);
    value["entries"][1]["origin"]["start"] = json!(offset + 1);
    value["entries"][1]["origin"]["end"] = json!(offset + 4);
    rehash_map(&mut value);
    assert!(import_map(&value, &vir, &captured(unicode.as_bytes())).is_err());
    let mut inputs = captured(SOURCE);
    inputs.push(inputs[1]);
    assert!(import_map(&map, &vir, &inputs).is_err());
    let mut inputs = captured(SOURCE);
    inputs.push(CapturedInput {
        kind: InputKind::BuildManifest,
        normalized_path: "pom.xml",
        bytes: b"",
    });
    assert!(import_map(&map, &vir, &inputs).is_err());
    let mut value = map.clone();
    let reference =
        serde_json::from_value::<mpk_vc::SourceReference>(value["entries"][1]["reference"].clone())
            .unwrap();
    value["entries"][1]["origin"] = json!({"kind":"synthetic","reason":"java.shift.helper"});
    rehash_map(&mut value);
    assert!(import_successor_source_map_json(
        &canonical(&value),
        SuccessorSourceMapValidationContext {
            registry: &registry,
            vir: &vir,
            captured_inputs: &captured(SOURCE),
            synthetic_permissions: &[mpk_vc::SyntheticPermission {
                reference,
                reason: "java.shift.helper".to_owned()
            }]
        }
    )
    .is_err());
}

#[test]
fn static_calls_are_typed_acyclic_and_entirely_owned_by_the_selection() {
    let helper = function("helper", &["int"], "int", vec![], var("arg0"));
    let call = json!({"kind":"CallStatic","id":"t0","type":ty("int"),"function":helper["id"],"contract_hash":ZERO,"args":[var("arg0")],"safety_checks":[]});
    let run = function("run", &["int"], "int", vec![call], var("t0"));
    let base = module(vec![helper.clone(), run.clone()]);
    accept_vir(&base);
    for (pointer, replacement) in [
        (
            "/units/0/functions/1/blocks/0/instructions/0/function",
            run["id"].clone(),
        ),
        (
            "/units/0/functions/1/blocks/0/instructions/0/function",
            json!("java.lang.Math::abs(int)->int"),
        ),
        (
            "/units/0/functions/1/blocks/0/instructions/0/args",
            json!([]),
        ),
        (
            "/units/0/functions/1/blocks/0/instructions/0/args",
            json!([{"bool":true}]),
        ),
    ] {
        let mut value = base.clone();
        *value.pointer_mut(pointer).unwrap() = replacement;
        reject_vir(value);
    }
    let mut reversed = base.clone();
    reversed["units"][0]["functions"]
        .as_array_mut()
        .unwrap()
        .reverse();
    reject_vir(reversed);
    let mut bad_hash = base.clone();
    bad_hash["units"][0]["functions"][1]["blocks"][0]["instructions"][0]["contract_hash"] =
        json!(ZERO);
    bad_hash["vir_hash"] = json!(successor_vir_hash_value(&bad_hash).unwrap().as_str());
    assert!(import_successor_vir_json(&canonical(&bad_hash), &registry()).is_err());
    let vir = accept_vir(&base);
    let mut captured = captured(SOURCE);
    captured.push(CapturedInput {
        kind: InputKind::Contract,
        normalized_path: "contracts/helper.json",
        bytes: SIDECAR,
    });
    captured.sort_by_key(|i| i.normalized_path);
    let map = import_map(&map_value(&vir), &vir, &captured).unwrap();
    let mut value = manifest_value(&vir, &map, &captured);
    value["selection"]["value"]["contracts"] =
        json!(["contracts/helper.json", "contracts/run.json"]);
    value["selection"]["value"]["methods"] = json!([run["id"]]);
    rehash_manifest(&mut value);
    import_manifest(&value, &vir, &map, &captured).unwrap();
    value["selection"]["value"]["methods"] = json!([helper["id"]]);
    rehash_manifest(&mut value);
    assert!(
        import_manifest(&value, &vir, &map, &captured).is_err(),
        "unselected unrelated method must reject"
    );
}

#[test]
fn shift_helpers_cannot_be_reordered_interleaved_or_reused() {
    let reordered = module(vec![function(
        "run",
        &["int", "int"],
        "int",
        vec![
            convert("t0", "u32", var("arg0")),
            constant("t1", "int", "31"),
            bin("t2", "bv_and", "int", var("arg1"), var("t1")),
            bin("t3", "bv_lshr", "u32", var("t0"), var("t2")),
            convert("t4", "int", var("t3")),
        ],
        var("t4"),
    )]);
    reject_vir(reordered);
    let interleaved = module(vec![function(
        "run",
        &["int", "int"],
        "int",
        vec![
            constant("t0", "int", "31"),
            bin("t1", "bv_and", "int", var("arg1"), var("t0")),
            constant("t2", "int", "0"),
            bin("t3", "bv_shl", "int", var("arg0"), var("t1")),
        ],
        var("t3"),
    )]);
    reject_vir(interleaved);
    let mut reused = shifted("bv_shl", "int");
    reused["units"][0]["functions"][0]["blocks"][0]["instructions"]
        .as_array_mut()
        .unwrap()
        .push(bin("t3", "bv_shl", "int", var("t2"), var("t1")));
    reused["units"][0]["functions"][0]["blocks"][0]["terminator"]["values"] = json!([var("t3")]);
    reject_vir(reused);
}

#[test]
fn acyclic_branch_joins_preserve_types_and_branch_local_checks() {
    let mut f = function(
        "run",
        &["boolean", "int", "int"],
        "int",
        vec![],
        var("arg1"),
    );
    f["features_used"] = json!(["branch"]);
    f["blocks"] = json!([
        {"label":"bb0","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":var("arg0"),"else_label":"bb1","else_args":[],"then_label":"bb2","then_args":[]}},
        {"label":"bb1","parameters":[],"instructions":[constant("t0","int","0")],"terminator":{"kind":"Jump","label":"bb3","args":[var("t0")]}},
        {"label":"bb2","parameters":[],"instructions":[bin("t1","bv_sdiv","int",var("arg1"),var("arg2"))],"terminator":{"kind":"Jump","label":"bb3","args":[var("t1")]}},
        {"label":"bb3","parameters":[{"id":"p0","type":ty("int")}],"instructions":[],"terminator":{"kind":"Return","values":[var("p0")]}}
    ]);
    let base = module(vec![f]);
    accept_vir(&base);
    for (pointer, replacement) in [
        ("/0/terminator/cond", var("arg1")),
        ("/1/terminator/args", json!([])),
        ("/1/terminator/args", json!([var("t1")])),
        ("/2/instructions/0/safety_checks", json!([])),
        ("/3/parameters/0/type", ty("long")),
        ("/3/terminator/values", json!([var("t1")])),
    ] {
        let mut value = base.clone();
        *value["units"][0]["functions"][0]["blocks"]
            .pointer_mut(pointer)
            .unwrap() = replacement;
        reject_vir(value);
    }
    let vir = accept_vir(&base);
    let captured = captured(SOURCE);
    let map = import_map(&map_value(&vir), &vir, &captured).unwrap();
    let manifest = import_manifest(
        &manifest_value(&vir, &map, &captured),
        &vir,
        &map,
        &captured,
    )
    .unwrap();
    let registry = registry();
    let contract = profile_contract("vc");
    let vc = generate_successor_vc(SuccessorVcSource {
        registry: &registry,
        vir: &vir,
        manifest: &manifest,
        profile_contract: &contract,
    })
    .unwrap();
    assert_eq!(vc.required_checks().len(), 1);
    let safety = vc.document().functions()[0]
        .members
        .iter()
        .find(|member| member.id == vc.required_checks()[0].member_id())
        .unwrap();
    assert_eq!(
        safety.assumptions,
        vec![VcTerm::Var {
            name: "arg0".to_owned()
        }]
    );
}
