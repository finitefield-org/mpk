//! CSHARP-03-T06-W08: pure-transition and complete-snapshot replay VCs.
//! CSHARP-03-T06-W07: canonical boundary and captured round-trip VCs.
//! CSHARP-03-T06-W06: binding and concrete foundation equivalence VCs.
//! CSHARP-03-T06-W05: exceptional outcomes, search/unwind and postcondition VCs.
//! CSHARP-03-T06-W04: loop cutpoints, decreases and ordered pattern VCs.
//! CSHARP-03-T06-W01: verification expression union and original-input attachment.
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
#[allow(dead_code)]
#[path = "../../mpk-cli/tests/support/csharp_practical_data_context.rs"]
mod support;
fn ty(s: &str) -> String {
    format!("mpk.csharp.value.{s}.v1")
}
fn b() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn lit(t: &str, v: Value) -> Value {
    json!({"tag":"literal","type_id":ty(t),"value":v})
}
fn var(id: &str, t: &str) -> Value {
    json!({"tag":"variable","type_id":t,"binding_id":id})
}
fn expr(tag: &str, t: &str, fields: Value) -> Value {
    let mut v = json!({"tag":tag,"type_id":t});
    v.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    v
}
fn primitive(t: &str) -> Value {
    json!({"kind":"primitive","id":t})
}
fn instance(t: &str, args: Vec<Value>) -> Value {
    json!({"kind":"instance","template":t,"arguments":args})
}
fn source(name: &str, kind: &str) -> Value {
    let identity = json!({"kind":"type","namespace":"Example","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""});
    let id = csharp_practical_declaration_id(&identity).unwrap();
    let m = csharp_practical_stored_member_id(&id, "Number", &primitive("i32"), "readonly_field")
        .unwrap();
    json!({"id":id,"identity":identity,"kind":kind,"members":[{"id":m,"name":"Number","type":primitive("i32"),"storage":"readonly_field","ordinal":0,"required":false}],"enum_values":[],"enum_underlying":null,"actual_default":{m:0},"public_default":true,"identity_sensitive":false,"source_sha256":"0".repeat(64)})
}
fn canonical(v: &Value) -> Vec<u8> {
    fn j(v: &Value) -> PracticalJsonValue {
        static PACKAGE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        let package = PACKAGE.get_or_init(|| {
            serde_json::from_str(include_str!(
                "../../../develop/specs/vectors/csharp-practical-profile-v1.json"
            ))
            .unwrap()
        });
        if let Some(tag) = v.get("tag").and_then(Value::as_str) {
            let variant = package["frozen_contract"]["expression_union"]["variants"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["tag"] == tag)
                .unwrap();
            PracticalJsonValue::Object(
                variant["ordered_fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|k| {
                        let k = k.as_str().unwrap();
                        let child = &v[k];
                        let value = match variant["field_types"][k].as_str().unwrap() {
                            "contract_expression" | "contract_expression_bool" => j(child),
                            "contract_expression_or_null" if !child.is_null() => j(child),
                            "ordered_array<contract_expression>" => PracticalJsonValue::Array(
                                child.as_array().unwrap().iter().map(j).collect(),
                            ),
                            "codec_parameters" => PracticalJsonValue::Object(
                                ["scale", "rounding"]
                                    .iter()
                                    .map(|k| {
                                        (
                                            k.to_string(),
                                            serde_json::from_value(child[k].clone()).unwrap(),
                                        )
                                    })
                                    .collect(),
                            ),
                            _ => serde_json::from_value(child.clone()).unwrap(),
                        };
                        (k.into(), value)
                    })
                    .collect(),
            )
        } else {
            serde_json::from_value(v.clone()).unwrap()
        }
    }
    canonical_practical_json_bytes(&j(v)).unwrap()
}
struct Fixture {
    b: ValidatedFoundationBundle,
    r: ValidatedClosedRootSet,
    c: ClosedInstanceSet,
    env: DataContractEnvironment,
    cases: Vec<Value>,
}
fn fixture() -> Fixture {
    let b = b();
    let s = source("Value", "readonly_struct");
    let e = source("Fault", "sealed_class");
    let sid = s["id"].as_str().unwrap();
    let eid = e["id"].as_str().unwrap();
    let mut sources = serde_json::Map::new();
    sources.insert(sid.into(), s.clone());
    sources.insert(eid.into(), e.clone());
    let types = vec![
        instance("bounded_sequence", vec![primitive("i32")]),
        instance("ordered_map", vec![primitive("i32"), primitive("i32")]),
        instance("ordered_set", vec![primitive("i32")]),
        instance("lookup", vec![primitive("i32")]),
        instance("option", vec![primitive("i32")]),
        instance("result", vec![primitive("i32"), primitive("parse_error")]),
        instance(
            "transition",
            vec![primitive("i32"), primitive("i32"), primitive("i32")],
        ),
        json!({"kind":"source","id":sid}),
        json!({"kind":"source","id":eid}),
    ];
    let roots=types.iter().enumerate().map(|(n,t)|json!({"origin":"semantic_binding","provenance_id":format!("verification.{n}"),"type":t})).collect::<Vec<_>>();
    let r = validate_closed_root_set(
        &b,
        &canonical_closed_root_set_transport(&b, &json!(roots), &Value::Object(sources)).unwrap(),
    )
    .unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    let ids = types[..7]
        .iter()
        .map(|t| csharp_practical_closed_instance_id(&b, t).unwrap())
        .collect::<Vec<_>>();
    let mut env = DataContractEnvironment {
        allow_old: true,
        result: Some(ty("i32")),
        verification_owner: "fixture.owner".into(),
        ..Default::default()
    };
    env.variables = BTreeMap::from([
        ("x".into(), ty("i32")),
        ("s".into(), sid.into()),
        ("seq".into(), ids[0].clone()),
        ("map".into(), ids[1].clone()),
        ("set".into(), ids[2].clone()),
        ("sum".into(), ids[4].clone()),
        ("transition".into(), ids[6].clone()),
        ("exception".into(), ty("exception")),
    ]);
    let member = s["members"][0]["id"].as_str().unwrap();
    let emember = e["members"][0]["id"].as_str().unwrap();
    env.properties
        .insert(member.into(), (sid.into(), ty("i32")));
    env.constructors.insert(
        "ctor".into(),
        ClosedOperationSignature {
            id: "ctor".into(),
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![ty("i32")],
            normal_result_type_id: sid.into(),
            ordered_checks: vec![],
        },
    );
    env.bindings
        .insert("binding.fixture".into(), (sid.into(), ty("i32")));
    env.exception_universe = Some(
        derive_closed_exception_universe(
            &r,
            &c,
            &[SourceExceptionDefinition {
                type_id: eid.into(),
                sealed: true,
                direct_base_type_id: "System.Exception".into(),
                payload_member_ids: vec![emember.into()],
            }],
        )
        .unwrap(),
    );
    env.exception_type = Some(eid.into());
    let x = var("x", &ty("i32"));
    let truth = lit("bool", json!(true));
    let int = lit("i32", json!("1"));
    let seq = var("seq", &ids[0]);
    let sum = var("sum", &ids[4]);
    let transition = var("transition", &ids[6]);
    let exception = var("exception", &ty("exception"));
    let cases = vec![
        truth.clone(),
        x.clone(),
        expr("result", &ty("i32"), json!({})),
        expr("old", &ty("i32"), json!({"expression":x})),
        expr(
            "field",
            &ty("i32"),
            json!({"receiver":var("s",sid),"member_id":member}),
        ),
        expr(
            "property",
            &ty("i32"),
            json!({"receiver":var("s",sid),"member_id":member}),
        ),
        expr(
            "unary",
            &ty("bool"),
            json!({"operation_id":"boolean.not","operand":truth}),
        ),
        expr(
            "binary",
            &ty("bool"),
            json!({"operation_id":"integer.i32.equal.checked","left":x,"right":int}),
        ),
        expr(
            "conditional",
            &ty("i32"),
            json!({"condition":truth,"when_true":x,"when_false":int}),
        ),
        expr(
            "let",
            &ty("i32"),
            json!({"binding_id":"local","value":int,"body":var("local",&ty("i32"))}),
        ),
        expr(
            "construct",
            sid,
            json!({"constructor_id":"ctor","arguments":[int]}),
        ),
        expr("sequence_length", &ty("i32"), json!({"sequence":seq})),
        expr(
            "sequence_index",
            &ty("i32"),
            json!({"sequence":seq,"index":int}),
        ),
        expr(
            "map_contains",
            &ty("bool"),
            json!({"map":var("map",&ids[1]),"key":int}),
        ),
        expr(
            "map_lookup",
            &ids[3],
            json!({"map":var("map",&ids[1]),"key":int}),
        ),
        expr(
            "set_contains",
            &ty("bool"),
            json!({"set":var("set",&ids[2]),"element":int}),
        ),
        expr(
            "tagged_make",
            &ids[4],
            json!({"semantic_instance_id":ids[4],"arm":"some","payload":int}),
        ),
        expr("tagged_is", &ty("bool"), json!({"value":sum,"arm":"some"})),
        expr(
            "tagged_payload",
            &ty("i32"),
            json!({"value":sum,"arm":"some"}),
        ),
        expr(
            "source_project",
            &ty("i32"),
            json!({"binding_id":"binding.fixture","source_value":var("s",sid)}),
        ),
        expr(
            "source_reconstruct",
            sid,
            json!({"binding_id":"binding.fixture","semantic_value":int}),
        ),
        expr(
            "structural_equal",
            &ty("bool"),
            json!({"left":x,"right":int}),
        ),
        expr(
            "structural_compare",
            &ty("i32"),
            json!({"left":x,"right":int}),
        ),
        expr(
            "codec_parse",
            &ids[5],
            json!({"codec_id":"integer.i32","codec_parameters":{"scale":null,"rounding":null},"text":lit("string",json!("42"))}),
        ),
        expr(
            "codec_format",
            &ty("string"),
            json!({"codec_id":"integer.i32","codec_parameters":{"scale":null,"rounding":null},"value":int,"mode":"canonical"}),
        ),
        expr(
            "parse_error_kind",
            &ty("u32"),
            json!({"value":lit("parse_error",json!("syntax"))}),
        ),
        expr(
            "exception_is",
            &ty("bool"),
            json!({"value":exception,"exception_type_id":eid}),
        ),
        expr(
            "exception_payload",
            &ty("i32"),
            json!({"value":exception,"member_id":emember}),
        ),
        expr("transition_state", &ty("i32"), json!({"value":transition})),
        expr("transition_events", &ids[0], json!({"value":transition})),
        expr(
            "transition_response",
            &ty("i32"),
            json!({"value":transition}),
        ),
        expr(
            "bounded_forall",
            &ty("bool"),
            json!({"binding_id":"q","lower":int,"upper":x,"body":truth}),
        ),
        expr(
            "bounded_exists",
            &ty("bool"),
            json!({"binding_id":"q","lower":int,"upper":x,"body":truth}),
        ),
    ];
    Fixture {
        b,
        r,
        c,
        env,
        cases,
    }
}
#[test]
fn csharp_03_t06_w01_complete_union_and_typing() {
    let f = fixture();
    let frozen: Value = serde_json::from_str(include_str!(
        "../../../develop/specs/vectors/csharp-practical-profile-v1.json"
    ))
    .unwrap();
    let expected = frozen["frozen_contract"]["expression_union"]["variants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["tag"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected,
        f.cases.iter().map(|v| v["tag"].as_str().unwrap()).collect()
    );
    assert_eq!(expected.len(), 33);
    let mut receipts = vec![];
    for v in &f.cases {
        let bytes = canonical(v);
        let tag = v["tag"].as_str().unwrap();
        let front = parse_data_contract_expression(&f.b, &f.r, &f.c, &f.env, &bytes)
            .unwrap_or_else(|e| panic!("frontend {tag}: {e:?}"));
        let imported = import_verification_contract_expression(&f.b, &f.r, &f.c, &f.env, &bytes)
            .unwrap_or_else(|e| panic!("import {tag}: {e:?}"));
        assert_eq!(front.type_id(), imported.term().type_id());
        assert_eq!(
            imported,
            import_verification_contract_encoding(
                &f.b,
                &f.r,
                &f.c,
                &f.env,
                &bytes,
                &imported.canonical_bytes()
            )
            .unwrap()
        );
        let mut bad = v.clone();
        bad["type_id"] = json!(ty("unit"));
        assert!(
            import_verification_contract_expression(&f.b, &f.r, &f.c, &f.env, &canonical(&bad))
                .is_err(),
            "{tag}"
        );
        let mut bad = imported.canonical_bytes();
        bad[3] ^= 0x80;
        assert!(
            import_verification_contract_encoding(&f.b, &f.r, &f.c, &f.env, &bytes, &bad).is_err()
        );
        receipts.push(serde_json::to_value(imported).unwrap());
    }
    if let Ok(path) = std::env::var("MPK_T06_W01_ENCODINGS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
    }
}
#[test]
fn csharp_03_t06_w01_scopes_normalization_and_parser_fuzz() {
    let f = fixture();
    let import =
        |bytes: &[u8]| import_verification_contract_expression(&f.b, &f.r, &f.c, &f.env, bytes);
    let base = canonical(&lit("bool", json!(true)));
    for n in 0..128 {
        let mut bytes = base.clone();
        let k = n * 7 % bytes.len();
        bytes[k] = 0xff;
        assert!(import(&bytes).is_err());
    }
    for s in [
        String::from_utf8(base.clone())
            .unwrap()
            .replace("\"value\":true", "\"value\":true,\"value\":false"),
        String::from_utf8(base.clone())
            .unwrap()
            .replace("literal", "call"),
        String::from_utf8(base.clone())
            .unwrap()
            .replace("true}", "true,\"extra\":0}"),
        String::from_utf8(base.clone()).unwrap() + " ",
    ] {
        assert!(import(s.as_bytes()).is_err());
    }
    for v in [
        var("missing", &ty("i32")),
        expr(
            "old",
            &ty("i32"),
            json!({"expression":expr("result",&ty("i32"),json!({}))}),
        ),
        expr(
            "let",
            &ty("i32"),
            json!({"binding_id":"x","value":lit("i32",json!("1")),"body":var("x",&ty("i32"))}),
        ),
        expr(
            "binary",
            &ty("i32"),
            json!({"operation_id":"source.impure","left":lit("i32",json!("1")),"right":lit("i32",json!("2"))}),
        ),
    ] {
        assert!(import(&canonical(&v)).is_err());
    }
    let renamed = |id: &str| {
        expr(
            "let",
            &ty("i32"),
            json!({"binding_id":id,"value":lit("i32",json!("1")),"body":var(id,&ty("i32"))}),
        )
    };
    let a = import(&canonical(&renamed("a"))).unwrap();
    let b = import(&canonical(&renamed("b"))).unwrap();
    assert_eq!(a.term(), b.term());
    assert_ne!(a.expression_sha256(), b.expression_sha256());
    let mut partial = f.env.clone();
    partial.partial_callables.insert("ctor".into());
    let constructor = canonical(f.cases.iter().find(|c| c["tag"] == "construct").unwrap());
    assert!(
        import_verification_contract_expression(&f.b, &f.r, &f.c, &partial, &constructor).is_err()
    );
    let mut env = f.env.clone();
    env.verification_owner = "different owner".into();
    assert_ne!(
        import(&base).unwrap().attachment_sha256(),
        import_verification_contract_expression(&f.b, &f.r, &f.c, &env, &base)
            .unwrap()
            .attachment_sha256()
    );
    let mut deep = lit("bool", json!(true));
    for n in 0..65 {
        deep = expr(
            "let",
            &ty("bool"),
            json!({"binding_id":format!("d{n}"),"value":lit("bool",json!(true)),"body":deep}),
        );
    }
    assert!(import(&canonical(&deep)).is_err());
    let surrogate =
        br#"{"tag":"literal","type_id":"mpk.csharp.value.string.v1","value":"\ud800x\udfff"}"#;
    let encoded = import(surrogate).unwrap();
    assert!(encoded.definitions()[0].parameters.contains("\\ud800"));
}
fn read(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn csharp_03_t06_w01_original_attachment_and_vc_handoff() {
    let b = b();
    let mut receipts = vec![];
    for (requests, responses, case) in [
        (
            "verification-expressions/type-requests.json",
            "verification-expressions/type-responses.json",
            Some("type"),
        ),
        (
            "boundary-transition/data-requests.json",
            "boundary-transition/data-responses.json",
            Some("total"),
        ),
        (
            "boundary-transition/control-requests.json",
            "boundary-transition/control-responses.json",
            Some("valid"),
        ),
        (
            "data-phase/data-sidecar-requests.json",
            "data-phase/data-sidecar-responses.json",
            None,
        ),
    ] {
        let req = read(requests);
        let res = read(responses);
        let row = if case == Some("type") {
            &req[0]
        } else if let Some(case) = case {
            req.as_array()
                .unwrap()
                .iter()
                .find(|r| r["case"] == case)
                .unwrap()
        } else {
            &req[20]
        };
        let result = res
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == row["id"])
            .unwrap();
        let (ctx, captures) = support::replay_context(&b, row);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &ctx,
            &captures,
            &serde_json::to_vec(&result["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &ctx, &captures, &source).unwrap();
        let expressions = emitted.vir().contract_expressions();
        if case == Some("total") {
            assert!(expressions.is_empty());
        } else {
            assert!(!expressions.is_empty());
        }
        let source = PracticalVcSource {
            artifact_context: &ctx,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(source).unwrap();
        assert_eq!(vc.contract_expressions(), expressions);
        for (schema, owner) in [
            (BOUNDARY_CONTRACT_SCHEMA, LaterProofOwner::BoundaryRoundTrip),
            (TRANSITION_CONTRACT_SCHEMA, LaterProofOwner::PureTransition),
        ] {
            if emitted
                .vir()
                .data_contracts()
                .iter()
                .any(|c| c.contains(schema))
            {
                assert!(vc
                    .obligation_groups()
                    .iter()
                    .any(|g| g.proof_owner() == owner
                        && g.subject_ids()
                            .iter()
                            .any(|s| s.starts_with(&format!("data_contract:{schema}:")))));
            }
        }

        assert!(
            vc.resource_reservation().ordinary_term_nodes_minimum()
                >= expressions.iter().map(|e| e.term().nodes() as u64).sum()
        );
        let second = import_csharp_practical_vc_json(vc.canonical_bytes(), source).unwrap();
        assert_eq!(vc.canonical_bytes(), second.canonical_bytes());
        assert_eq!(second.contract_expressions(), expressions);
        let original = String::from_utf8(vc.canonical_bytes().to_vec()).unwrap();
        let bad = original.replace(
            &format!(
                "\"ordinary_term_nodes_minimum\":{}",
                vc.resource_reservation().ordinary_term_nodes_minimum()
            ),
            "\"ordinary_term_nodes_minimum\":0",
        );
        assert_eq!(
            import_csharp_practical_vc_json(bad.as_bytes(), source)
                .err()
                .unwrap()
                .phase(),
            PracticalVcValidationPhase::Limits
        );
        receipts.push(json!({"requests":requests,"input_sha256":captures.snapshot_sha256(),"vir_sha256":emitted.vir().hash(),"vc_sha256":vc.hash(),"expressions":expressions,"reservation":vc.resource_reservation()}));
    }
    if let Ok(path) = std::env::var("MPK_T06_W01_ATTACHMENTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
    }
}

#[test]
fn csharp_03_t06_w01_specialization_and_inclusive_limits() {
    let mut f = fixture();
    let definition =
        f.c.entries()
            .iter()
            .flat_map(|e| e["operation_definitions"].as_array().unwrap())
            .find(|o| {
                o["argument_type_ids"].as_array().unwrap().len() == 1
                    && o["error_precedence"].as_array().unwrap().is_empty()
            })
            .unwrap();
    let argument = definition["argument_type_ids"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let result = definition["normal_result_type_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let id = definition["id"].as_str().unwrap().to_owned();
    f.env
        .variables
        .insert("specialized".into(), argument.clone());
    f.env.operations.insert(
        id.clone(),
        ClosedOperationSignature {
            id: id.clone(),
            tag: ClosedOperationTag::Foundation,
            argument_type_ids: vec![argument.clone()],
            normal_result_type_id: result.clone(),
            ordered_checks: vec![],
        },
    );
    let expression = canonical(&expr(
        "unary",
        &result,
        json!({"operation_id":id,"operand":var("specialized",&argument)}),
    ));
    assert!(import_verification_contract_expression(&f.b, &f.r, &f.c, &f.env, &expression).is_ok());
    f.env.operations.get_mut(&id).unwrap().tag = ClosedOperationTag::SourceCall;
    assert!(
        import_verification_contract_expression(&f.b, &f.r, &f.c, &f.env, &expression).is_err()
    );
    let literal = canonical(&lit("bool", json!(true)));
    for count in [255, 256, 257] {
        let env = DataContractEnvironment {
            variables: (0..count).map(|n| (format!("v{n}"), ty("bool"))).collect(),
            ..Default::default()
        };
        assert_eq!(
            import_verification_contract_expression(&f.b, &f.r, &f.c, &env, &literal).is_ok(),
            count <= 256
        );
    }
    let env = DataContractEnvironment::default();
    let mut depth = lit("bool", json!(true));
    for n in 1..=33 {
        if n >= 31 {
            assert_eq!(
                import_verification_contract_expression(&f.b, &f.r, &f.c, &env, &canonical(&depth))
                    .is_ok(),
                n <= 32
            );
        }
        depth = expr(
            "unary",
            &ty("bool"),
            json!({"operation_id":"boolean.not","operand":depth}),
        );
    }
    let mut nodes = lit("bool", json!(true));
    for _ in 0..9 {
        nodes = expr(
            "binary",
            &ty("bool"),
            json!({"operation_id":"boolean.and","left":nodes,"right":nodes}),
        );
    }
    for count in 1023..=1025 {
        assert_eq!(
            import_verification_contract_expression(&f.b, &f.r, &f.c, &env, &canonical(&nodes))
                .is_ok(),
            count <= 1024
        );
        nodes = expr(
            "unary",
            &ty("bool"),
            json!({"operation_id":"boolean.not","operand":nodes}),
        );
    }
    let mut quantifier = lit("bool", json!(true));
    for depth in 1..=5 {
        quantifier = expr(
            "bounded_forall",
            &ty("bool"),
            json!({"binding_id":format!("q{depth}"),"lower":lit("i32",json!("0")),"upper":lit("i32",json!("1")),"body":quantifier}),
        );
        assert_eq!(
            import_verification_contract_expression(
                &f.b,
                &f.r,
                &f.c,
                &env,
                &canonical(&quantifier)
            )
            .is_ok(),
            depth <= 4
        );
    }
}

#[path = "support/csharp_practical_construction_vc.rs"]
mod construction;

#[path = "support/csharp_practical_data_vc.rs"]
mod data;

#[path = "support/csharp_practical_control_vc.rs"]
mod control;

#[path = "support/csharp_practical_exception_vc.rs"]
mod exception;

#[path = "support/csharp_practical_binding_vc.rs"]
mod binding;

#[path = "support/csharp_practical_boundary_vc.rs"]
mod boundary;

#[path = "support/csharp_practical_transition_vc.rs"]
mod transition;
