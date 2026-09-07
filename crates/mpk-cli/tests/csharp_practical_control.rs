//! CSHARP-03-T04-W01: actual Roslyn source handoffs and retained sidecar mutations.
//! CSHARP-03-T04-W02: structured CFGs, ownership and runtime differentials.
//! CSHARP-03-T04-W03: switch and pattern decision graphs.
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use mpk_vc::csharp_practical_vir_model::*;
use mpk_vc::hash_domain_separated_raw;
use serde_json::{json, Value};
use std::{fs, path::Path};
#[allow(dead_code)]
#[path = "support/csharp_practical_data_context.rs"]
mod context_support;
fn read(path: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap()
}
fn cases() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/loop-contracts/source-cases.json",
    ))
    .unwrap()
}
fn case(id: &str) -> Value {
    cases().into_iter().find(|v| v["id"] == id).unwrap()
}
fn method(case: &Value) -> &Value {
    case["facts"]["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["callable_id"] == case["root"])
        .unwrap()
}
fn ty(token: &str) -> String {
    format!("mpk.csharp.value.{token}.v1")
}
fn literal(token: &str, value: J) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", value),
    ])
}
fn boolean() -> J {
    literal("bool", J::Bool(true))
}
fn integer() -> J {
    literal("i32", J::string("0"))
}
fn variable(token: &str, id: &str) -> J {
    J::object(vec![
        ("tag", J::string("variable")),
        ("type_id", J::string(ty(token))),
        ("binding_id", J::string(id)),
    ])
}
fn rows(case: &Value) -> Vec<J> {
    method(case)["loops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            J::object(vec![
                ("loop_id", J::string(l["loop_id"].as_str().unwrap())),
                ("invariants", J::Array(vec![boolean()])),
                (
                    "modifies",
                    J::Array(
                        l["modifies"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| J::string(v.as_str().unwrap()))
                            .collect(),
                    ),
                ),
                ("decreases", J::Array(vec![integer()])),
            ])
        })
        .collect()
}
fn set(value: &mut J, key: &str, replacement: J) {
    let J::Object(fields) = value else { panic!() };
    fields.iter_mut().find(|(k, _)| k == key).unwrap().1 = replacement;
}
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn roots(b: &ValidatedFoundationBundle) -> (ValidatedClosedRootSet, ClosedInstanceSet) {
    let root = json!([{"origin":"semantic_binding","provenance_id":"loop.array","type":{"kind":"instance","template":"bounded_sequence","arguments":[{"kind":"primitive","id":"i32"}]}}]);
    let bytes = canonical_closed_root_set_transport(b, &root, &json!({})).unwrap();
    let r = validate_closed_root_set(b, &bytes).unwrap();
    let c = derive_closed_instances(b, &r).unwrap();
    (r, c)
}
fn document(
    context: &a::PracticalArtifactContext,
    case: &Value,
    termination: &str,
    rows: Vec<J>,
) -> J {
    J::object(vec![
        ("schema", J::string(a::METHOD_CONTRACT_SCHEMA)),
        ("semantic_context", context.semantic_context().clone()),
        ("compilation_id", J::string(context.compilation_id())),
        ("callable_id", J::string(case["root"].as_str().unwrap())),
        (
            "source_content_sha256",
            J::string(case["facts"]["sources"][0]["raw_sha256"].as_str().unwrap()),
        ),
        ("termination", J::string(termination)),
        ("requires", J::Array(vec![])),
        ("ensures", J::Array(vec![boolean()])),
        ("exceptional_cases", J::Array(vec![])),
        ("modifies", J::Array(vec![])),
        ("loops", J::Array(rows)),
    ])
}
fn hashed(mut value: J) -> Vec<u8> {
    let hash = hash_domain_separated_raw(
        a::METHOD_CONTRACT_HASH_DOMAIN,
        &a::canonical_practical_json_bytes(&value).unwrap(),
    )
    .unwrap()
    .to_hex();
    let J::Object(fields) = &mut value else {
        panic!()
    };
    fields.push(("contract_sha256".into(), J::string(hash)));
    a::canonical_practical_json_bytes(&value).unwrap()
}
fn run_mutation(
    case: &Value,
    termination: &str,
    rows: Vec<J>,
    mutation: impl FnOnce(&mut J),
) -> Result<PreparedLoopContracts, LoopContractError> {
    let b = bundle();
    let (r, c) = roots(&b);
    let (context, captures) = context_support::context_with_sidecar(
        &b,
        case["root"].as_str().unwrap(),
        case["source"].as_str().unwrap().as_bytes(),
        |ctx| {
            let mut doc = document(ctx, case, termination, rows);
            mutation(&mut doc);
            hashed(doc)
        },
    );
    prepare_loop_contracts(
        &b,
        &r,
        &c,
        &context,
        &captures,
        &serde_json::to_vec(&case["facts"]).unwrap(),
        &DataContractEnvironment::default(),
    )
}
fn run(
    case: &Value,
    termination: &str,
    rows: Vec<J>,
) -> Result<PreparedLoopContracts, LoopContractError> {
    run_mutation(case, termination, rows, |_| {})
}
#[test]
fn csharp_03_t04_w01_actual_source_forms_and_pending_claims() {
    for source in cases().into_iter().filter(|v| {
        v["accepted"] == true
            && !v["id"].as_str().unwrap().starts_with("set_")
            && !v["id"].as_str().unwrap().starts_with("map_")
    }) {
        let result = run(&source, "total", rows(&source))
            .unwrap_or_else(|e| panic!("{}: {e:?}", source["id"]));
        assert_eq!(result.artifact_count(), 0);
        assert_eq!(
            result.loops().len(),
            source["facts"]["methods"][0]["loops"]
                .as_array()
                .unwrap()
                .len()
        );
        assert!(result
            .loops()
            .iter()
            .all(|l| l.obligations.contains(&"ownership_frame".into())));
        if source["id"] == "array_fill" || source["id"] == "count_fill" {
            assert!(result
                .loops()
                .iter()
                .any(|l| l.obligations.contains(&"initialized_prefix".into())
                    && l.obligations.contains(&"exact_output_count".into())));
        }
        assert_eq!(
            result.loops(),
            run(&source, "total", rows(&source)).unwrap().loops()
        );
    }
    let source = case("nested");
    let result = run(&source, "total", rows(&source)).unwrap();
    assert!(result.loops()[0].abrupt_exits.is_empty());
    assert_eq!(
        result.loops()[1]
            .abrupt_exits
            .iter()
            .map(|e| e.kind.as_str())
            .collect::<Vec<_>>(),
        ["continue", "break"]
    );
    assert_eq!(
        result.loops()[1].parent.as_ref(),
        Some(&result.loops()[0].loop_id)
    );
    let source = case("switch_break");
    assert_eq!(
        run(&source, "total", rows(&source)).unwrap().loops()[0]
            .abrupt_exits
            .len(),
        1
    );
    let source = case("return");
    let result = run(&source, "total", rows(&source)).unwrap();
    assert_eq!(result.loops()[0].return_claims, vec![boolean()]);
    assert_eq!(result.loops()[0].abrupt_exits[0].kind, "return");
}
#[test]
fn csharp_03_t04_w01_frozen_limits_and_approved_partial_amendment() {
    let source = case("while");
    for count in [63, 64, 65] {
        let mut rows = rows(&source);
        set(
            &mut rows[0],
            "invariants",
            J::Array(vec![boolean(); count - 1]),
        );
        let result = run(&source, "total", rows);
        if count <= 64 {
            assert!(result.is_ok())
        } else {
            assert_eq!(
                result.unwrap_err(),
                LoopContractError::Limit("invariant_decreases_per_loop")
            );
        }
    }
    for termination in ["partial", "total"] {
        for count in [0, 1] {
            let mut rows = rows(&source);
            set(&mut rows[0], "decreases", J::Array(vec![integer(); count]));
            let result = run(&source, termination, rows);
            if termination == "total" && count == 0 {
                assert_eq!(result.unwrap_err(), LoopContractError::MissingDecreases)
            } else {
                assert_eq!(result.unwrap().loops()[0].termination, termination);
            }
        }
    }
    for (id, counter) in [
        ("loops_33", "loops_per_method"),
        ("nesting_9", "loop_nesting"),
    ] {
        let source = case(id);
        assert_eq!(source["accepted"], false);
        assert_eq!(source["diagnostic"], "CSHARP_PRACTICAL_LIMIT");
        assert_eq!(source["code"], counter);
    }
    let p: Value = serde_json::from_slice(&read(
        "develop/specs/vectors/csharp-practical-profile-v1.json",
    ))
    .unwrap();
    let owned = p["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T04-W01")
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 13);
    assert_eq!(LoopContractError::Expression.diagnostic_phase(), 7);
    assert_eq!(
        LoopContractError::Expression.diagnostic_family(),
        "CSHARP_PRACTICAL_LOOP_CONTRACT"
    );
    assert_eq!(
        LoopContractError::Limit("loops_per_method").diagnostic_phase(),
        0
    );
    assert!(owned.iter().all(|v| v["production_test_owner"]
        == "crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W01"));
    assert_eq!(
        p["frozen_contract"]["amendments"][1]["id"],
        "partial_loop_decreases"
    );
}
#[test]
fn csharp_03_t04_w01_strict_attachment_modifies_and_sidecar_hashes() {
    let source = case("nested");
    for mutation in 0..9 {
        let mut rows = rows(&source);
        match mutation {
            0 => {
                rows.pop();
            }
            1 => rows.push(rows[0].clone()),
            2 => rows.swap(0, 1),
            3 => set(&mut rows[0], "loop_id", J::string("wrong")),
            4 => set(&mut rows[0], "modifies", J::Array(vec![])),
            5 => set(
                &mut rows[0],
                "modifies",
                J::Array(vec![J::string("local:0"), J::string("local:0")]),
            ),
            6 => set(&mut rows[0], "invariants", J::Array(vec![])),
            7 => {
                let J::Object(f) = &mut rows[0] else { panic!() };
                f.retain(|(k, _)| k != "decreases");
            }
            _ => {
                let J::Object(f) = &mut rows[0] else { panic!() };
                f.push(("extra".into(), J::Null));
            }
        }
        assert!(run(&source, "total", rows).is_err(), "mutation {mutation}");
    }
    for field in ["source_content_sha256", "callable_id", "compilation_id"] {
        assert!(run_mutation(&source, "total", rows(&source), |doc| set(
            doc,
            field,
            J::string("wrong")
        ))
        .is_err());
    }
    let b = bundle();
    let (r, c) = roots(&b);
    let (context, captures) = context_support::context(
        &b,
        source["root"].as_str().unwrap(),
        source["source"].as_str().unwrap().as_bytes(),
    );
    assert_eq!(
        prepare_loop_contracts(
            &b,
            &r,
            &c,
            &context,
            &captures,
            &serde_json::to_vec(&source["facts"]).unwrap(),
            &DataContractEnvironment::default()
        )
        .unwrap_err(),
        LoopContractError::Attachment
    );
}
#[test]
fn csharp_03_t04_w01_expression_types_purity_scopes_and_precedence() {
    let source = case("scope");
    let expressions = vec![
        integer(),
        variable("bool", "local:0"),
        variable("i32", "local:1"),
        variable("i32", "local:2"),
        J::object(vec![
            ("tag", J::string("result")),
            ("type_id", J::string(ty("bool"))),
        ]),
        J::object(vec![
            ("tag", J::string("call")),
            ("type_id", J::string(ty("bool"))),
        ]),
        J::object(vec![
            ("tag", J::string("old")),
            ("type_id", J::string(ty("bool"))),
            ("expression", boolean()),
        ]),
    ];
    for bad in expressions {
        let mut rows = rows(&source);
        set(&mut rows[0], "invariants", J::Array(vec![bad]));
        assert_eq!(
            run(&source, "total", rows).unwrap_err(),
            LoopContractError::Expression
        );
    }
    let comparison = J::object(vec![
        ("tag", J::string("binary")),
        ("type_id", J::string(ty("bool"))),
        (
            "operation_id",
            J::string("integer.i32.greater_equal.checked"),
        ),
        ("left", variable("i32", "local:0")),
        ("right", integer()),
    ]);
    let mut valid = rows(&source);
    set(&mut valid[0], "invariants", J::Array(vec![comparison]));
    set(
        &mut valid[0],
        "decreases",
        J::Array(vec![variable("i32", "parameter:0")]),
    );
    assert!(run(&source, "total", valid).is_ok());
    let mut rows = rows(&source);
    set(&mut rows[0], "invariants", J::Array(vec![integer(); 65]));
    assert_eq!(
        run(&source, "total", rows).unwrap_err(),
        LoopContractError::Limit("invariant_decreases_per_loop")
    );
}
#[test]
fn csharp_03_t04_w01_nested_and_source_provenance_mutations() {
    for mutation in 0..8 {
        let mut source = case("nested");
        match mutation {
            0 => source["facts"]["sources"][0]["raw_sha256"] = json!("0".repeat(64)),
            1 => source["facts"]["methods"][0]["loops"][1]["parent"] = Value::Null,
            2 => source["facts"]["methods"][0]["loops"][1]["start_byte"] = json!(0),
            3 => source["facts"]["methods"][0]["loops"][1]["end_byte"] = json!(999999),
            4 => source["facts"]["methods"][0]["loops"][1]["loop_id"] = json!("wrong"),
            5 => source["facts"]["methods"][0]["loops"][1]["exits"][0]["target"] = json!("wrong"),
            6 => source["facts"]["compilation_id"] = json!("wrong"),
            _ => source["facts"]["methods"][0]["loops"][1]["extra"] = Value::Null,
        }
        assert!(
            run(&source, "total", rows(&source)).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn csharp_03_t04_w01_raw_sidecar_duplicate_hash_and_unknown_field_rejections() {
    let source = case("while");
    let b = bundle();
    let (r, c) = roots(&b);
    for mutation in 0..3 {
        let (context, captures) = context_support::context_with_sidecar(
            &b,
            source["root"].as_str().unwrap(),
            source["source"].as_str().unwrap().as_bytes(),
            |ctx| {
                let raw = String::from_utf8(hashed(document(ctx, &source, "total", rows(&source))))
                    .unwrap();
                match mutation {
                    0 => raw.replacen("\"decreases\":[", "\"decreases\":[],\"decreases\":[", 1),
                    1 => raw.replacen("\"contract_sha256\":\"", "\"contract_sha256\":\"0", 1),
                    _ => raw.replacen(
                        "\"tag\":\"literal\"",
                        "\"unknown\":true,\"tag\":\"literal\"",
                        1,
                    ),
                }
                .into_bytes()
            },
        );
        assert!(prepare_loop_contracts(
            &b,
            &r,
            &c,
            &context,
            &captures,
            &serde_json::to_vec(&source["facts"]).unwrap(),
            &DataContractEnvironment::default()
        )
        .is_err());
    }
}

#[test]
fn csharp_03_t04_w01_input_inventory_and_pinned_frontend_when_available() {
    use sha2::{Digest, Sha256};
    let manifest: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/loop-contracts/loop-contract-inputs.json",
    ))
    .unwrap();
    assert_eq!(manifest["work_item"], "CSHARP-03-T04-W01");
    for row in manifest["files"].as_array().unwrap() {
        let bytes = read(row["path"].as_str().unwrap());
        assert_eq!(row["size_bytes"].as_u64().unwrap(), bytes.len() as u64);
        assert_eq!(row["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    if !cfg!(target_os = "linux") {
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cache=root.join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    if !cache.is_dir() {
        return;
    }
    let output =
        std::process::Command::new(root.join("scripts/build-csharp-practical-frontend.sh"))
            .arg("--test-loop-contracts")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        read("develop/migrations/csharp-03/loop-contracts/source-cases.json")
    );
}

#[test]
fn csharp_03_t04_w01_collection_clauses_use_source_bound_t03_projections() {
    for name in ["set_add", "set_count", "map_add", "map_replace"] {
        check_collection_projection_source(name, case(name));
    }
}
fn check_collection_projection_source(name: &str, source: Value) {
    use std::collections::BTreeMap;
    let b = bundle();
    let map = name.starts_with("map");
    let source_hash = source["facts"]["sources"][0]["raw_sha256"]
        .as_str()
        .unwrap();
    let primitive = |id: &str| json!({"kind":"primitive","id":id});
    let seq = |t: Value| json!({"kind":"instance","template":"bounded_sequence","arguments":[t]});
    let source_type = |name: &str, kind: &str, members: Vec<(&str, Value)>| {
        let identity = json!({"kind":"type","namespace":"Business","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""});
        let id = csharp_practical_declaration_id(&identity).unwrap();
        let mut defaults = serde_json::Map::new();
        let members=members.into_iter().enumerate().map(|(ordinal,(name,ty))|{
                let mid=csharp_practical_stored_member_id(&id,name,&ty,"readonly_field").unwrap();
                defaults.insert(mid.clone(),if ty["kind"]=="primitive" {json!(0)} else {Value::Null});
                json!({"id":mid,"name":name,"type":ty,"storage":"readonly_field","ordinal":ordinal,"required":false})
            }).collect::<Vec<_>>();
        json!({"id":id,"identity":identity,"kind":kind,"members":members,"enum_values":[],"enum_underlying":null,"actual_default":defaults,"public_default":kind=="readonly_struct","identity_sensitive":false,"source_sha256":source_hash})
    };
    let pair = source_type(
        "Pair",
        "readonly_struct",
        vec![("Key", primitive("i32")), ("Value", primitive("i32"))],
    );
    let elements = seq(if map {
        json!({"kind":"source","id":pair["id"]})
    } else {
        primitive("i32")
    });
    let wrapper = source_type(
        if map { "Map" } else { "Set" },
        "sealed_class",
        vec![("Items", elements)],
    );
    let mut sources = serde_json::Map::new();
    sources.insert(wrapper["id"].as_str().unwrap().into(), wrapper.clone());
    if map {
        sources.insert(pair["id"].as_str().unwrap().into(), pair.clone());
    }
    let roots_json = json!([{"origin":"semantic_binding","provenance_id":"loop.collection","type":{"kind":"source","id":wrapper["id"]}}]);
    let root_bytes = canonical_closed_root_set_transport(&b, &roots_json, &json!(sources)).unwrap();
    let r = validate_closed_root_set(&b, &root_bytes).unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    let operation = if name.ends_with("replace") {
        "replace"
    } else if name.ends_with("count") {
        "count"
    } else {
        "add"
    };
    let binding =
        |source: &Value,
         role: &str,
         mappings: Vec<(&str, usize)>,
         args: Vec<String>,
         ops: Vec<a::SemanticOperationMapping>| a::SemanticBindingInput {
            source_type_id: source["id"].as_str().unwrap().into(),
            source_content_sha256: source_hash.into(),
            role: role.into(),
            member_map: mappings
                .into_iter()
                .map(|(role, index)| a::SemanticBindingMember {
                    role: role.into(),
                    member_id: source["members"][index]["id"].as_str().unwrap().into(),
                })
                .collect(),
            tag_arms: vec![],
            inferred_argument_ids: args,
            default_arm: "ineligible".into(),
            bounds: if role == "ordered_entry" {
                vec![]
            } else {
                vec![a::SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }]
            },
            operation_map: ops,
            enum_arms: BTreeMap::new(),
        };
    let mut bindings = vec![binding(
        &wrapper,
        if map { "ordered_map" } else { "ordered_set" },
        vec![(if map { "entries" } else { "elements" }, 0)],
        if map {
            vec![ty("i32"), ty("i32")]
        } else {
            vec![ty("i32")]
        },
        vec![a::SemanticOperationMapping {
            operation: operation.into(),
            member_id: source["root"].as_str().unwrap().into(),
        }],
    )];
    if map {
        bindings.push(binding(
            &pair,
            "ordered_entry",
            vec![("key", 0), ("value", 1)],
            vec![ty("i32"), ty("i32")],
            vec![],
        ));
    }
    let empty_ctx = context_support::context(
        &b,
        source["root"].as_str().unwrap(),
        source["source"].as_str().unwrap().as_bytes(),
    );
    let semantic =
        a::build_semantic_bindings(&empty_ctx.0, &empty_ctx.1, bindings.clone()).unwrap();
    let semantic_row = semantic
        .value()
        .get("bindings")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row.get("source_type_id").unwrap().as_str() == wrapper["id"].as_str())
        .unwrap();
    let semantic_type = csharp_practical_closed_instance_id(&b, &json!({"kind":"instance","template":if map{"ordered_map"}else{"ordered_set"},"arguments":if map{vec![primitive("i32"),primitive("i32")]}else{vec![primitive("i32")]}})).unwrap();
    let projection = J::object(vec![
        ("tag", J::string("source_project")),
        ("type_id", J::string(semantic_type)),
        (
            "binding_id",
            J::string(format!(
                "binding.{}",
                semantic_row
                    .get("binding_sha256")
                    .unwrap()
                    .as_str()
                    .unwrap()
            )),
        ),
        (
            "source_value",
            J::object(vec![
                ("tag", J::string("variable")),
                ("type_id", J::string(wrapper["id"].as_str().unwrap())),
                ("binding_id", J::string("parameter:0")),
            ]),
        ),
    ]);
    let contains = J::object(vec![
        (
            "tag",
            J::string(if map { "map_contains" } else { "set_contains" }),
        ),
        ("type_id", J::string(ty("bool"))),
        (if map { "map" } else { "set" }, projection),
        (if map { "key" } else { "element" }, integer()),
    ]);
    let mut loop_rows = rows(&source);
    set(&mut loop_rows[0], "invariants", J::Array(vec![contains]));
    let doc = hashed(document(&empty_ctx.0, &source, "total", loop_rows));
    // The context's semantic identity is independent of the sidecar paths;
    // the final capture hashes and byte membership are rebound below.
    let request = json!({"compilation_id":"data","roots":[source["root"]],"inputs":[
        {"kind":"source","path":"src/Entry.cs","utf8":source["source"]},
        {"kind":"sidecar","path":"contracts/a.json","utf8":String::from_utf8(doc).unwrap()},
        {"kind":"sidecar","path":"contracts/b.json","utf8":std::str::from_utf8(semantic.canonical_bytes()).unwrap()}
    ]});
    let (context, captures) = context_support::replay_context(&b, &request);
    let result = prepare_loop_contracts(
        &b,
        &r,
        &c,
        &context,
        &captures,
        &serde_json::to_vec(&source["facts"]).unwrap(),
        &DataContractEnvironment::default(),
    )
    .unwrap_or_else(|e| panic!("{name}:{e:?}"));
    if source["lowering"].is_object() {
        let lowered = prepare_loop_lowering(
            &b,
            &r,
            &c,
            &context,
            &captures,
            &serde_json::to_vec(&source["facts"]).unwrap(),
            &serde_json::to_vec(&source["lowering"]).unwrap(),
            &DataContractEnvironment::default(),
        )
        .unwrap();
        assert_eq!(lowered.contracts().loops(), result.loops());
        assert_eq!(lowered.functions().len(), 1);
        assert_eq!(
            lowered.functions()[0].loops.len(),
            method(&source)["loops"].as_array().unwrap().len()
        );
    }
    if operation != "count" {
        let mut wrong = bindings.clone();
        wrong[0].operation_map[0].operation = "count".into();
        let artifact = a::build_semantic_bindings(&empty_ctx.0, &empty_ctx.1, wrong).unwrap();
        let mut changed = request.clone();
        changed["inputs"][2]["utf8"] =
            json!(std::str::from_utf8(artifact.canonical_bytes()).unwrap());
        let (ctx, cap) = context_support::replay_context(&b, &changed);
        assert_eq!(
            prepare_loop_contracts(
                &b,
                &r,
                &c,
                &ctx,
                &cap,
                &serde_json::to_vec(&source["facts"]).unwrap(),
                &DataContractEnvironment::default()
            )
            .unwrap_err(),
            LoopContractError::Attachment
        );
    }
    let loop_ = &result.loops()[0];
    let clause = loop_
        .collection_clauses
        .iter()
        .find(|clause| clause.subject_id == wrapper["id"])
        .unwrap();
    assert_eq!(clause.operation, operation);
    assert_eq!(
        clause.member_ids,
        vec![wrapper["members"][0]["id"].as_str().unwrap()]
    );
    assert_eq!(clause.supporting_invariants, loop_.invariants);
    assert!(clause.predicates.contains(&"canonical_order".into()));
    assert!(clause.predicates.contains(&"uniqueness".into()));
    if operation == "add" {
        assert!(clause
            .predicates
            .contains(&"duplicate_policy_reject".into()));
        assert!(clause
            .predicates
            .contains(&"insertion_order_independence".into()));
    }
    if operation == "replace" {
        assert!(clause
            .predicates
            .contains(&"duplicate_policy_replace".into()));
        assert!(!clause
            .predicates
            .contains(&"insertion_order_independence".into()));
    }
}

#[test]
fn csharp_03_t04_w01_each_form_attachment_typing_and_bounded_facts() {
    for form in ["for", "while", "do", "foreach_array", "foreach_string"] {
        let source = case(form);
        assert_eq!(
            run(&source, "total", vec![]).unwrap_err(),
            LoopContractError::Attachment
        );
        let mut invalid = rows(&source);
        set(&mut invalid[0], "invariants", J::Array(vec![integer()]));
        assert_eq!(
            run(&source, "total", invalid).unwrap_err(),
            LoopContractError::Expression
        );
        let mut invalid = rows(&source);
        set(&mut invalid[0], "invariants", J::Array(vec![integer(); 65]));
        assert_eq!(
            run(&source, "total", invalid).unwrap_err(),
            LoopContractError::Limit("invariant_decreases_per_loop")
        );
    }
    let source = case("while");
    let predicate = J::object(vec![
        ("tag", J::string("binary")),
        ("type_id", J::string(ty("bool"))),
        (
            "operation_id",
            J::string("integer.i32.greater_equal.checked"),
        ),
        ("left", variable("i32", "index")),
        ("right", integer()),
    ]);
    let quantifier = J::object(vec![
        ("tag", J::string("bounded_forall")),
        ("type_id", J::string(ty("bool"))),
        ("binding_id", J::string("index")),
        ("lower", integer()),
        ("upper", variable("i32", "local:0")),
        ("body", predicate),
    ]);
    let mut valid = rows(&source);
    set(
        &mut valid[0],
        "invariants",
        J::Array(vec![quantifier.clone()]),
    );
    assert!(run(&source, "total", valid).is_ok());
    let mut shadow = quantifier;
    set(&mut shadow, "binding_id", J::string("local:0"));
    let mut invalid = rows(&source);
    set(&mut invalid[0], "invariants", J::Array(vec![shadow]));
    assert_eq!(
        run(&source, "total", invalid).unwrap_err(),
        LoopContractError::Expression
    );
    let source = case("shadow");
    let mut invalid = rows(&source);
    set(
        &mut invalid[1],
        "decreases",
        J::Array(vec![variable("i32", "local:0")]),
    );
    assert_eq!(
        run(&source, "total", invalid).unwrap_err(),
        LoopContractError::Expression
    );
    let source = case("count_fill");
    let result = run(&source, "total", rows(&source)).unwrap();
    let count = &result.loops()[0].collection_clauses[0];
    let fill = &result.loops()[1].collection_clauses[0];
    assert_eq!(count.operation, "count");
    assert_eq!(fill.operation, "fill");
    assert_eq!(count.allocation, fill.allocation);
    assert_eq!(
        count.allocation.as_ref().unwrap().length_binding.as_deref(),
        Some("local:0")
    );
    assert!(count
        .predicates
        .contains(&"count_allocation_agreement".into()));
    let mut invalid = source.clone();
    invalid["facts"]["methods"][0]["allocations"][0]["length_binding"] = json!("local:999");
    assert!(run(&invalid, "total", rows(&invalid)).is_err());
}

#[path = "support/csharp_practical_loop_lowering.rs"]
mod lowering;

#[path = "support/csharp_practical_pattern_lowering.rs"]
mod patterns;

#[path = "support/csharp_practical_exception_lowering.rs"]
mod exceptions;

#[path = "support/csharp_practical_handler_lowering.rs"]
mod handlers;
