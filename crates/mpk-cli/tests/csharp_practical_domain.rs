//! CSHARP-03-T03-W12: nullable relations and lossless application outcome bindings.
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(p: &str) -> Vec<u8> {
    fs::read(root().join(p)).unwrap()
}
fn file(p: &str) -> Value {
    serde_json::from_slice(&read(p)).unwrap()
}
fn ty(s: &str) -> String {
    format!("mpk.csharp.value.{s}.v1")
}

#[test]
fn csharp_03_t03_w14_reference_dereference_keeps_its_exception_owner() {
    let b = bundle();
    let (r, c, ids) = fixture(
        &b,
        &[
            instance("option", vec![primitive("string")]),
            instance("option", vec![primitive("i32")]),
        ],
        json!({}),
    );
    let reference = OutcomeModel::new(&b, &r, &c, &ids[0]).unwrap();
    let none = reference.construct("none", None).unwrap();
    assert_eq!(
        reference.reference_value(&none),
        Err(DomainError::NullReceiver)
    );
    assert_eq!(
        DomainError::NullReceiver.exception_type(),
        Some("System.NullReferenceException")
    );
    let text = MonomorphicValue::String {
        type_id: ty("string"),
        utf16: vec![0xd800],
    };
    let some = reference.construct("some", Some(text.clone())).unwrap();
    assert_eq!(reference.reference_value(&some).unwrap(), text);
    let nullable = OutcomeModel::new(&b, &r, &c, &ids[1]).unwrap();
    assert_eq!(
        nullable.reference_value(&nullable.construct("none", None).unwrap()),
        Err(DomainError::Signature)
    );
}

#[test]
fn csharp_03_t03_w14_explicit_codec_configuration_and_frozen_schema_mutations() {
    use mpk_vc::csharp_practical_source_artifacts::PracticalJsonValue as J;
    let decode = |codec: &str, bytes: &[u8]| {
        BoundaryCodec::from_contract_parameters_json(codec, &ty("decimal"), bytes)
    };
    let valid = br#"{"scale":2,"rounding":"ToEven"}"#;
    // Internal typed evidence; T05 owns the first boundary-document invocation.
    let no_parameters = J::object(vec![("scale", J::Null), ("rounding", J::Null)]);
    assert!(
        BoundaryCodec::from_optional_contract_parameters(None, &ty("decimal"), &J::Null)
            .unwrap()
            .is_none()
    );
    assert!(
        BoundaryCodec::from_optional_contract_parameters(None, &ty("decimal"), &no_parameters)
            .is_err()
    );
    assert!(BoundaryCodec::from_optional_contract_parameters(
        Some("decimal.normalized"),
        &ty("decimal"),
        &J::Null
    )
    .is_err());
    assert!(BoundaryCodec::from_optional_contract_parameters(
        Some("decimal.normalized"),
        &ty("decimal"),
        &no_parameters
    )
    .unwrap()
    .is_some());
    assert!(decode("decimal.fixed", valid).is_ok());
    for invalid in [
        br#"{}"#.as_slice(),
        br#"null"#,
        br#"[]"#,
        br#"{"scale":2}"#,
        br#"{"rounding":"ToEven"}"#,
        br#"{"scale":2,"rounding":"ToEven","extra":0}"#,
        br#"{"scale":2,"scale":2,"rounding":"ToEven"}"#,
        br#"{"rounding":"ToEven","scale":2}"#,
        br#"{"scale":-1,"rounding":"ToEven"}"#,
        br#"{"scale":29,"rounding":"ToEven"}"#,
        br#"{"scale":2.0,"rounding":"ToEven"}"#,
        br#"{"scale":2e0,"rounding":"ToEven"}"#,
        br#"{"scale":true,"rounding":"ToEven"}"#,
        br#"{"scale":"2","rounding":"ToEven"}"#,
        br#"{"scale":null,"rounding":"ToEven"}"#,
        br#"{"scale":2,"rounding":null}"#,
        br#"{"scale":2,"rounding":"unknown"}"#,
        br#"{"scale":2,"rounding":0}"#,
        br#"{"scale":2,"rounding":"ToEven"} "#,
    ] {
        assert!(decode("decimal.fixed", invalid).is_err(), "{invalid:?}");
    }
    assert!(decode("decimal.fixed.scale.2", valid).is_err());
    assert!(decode("decimal.normalized", valid).is_err());
    assert!(decode("decimal.normalized", br#"{"scale":null,"rounding":null}"#).is_ok());
    let b = bundle();
    let (r, c, _) = fixture(&b, &[], json!({}));
    let value = MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        negative: false,
        scale: 2,
        coefficient: "125".into(),
    };
    for scale in [0, 1, 2, 28] {
        for rounding in [
            "ToEven",
            "AwayFromZero",
            "ToZero",
            "ToNegativeInfinity",
            "ToPositiveInfinity",
        ] {
            let parameters = J::object(vec![
                ("scale", J::U64(scale)),
                ("rounding", J::string(rounding)),
            ]);
            let codec = BoundaryCodec::from_contract_parameters(
                "decimal.fixed",
                &ty("decimal"),
                &parameters,
            )
            .unwrap();
            assert!(codec.validate_contract_format_mode("canonical").is_ok());
            assert!(codec.validate_contract_format_mode(rounding).is_err());
            let text = codec.format(&b, &r, &c, &value).unwrap();
            assert!(codec.parse(&text).is_ok());
            if scale == 1 {
                let expected = if ["AwayFromZero", "ToPositiveInfinity"].contains(&rounding) {
                    "1.3"
                } else {
                    "1.2"
                };
                assert_eq!(String::from_utf16(&text).unwrap(), expected);
            }
        }
    }
    let text: Vec<u16> = "1.20".encode_utf16().collect();
    assert!(
        decode("decimal.fixed", br#"{"scale":1,"rounding":"ToEven"}"#)
            .unwrap()
            .parse(&text)
            .is_err()
    );
    assert!(decode("decimal.fixed", valid).unwrap().parse(&text).is_ok());
    let package = file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let mut count = 0;
    for row in package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["inputs"]["record"] == "codec_parameters")
    {
        assert_eq!(row["implementation_owner"], "CSHARP-03-T03-W14");
        count += 1;
        match row["id"].as_str().unwrap().rsplit('.').next().unwrap() {
            "valid" => assert!(decode("decimal.fixed", valid).is_ok()),
            "unknown_field" => {
                assert!(
                    decode("decimal.fixed", br#"{"scale":2,"rounding":"ToEven","x":0}"#).is_err()
                )
            }
            "missing_field" => {
                assert!(decode("decimal.fixed", br#"{"scale":2}"#).is_err());
                assert!(decode("decimal.fixed", br#"{"rounding":"ToEven"}"#).is_err());
            }
            "wrong_field_type" => {
                assert!(decode("decimal.fixed", br#"{"scale":[],"rounding":"ToEven"}"#).is_err());
                assert!(decode("decimal.fixed", br#"{"scale":2,"rounding":[]}"#).is_err());
            }
            "duplicate_key" => assert!(decode(
                "decimal.fixed",
                row["inputs"]["raw_utf8"].as_str().unwrap().as_bytes()
            )
            .is_err()),
            _ => panic!("unexecuted configuration vector {row}"),
        }
    }
    assert_eq!(count, 5);
}

fn primitive(s: &str) -> Value {
    json!({"kind":"primitive","id":s})
}
fn instance(role: &str, args: Vec<Value>) -> Value {
    json!({"kind":"instance","template":role,"arguments":args})
}
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn fixture(
    b: &ValidatedFoundationBundle,
    types: &[Value],
    sources: Value,
) -> (ValidatedClosedRootSet, ClosedInstanceSet, Vec<String>) {
    let roots:Vec<_>=types.iter().enumerate().map(|(i,t)|json!({"origin":"semantic_binding","provenance_id":format!("domain.{i}"),"type":t})).collect();
    let bytes = canonical_closed_root_set_transport(b, &json!(roots), &sources).unwrap();
    let r = validate_closed_root_set(b, &bytes).unwrap();
    let c = derive_closed_instances(b, &r).unwrap();
    let ids = types
        .iter()
        .map(|t| {
            if t["kind"] == "source" {
                t["id"].as_str().unwrap().to_owned()
            } else if t["kind"] == "primitive" {
                ty(t["id"].as_str().unwrap())
            } else {
                csharp_practical_closed_instance_id(b, t).unwrap()
            }
        })
        .collect();
    (r, c, ids)
}
fn raw(token: &str, s: &str) -> MonomorphicValue {
    match token {
        "bool" => MonomorphicValue::Bool {
            type_id: ty(token),
            value: s == "1",
        },
        "f32" => MonomorphicValue::F32Bits {
            type_id: ty(token),
            bits: match s {
                "-1" => "bf800000",
                "0" => "00000000",
                "1" => "3f800000",
                "nan" => "ffc00000",
                _ => panic!("float input"),
            }
            .into(),
        },
        "f64" => MonomorphicValue::F64Bits {
            type_id: ty(token),
            bits: match s {
                "-1" => "bff0000000000000",
                "0" => "0000000000000000",
                "1" => "3ff0000000000000",
                "nan" => "fff8000000000000",
                _ => panic!("float input"),
            }
            .into(),
        },
        "decimal" => MonomorphicValue::DecimalBits {
            type_id: ty(token),
            negative: s.starts_with('-'),
            scale: 0,
            coefficient: s.trim_start_matches('-').into(),
        },
        _ => MonomorphicValue::Signed {
            type_id: ty(token),
            value: s.into(),
        },
    }
}
fn encoded(v: &MonomorphicValue) -> String {
    match v {
        MonomorphicValue::Option { value: None, .. } => "none".into(),
        MonomorphicValue::Option { value: Some(v), .. } => {
            if let MonomorphicValue::Bool { value, .. } = **v {
                if value { "1" } else { "0" }.into()
            } else {
                encoded(v)
            }
        }
        MonomorphicValue::Bool { value, .. } => value.to_string(),
        MonomorphicValue::Signed { value, .. } => value.clone(),
        MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
            bits.clone()
        }
        MonomorphicValue::DecimalBits {
            negative,
            coefficient,
            scale,
            ..
        } => {
            let mut n = coefficient.parse::<u128>().unwrap();
            let mut scale = *scale;
            while scale > 0 && n % 10 == 0 {
                n /= 10;
                scale -= 1;
            }
            assert_eq!(scale, 0);
            format!("{}{}", if *negative && n != 0 { "-" } else { "" }, n)
        }
        _ => panic!("unexpected value {v:?}"),
    }
}
fn nullable(m: &OutcomeModel<'_>, token: &str, s: &str) -> MonomorphicValue {
    m.construct(
        if s == "none" { "none" } else { "some" },
        if s == "none" {
            None
        } else {
            Some(raw(token, s))
        },
    )
    .unwrap()
}
#[test]
fn csharp_03_t03_w12_all_628_frozen_nullable_relations() {
    let b = bundle();
    let tokens = ["i32", "i64", "f32", "f64", "decimal", "bool"];
    let types: Vec<_> = tokens
        .iter()
        .map(|t| instance("option", vec![primitive(t)]))
        .collect();
    let (r, c, ids) = fixture(&b, &types, json!({}));
    let vectors = file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let mut count = 0;
    for v in vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W12")
    {
        count += 1;
        let op = v["inputs"]["operation"].as_str().unwrap();
        let inputs: Vec<_> = v["inputs"]["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        let token = if op == "nullable.boolean" {
            "bool"
        } else if op.starts_with("lifted.") {
            op.split('.').nth(1).unwrap()
        } else {
            "i32"
        };
        let model = OutcomeModel::new(
            &b,
            &r,
            &c,
            &ids[tokens.iter().position(|t| *t == token).unwrap()],
        )
        .unwrap();
        let values: Vec<_> = inputs.iter().map(|s| nullable(&model, token, s)).collect();
        let result = (|| -> Result<Vec<String>, DomainError> {
            let name = op.rsplit('.').next().unwrap();
            Ok(match name {
                "inspect" => vec![
                    (model.arm(&values[0])? == "some").to_string(),
                    encoded(&model.value_or_default(&values[0])?),
                    encoded(&model.value_or(&values[0], raw("i32", "7"))?),
                ],
                "value" => vec![encoded(model.read(&values[0], "some")?)],
                "boolean" => vec![
                    encoded(&model.lift("and", &values, true)?),
                    encoded(&model.lift("or", &values, true)?),
                    encoded(&model.lift("not", &values[..1], true)?),
                    encoded(&model.lift("equal", &values, true)?),
                    encoded(&model.lift("not_equal", &values, true)?),
                ],
                "compare" => [
                    "equal",
                    "not_equal",
                    "less",
                    "less_equal",
                    "greater",
                    "greater_equal",
                ]
                .iter()
                .map(|op| model.lift(op, &values, true).map(|v| encoded(&v)))
                .collect::<Result<Vec<_>, _>>()?,
                "null_short_circuit" => {
                    let mut trace = vec!["left"];
                    let value = model.coalesce(&model.construct("none", None)?, || {
                        trace.push("fallback");
                        Ok(raw("i32", "7"))
                    })?;
                    trace.push("receiver");
                    vec![
                        trace.join(","),
                        encoded(&value),
                        encoded(&model.construct("none", None)?),
                    ]
                }
                _ => vec![encoded(&model.lift(name, &values, true)?)],
            })
        })();
        let actual = match result {
            Ok(values) => json!({"kind":"value","value":values}),
            Err(e) => {
                json!({"kind":"exception","value":[e.exception_type().expect("runtime error")]})
            }
        };
        assert_eq!(actual, v["expected"], "{}", v["id"]);
    }
    assert_eq!(count, 628);
}
#[test]
fn csharp_03_t03_w12_arms_defaults_eager_fallback_and_error_order() {
    let b = bundle();
    let p = primitive("i32");
    let option = instance("option", vec![p.clone()]);
    let types = vec![
        option.clone(),
        instance("lookup", vec![option]),
        instance("result", vec![p.clone(), p.clone()]),
        instance("validation", vec![p.clone(), p.clone()]),
        instance("boundary_field", vec![p.clone()]),
        instance("bounded_sequence", vec![p]),
    ];
    let (r, c, ids) = fixture(&b, &types, json!({}));
    let option = OutcomeModel::new(&b, &r, &c, &ids[0]).unwrap();
    let some = option.construct("some", Some(raw("i32", "7"))).unwrap();
    let none = option.construct("none", None).unwrap();
    assert_eq!(encoded(&option.value_or_default(&none).unwrap()), "0");
    assert!(option.value_or(&some, raw("i64", "7")).is_err());
    assert_eq!(
        option.coalesce(&some, || panic!("eager coalesce")).unwrap(),
        raw("i32", "7")
    );
    assert_eq!(
        option.read(&none, "some"),
        Err(DomainError::InactivePayload)
    );
    assert_eq!(
        DomainError::InactivePayload.exception_type(),
        Some("System.InvalidOperationException")
    );
    let lookup = OutcomeModel::new(&b, &r, &c, &ids[1]).unwrap();
    let missing = lookup.construct("missing_key", None).unwrap();
    let null = lookup.construct("found", Some(none)).unwrap();
    let found = lookup.construct("found", Some(some)).unwrap();
    assert_ne!(missing, null);
    assert_ne!(null, found);
    for (index, arms) in [
        (0, vec!["none", "some"]),
        (1, vec!["missing_key", "found"]),
        (2, vec!["ok", "error"]),
        (3, vec!["valid", "invalid"]),
        (4, vec!["missing", "null", "value"]),
    ] {
        let model = OutcomeModel::new(&b, &r, &c, &ids[index]).unwrap();
        model
            .exhaustive(&arms.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap();
        assert!(model.exhaustive(&[arms[0].into()]).is_err());
        for arm in arms {
            let payload = match arm {
                "none" | "missing_key" | "missing" | "null" => None,
                "found" => Some(option.construct("none", None).unwrap()),
                "invalid" => Some(MonomorphicValue::Sequence {
                    type_id: ids[5].clone(),
                    elements: vec![raw("i32", "1")],
                }),
                _ => Some(raw("i32", "1")),
            };
            let v = model.construct(arm, payload.clone()).unwrap();
            assert_eq!(model.arm(&v).unwrap(), arm);
            if let Some(payload) = &payload {
                assert_eq!(model.read(&v, arm).unwrap(), payload);
            }
            assert!(model.construct("unknown", payload).is_err());
        }
    }
    assert!(domain_default(&b, &r, &c, &ids[2]).is_err());
    assert!(domain_default(&b, &r, &c, &ids[3]).is_err());
    let validation = OutcomeModel::new(&b, &r, &c, &ids[3]).unwrap();
    let errors = |values: Vec<i32>| MonomorphicValue::Sequence {
        type_id: ids[5].clone(),
        elements: values.iter().map(|v| raw("i32", &v.to_string())).collect(),
    };
    assert_eq!(
        validation.construct("invalid", Some(errors(vec![]))),
        Err(DomainError::EmptyInvalid)
    );
    assert_eq!(DomainError::EmptyInvalid.exception_type(), None);
    let a = validation
        .construct("invalid", Some(errors(vec![2, 1, 2])))
        .unwrap();
    let b = validation
        .construct("invalid", Some(errors(vec![3, 1])))
        .unwrap();
    assert_eq!(
        validation
            .read(&validation.append_errors(&a, &b).unwrap(), "invalid")
            .unwrap(),
        &errors(vec![2, 1, 2, 3, 1])
    );
    assert_eq!(
        validation.construct("invalid", Some(errors(vec![1; 257]))),
        Err(DomainError::Bound)
    );
    let full = validation
        .construct("invalid", Some(errors(vec![1; 256])))
        .unwrap();
    assert_eq!(validation.append_errors(&full, &a), Err(DomainError::Bound));
}
fn source_fixture(name: &str, kind: &str, members: &[(&str, Value)], enums: &[i64]) -> Value {
    let identity = json!({"kind":"type","namespace":"Example","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""});
    let id = csharp_practical_declaration_id(&identity).unwrap();
    let mut defaults = Map::new();
    let members:Vec<_>=members.iter().enumerate().map(|(ordinal,(name,t))|{let member=csharp_practical_stored_member_id(&id,name,t,"readonly_field").unwrap();defaults.insert(member.clone(),json!(0));json!({"id":member,"name":name,"type":t,"storage":"readonly_field","ordinal":ordinal,"required":false})}).collect();
    json!({"id":id,"identity":identity,"kind":kind,"members":members,"enum_values":enums.iter().map(ToString::to_string).collect::<Vec<_>>(),"enum_underlying":if kind=="enum"{json!("i32")}else{Value::Null},"actual_default":defaults,"public_default":true,"identity_sensitive":false,"source_sha256":format!("{:x}",Sha256::digest(name.as_bytes()))})
}
#[test]
fn csharp_03_t03_w12_binding_identity_all_fields_and_default_mutations() {
    let b = bundle();
    let p = primitive("i32");
    for (role, arms) in [
        ("option", vec!["none", "some"]),
        ("lookup", vec!["missing_key", "found"]),
        ("result", vec!["ok", "error"]),
        ("validation", vec!["valid", "invalid"]),
        ("boundary_field", vec!["missing", "null", "value"]),
    ] {
        let en = source_fixture(
            "Tag",
            "enum",
            &[],
            &(0..arms.len() as i64).collect::<Vec<_>>(),
        );
        let mut members = vec![
            ("Tag", json!({"kind":"source","id":en["id"]})),
            ("Value", p.clone()),
            ("Extra", p.clone()),
        ];
        if role == "result" {
            members.push(("Error", p.clone()));
        }
        if role == "validation" {
            members.push(("Errors", instance("bounded_sequence", vec![p.clone()])));
        }
        let source = source_fixture("Outcome", "readonly_struct", &members, &[]);
        let source_id = source["id"].as_str().unwrap();
        let mut args = vec![p.clone()];
        if matches!(role, "result" | "validation") {
            args.push(p.clone());
        }
        let semantic = instance(role, args);
        let mut types = vec![semantic, json!({"kind":"source","id":source_id})];
        if role == "validation" {
            types.push(instance("bounded_sequence", vec![p.clone()]));
        }
        let sources = json!({source_id:source.clone(),en["id"].as_str().unwrap():en.clone()});
        let (r, c, ids) = fixture(&b, &types, sources.clone());
        let roles = if role == "result" {
            vec![("tag", "Tag"), ("value", "Value"), ("error", "Error")]
        } else if role == "validation" {
            vec![("tag", "Tag"), ("value", "Value"), ("errors", "Errors")]
        } else {
            vec![("tag", "Tag"), ("value", "Value")]
        };
        let binding = SemanticBindingInput {
            enum_arms: Default::default(),
            source_type_id: source_id.into(),
            source_content_sha256: source["source_sha256"].as_str().unwrap().into(),
            role: role.into(),
            member_map: roles
                .iter()
                .map(|(role, name)| SemanticBindingMember {
                    role: (*role).into(),
                    member_id: source["members"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|m| m["name"] == *name)
                        .unwrap()["id"]
                        .as_str()
                        .unwrap()
                        .into(),
                })
                .collect(),
            tag_arms: arms
                .iter()
                .enumerate()
                .map(|(i, arm)| SemanticArmMapping {
                    source_tag: i.to_string(),
                    semantic_arm: (*arm).into(),
                })
                .collect(),
            inferred_argument_ids: if matches!(role, "result" | "validation") {
                vec![ty("i32"), ty("i32")]
            } else {
                vec![ty("i32")]
            },
            default_arm: match role {
                "option" => "none",
                "lookup" => "missing_key",
                _ => "ineligible",
            }
            .into(),
            bounds: if role == "validation" {
                vec![SemanticBound {
                    id: "errors".into(),
                    maximum: 256,
                }]
            } else {
                vec![]
            },
            operation_map: vec![],
        };
        let plan = OutcomeBindingPlan::new(&b, &r, &c, &binding, &BTreeMap::new()).unwrap();
        assert_eq!(plan.semantic_type_id(), ids[0]);
        assert!(plan.obligations().iter().all(|o| !o.discharged));
        assert_eq!(
            plan.obligations()
                .iter()
                .filter(|o| o.kind == "field_complete_reconstruction")
                .count(),
            members.len()
        );
        assert_eq!(plan.default_eligible(), matches!(role, "option" | "lookup"));
        for (index, arm) in arms.iter().enumerate() {
            let fields: Vec<_> = members
                .iter()
                .map(|(name, _)| NamedMonomorphicValue {
                    name: (*name).into(),
                    value: Box::new(match *name {
                        "Tag" => MonomorphicValue::Enum {
                            type_id: en["id"].as_str().unwrap().into(),
                            underlying: "i32".into(),
                            carrier: index.to_string(),
                        },
                        "Errors" => MonomorphicValue::Array {
                            type_id: ids[2].clone(),
                            elements: vec![raw("i32", "2")],
                        },
                        _ => raw("i32", "7"),
                    }),
                })
                .collect();
            let original = MonomorphicValue::Product {
                type_id: source_id.into(),
                fields,
            };
            let value = plan.project(&b, &r, &c, &original).unwrap();
            assert_eq!(
                OutcomeModel::new(&b, &r, &c, &ids[0])
                    .unwrap()
                    .arm(&value)
                    .unwrap(),
                *arm
            );
            plan.check_source_round_trip(&b, &r, &c, &original, &original)
                .unwrap();
            let mut changed = original.clone();
            if let MonomorphicValue::Product { fields, .. } = &mut changed {
                *fields.iter_mut().find(|f| f.name == "Extra").unwrap().value = raw("i32", "8");
            }
            assert_eq!(plan.project(&b, &r, &c, &changed).unwrap(), value);
            assert_eq!(
                plan.check_source_round_trip(&b, &r, &c, &original, &changed),
                Err(DomainError::ObservationLoss)
            );
        }
        for mutation in 0..7 {
            let mut bad = binding.clone();
            match mutation {
                0 => bad.source_content_sha256 = "0".repeat(64),
                1 => bad.member_map[0].member_id = bad.member_map[1].member_id.clone(),
                2 => bad.tag_arms[1].source_tag = bad.tag_arms[0].source_tag.clone(),
                3 => bad.inferred_argument_ids[0] = ty("i64"),
                4 => bad.default_arm = "some".into(),
                5 => bad.operation_map.push(SemanticOperationMapping {
                    operation: "value".into(),
                    member_id: "invented".into(),
                }),
                _ => bad.bounds.push(SemanticBound {
                    id: "unknown".into(),
                    maximum: 1,
                }),
            };
            assert!(OutcomeBindingPlan::new(&b, &r, &c, &bad, &BTreeMap::new()).is_err());
        }
    }
}
#[test]
fn csharp_03_t03_w12_six_profile_limits_and_nested_option_rejection() {
    let b = bundle();
    let p = primitive("i32");
    let types = vec![
        instance("validation", vec![p.clone(), p.clone()]),
        instance("bounded_sequence", vec![p.clone()]),
    ];
    let (r, c, ids) = fixture(&b, &types, json!({}));
    let model = OutcomeModel::new(&b, &r, &c, &ids[0]).unwrap();
    let profile = file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let mut count = 0;
    for v in profile["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W12")
    {
        count += 1;
        let n = v["inputs"]["value"].as_u64().unwrap() as usize;
        let accepted = if v["inputs"]["counter"] == "validation_errors" {
            model
                .construct(
                    "invalid",
                    Some(MonomorphicValue::Sequence {
                        type_id: ids[1].clone(),
                        elements: vec![raw("i32", "1"); n],
                    }),
                )
                .is_ok()
        } else {
            let mut t = p.clone();
            for _ in 0..n {
                t = instance("result", vec![t, p.clone()]);
            }
            let roots = json!([{"origin":"semantic_binding","provenance_id":"depth","type":t}]);
            canonical_closed_root_set_transport(&b, &roots, &json!({}))
                .and_then(|bytes| validate_closed_root_set(&b, &bytes))
                .and_then(|r| derive_closed_instances(&b, &r))
                .is_ok()
        };
        assert_eq!(accepted, v["expected"]["accept"] == true, "{}", v["id"]);
    }
    assert_eq!(count, 6);
    let nested = instance("option", vec![instance("option", vec![p])]);
    let roots = json!([{"origin":"semantic_binding","provenance_id":"nested","type":nested}]);
    assert!(canonical_closed_root_set_transport(&b, &roots, &json!({}))
        .and_then(|bytes| validate_closed_root_set(&b, &bytes))
        .and_then(|r| derive_closed_instances(&b, &r))
        .is_err());
}
#[test]
fn csharp_03_t03_w12_private_inputs_frozen_runtime_and_manifest() {
    let frozen = file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let rows:Vec<_>=frozen["vectors"].as_array().unwrap().iter().filter(|v|v["implementation_owner"]=="CSHARP-03-T03-W12").map(|v|json!({"id":v["id"],"operation":v["inputs"]["operation"],"inputs":v["inputs"]["inputs"],"expected":v["expected"]})).collect();
    assert_eq!(
        json!(rows),
        file("develop/migrations/csharp-03/domain/domain-runtime.json")
    );
    let record = file("develop/migrations/csharp-03/probes/runtime-foundation-data.json");
    for row in &rows {
        let id = row["id"]
            .as_str()
            .unwrap()
            .strip_prefix("nullable.runtime_")
            .unwrap();
        for observation in record["observations"].as_array().unwrap() {
            let actual = observation["vectors"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["id"] == id)
                .unwrap();
            assert_eq!(actual["observed"], row["expected"]);
        }
    }
    let path = "develop/migrations/csharp-03/domain/domain-inputs.json";
    let manifest = file(path);
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w12.domain_inputs.v1"
    );
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W12");
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 12);
    let mut previous = "";
    for f in files {
        let path = f["path"].as_str().unwrap();
        assert!(path > previous);
        previous = path;
        let bytes = read(path);
        assert_eq!(f["size_bytes"], bytes.len());
        assert_eq!(f["sha256"], format!("{:x}", Sha256::digest(&bytes)));
    }
    assert!(
        !file("develop/migrations/csharp-03/build-inputs/build-inputs.json")
            .to_string()
            .contains("PracticalDomain.cs")
    );
}

#[test]
fn csharp_03_t03_w12_pinned_source_harness_when_available() {
    let package = file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache=root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            cache
                .join(format!(
                    "{}.{}",
                    archive["id"].as_str().unwrap(),
                    archive["kind"].as_str().unwrap()
                ))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned cache"
    );
    if count == 0 {
        return;
    }
    let output =
        std::process::Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
            .arg("--test-domain")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn csharp_03_t03_w12_lifted_edges_and_ineligible_payload_default() {
    let b = bundle();
    let p = primitive("i32");
    let result = instance("result", vec![p.clone(), p.clone()]);
    let types = vec![
        instance("option", vec![p]),
        instance("option", vec![primitive("i64")]),
        instance("option", vec![result.clone()]),
        result,
    ];
    let (r, c, ids) = fixture(&b, &types, json!({}));
    let model = OutcomeModel::new(&b, &r, &c, &ids[0]).unwrap();
    let min = nullable(&model, "i32", "-2147483648");
    let minus = nullable(&model, "i32", "-1");
    let max = nullable(&model, "i32", "2147483647");
    let one = nullable(&model, "i32", "1");
    let zero = nullable(&model, "i32", "0");
    let none = model.construct("none", None).unwrap();
    assert_eq!(
        model
            .lift("divide", &[min.clone(), minus.clone()], false)
            .unwrap_err()
            .exception_type(),
        Some("System.OverflowException")
    );
    assert_eq!(
        model
            .lift("remainder", &[min.clone(), minus], false)
            .unwrap_err()
            .exception_type(),
        Some("System.OverflowException")
    );
    assert_eq!(
        encoded(
            &model
                .lift("add", &[max.clone(), one.clone()], false)
                .unwrap()
        ),
        "-2147483648"
    );
    assert_eq!(
        model
            .lift("add", &[max, one], true)
            .unwrap_err()
            .exception_type(),
        Some("System.OverflowException")
    );
    assert_eq!(
        model.lift("divide", &[none.clone(), zero], true).unwrap(),
        none
    );
    assert_eq!(
        encoded(
            &model
                .lift("negate", std::slice::from_ref(&min), false)
                .unwrap()
        ),
        "-2147483648"
    );
    let wide = OutcomeModel::new(&b, &r, &c, &ids[1])
        .unwrap()
        .construct("none", None)
        .unwrap();
    assert!(model.lift("add", &[none.clone(), wide], true).is_err());
    assert!(model.lift("and", &[none.clone(), none], true).is_err());
    let model = OutcomeModel::new(&b, &r, &c, &ids[2]).unwrap();
    let result = OutcomeModel::new(&b, &r, &c, &ids[3])
        .unwrap()
        .construct("ok", Some(raw("i32", "1")))
        .unwrap();
    let some = model.construct("some", Some(result.clone())).unwrap();
    assert_eq!(
        model.value_or_default(&some),
        Err(DomainError::DefaultIneligible)
    );
    assert_eq!(model.value_or(&some, result.clone()).unwrap(), result);
}
#[test]
fn csharp_03_t03_w12_application_lookup_option_dependency_and_commutation() {
    let b = bundle();
    let tag = source_fixture("PresenceTag", "enum", &[], &[0, 1]);
    let tag_type = json!({"kind":"source","id":tag["id"]});
    let option = source_fixture(
        "ApplicationOption",
        "readonly_struct",
        &[
            ("Tag", tag_type.clone()),
            ("Value", primitive("i32")),
            ("Extra", primitive("i32")),
        ],
        &[],
    );
    let lookup = source_fixture(
        "ApplicationLookup",
        "readonly_struct",
        &[
            ("Tag", tag_type),
            ("Value", json!({"kind":"source","id":option["id"]})),
        ],
        &[],
    );
    let option_type = instance("option", vec![primitive("i32")]);
    let types = vec![
        option_type.clone(),
        instance("lookup", vec![option_type]),
        json!({"kind":"source","id":lookup["id"]}),
    ];
    let (r, c, ids) = fixture(
        &b,
        &types,
        json!({tag["id"].as_str().unwrap():tag,option["id"].as_str().unwrap():option,lookup["id"].as_str().unwrap():lookup}),
    );
    let binding = |source: &Value, role: &str, arg: String, arms: [&str; 2]| SemanticBindingInput {
        enum_arms: Default::default(),
        source_type_id: source["id"].as_str().unwrap().into(),
        source_content_sha256: source["source_sha256"].as_str().unwrap().into(),
        role: role.into(),
        member_map: [("tag", "Tag"), ("value", "Value")]
            .iter()
            .map(|(role, name)| SemanticBindingMember {
                role: (*role).into(),
                member_id: source["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|m| m["name"] == *name)
                    .unwrap()["id"]
                    .as_str()
                    .unwrap()
                    .into(),
            })
            .collect(),
        tag_arms: arms
            .iter()
            .enumerate()
            .map(|(i, arm)| SemanticArmMapping {
                source_tag: i.to_string(),
                semantic_arm: (*arm).into(),
            })
            .collect(),
        inferred_argument_ids: vec![arg],
        default_arm: arms[0].into(),
        bounds: vec![],
        operation_map: vec![],
    };
    let mut first = binding(&option, "option", ty("i32"), ["none", "some"]);
    let method=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"Example","owner":option["id"],"name":"GetValue","parameter_type_ids":[],"result_type_id":ty("i32")})).unwrap();
    first.operation_map.push(SemanticOperationMapping {
        operation: "value".into(),
        member_id: method.clone(),
    });
    let captured = BTreeMap::from([(
        method.clone(),
        ClosedOperationSignature {
            id: method,
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![option["id"].as_str().unwrap().into()],
            normal_result_type_id: ty("i32"),
            ordered_checks: vec![],
        },
    )]);
    let first = OutcomeBindingPlan::new(&b, &r, &c, &first, &captured).unwrap();
    assert_eq!(
        first
            .obligations()
            .iter()
            .filter(|o| o.kind.starts_with("operation_"))
            .count(),
        3
    );
    let second = binding(&lookup, "lookup", ids[0].clone(), ["missing_key", "found"]);
    assert!(OutcomeBindingPlan::new(&b, &r, &c, &second, &BTreeMap::new()).is_err());
    let second =
        OutcomeBindingPlan::new_with_dependencies(&b, &r, &c, &second, &BTreeMap::new(), &[first])
            .unwrap();
    assert_eq!(second.semantic_type_id(), ids[1]);
    let enumeration = |n: &str| MonomorphicValue::Enum {
        type_id: tag["id"].as_str().unwrap().into(),
        underlying: "i32".into(),
        carrier: n.into(),
    };
    let product = |id: &str, fields: Vec<(&str, MonomorphicValue)>| MonomorphicValue::Product {
        type_id: id.into(),
        fields: fields
            .into_iter()
            .map(|(name, value)| NamedMonomorphicValue {
                name: name.into(),
                value: Box::new(value),
            })
            .collect(),
    };
    let payload = product(
        option["id"].as_str().unwrap(),
        vec![
            ("Tag", enumeration("0")),
            ("Value", raw("i32", "7")),
            ("Extra", raw("i32", "8")),
        ],
    );
    let source = product(
        lookup["id"].as_str().unwrap(),
        vec![("Tag", enumeration("1")), ("Value", payload)],
    );
    let projected = second.project(&b, &r, &c, &source).unwrap();
    let model = OutcomeModel::new(&b, &r, &c, &ids[1]).unwrap();
    assert_eq!(model.arm(&projected).unwrap(), "found");
    assert_eq!(encoded(model.read(&projected, "found").unwrap()), "none");
    let mut changed = source.clone();
    if let MonomorphicValue::Product { fields, .. } = &mut changed {
        if let MonomorphicValue::Product { fields, .. } = fields[1].value.as_mut() {
            *fields[2].value = raw("i32", "9");
        }
    }
    assert_eq!(second.project(&b, &r, &c, &changed).unwrap(), projected);
    assert_eq!(
        second.check_source_round_trip(&b, &r, &c, &source, &changed),
        Err(DomainError::ObservationLoss)
    );
}

#[path = "support/csharp_practical_business.rs"]
mod business;

#[test]
fn csharp_03_t03_w14_contract_literals_preserve_frozen_values_and_utf16() {
    let b = bundle();
    let (r, c, _) = fixture(&b, &[], json!({}));
    let env = DataContractEnvironment::default();
    let check = |token: &str, value: &str| {
        parse_data_contract_expression(
            &b,
            &r,
            &c,
            &env,
            format!(
                r#"{{"tag":"literal","type_id":"{}","value":{value}}}"#,
                ty(token)
            )
            .as_bytes(),
        )
    };
    for (token, literal) in [
        ("bool", "false"),
        ("i32", r#""-12""#),
        ("u64", r#""18446744073709551615""#),
        ("decimal", r#""1.25""#),
        ("f32", r#""80000000""#),
        ("string", r#""\ud800x\udfff""#),
        ("char", r#""\ud800""#),
        ("unit", "null"),
    ] {
        let result = check(token, literal).unwrap();
        assert_eq!(result.type_id(), ty(token));
        assert_eq!(result.nodes(), 1);
        let encoded = canonical_practical_json_bytes(result.value()).unwrap();
        assert!(encoded.ends_with(format!("{literal}}}").as_bytes()));
    }
    for (token, literal) in [
        ("bool", r#""false""#),
        ("i32", "12"),
        ("i32", r#""2147483648""#),
        ("decimal", r#""1.250""#),
        ("char", r#""ab""#),
        ("unit", "{}"),
        (
            "bool",
            r#"{"kind":"bool","type_id":"mpk.csharp.value.bool.v1","value":false}"#,
        ),
    ] {
        assert!(check(token, literal).is_err(), "{token}: {literal}");
    }
    for raw in [
        r#"{"type_id":"mpk.csharp.value.bool.v1","tag":"literal","value":false}"#,
        r#"{"tag":"literal","type_id":"mpk.csharp.value.bool.v1","value":false,"value":true}"#,
        r#"{"tag":"literal","type_id":"mpk.csharp.value.bool.v1","value":false,"extra":0}"#,
    ] {
        assert!(parse_data_contract_expression(&b, &r, &c, &env, raw.as_bytes()).is_err());
    }
}

#[test]
fn csharp_03_t03_w14_contract_quantifier_scopes_and_frozen_nesting_bound() {
    let b = bundle();
    let (r, c, _) = fixture(&b, &[], json!({}));
    let env = DataContractEnvironment::default();
    let literal = r#"{"tag":"literal","type_id":"mpk.csharp.value.i32.v1","value":"0"}"#;
    let mut expression =
        r#"{"tag":"literal","type_id":"mpk.csharp.value.bool.v1","value":true}"#.to_owned();
    for depth in 1..=5 {
        expression = format!(
            r#"{{"tag":"bounded_forall","type_id":"mpk.csharp.value.bool.v1","binding_id":"q{depth}","lower":{literal},"upper":{literal},"body":{expression}}}"#
        );
        assert_eq!(
            parse_data_contract_expression(&b, &r, &c, &env, expression.as_bytes()).is_ok(),
            depth <= 4
        );
    }
    let escaped = r#"{"tag":"variable","type_id":"mpk.csharp.value.i32.v1","binding_id":"q1"}"#;
    assert!(parse_data_contract_expression(&b, &r, &c, &env, escaped.as_bytes()).is_err());
}

#[path = "support/csharp_practical_data_context.rs"]
mod data_context;

#[test]
fn csharp_03_t03_w14_captured_source_identity_provenance_and_default_mutations() {
    let source = b"namespace Data;public readonly struct Value{public readonly int Amount;public Value(int amount){Amount=amount;}}public static class Entry{public static int Run(Value value){return new Value(value.Amount).Amount;}}\n";
    let b = bundle();
    let bytes = read("develop/migrations/csharp-03/data-phase/data-source.json");
    let facts: Value = serde_json::from_slice(&bytes).unwrap();
    let (context, captures) =
        data_context::context(&b, facts["selected_root_ids"][0].as_str().unwrap(), source);
    let actual =
        ValidatedDataSource::import_captured_facts(&b, &context, &captures, &bytes).unwrap();
    let emitted = emit_data_phase(&b, &context, &captures, &actual).unwrap();
    assert_eq!(emitted.vir().functions().len(), 2);
    assert_eq!(
        emitted
            .vir()
            .functions()
            .iter()
            .flat_map(|f| &f.blocks)
            .filter(|b| b.invocation.is_some())
            .count(),
        4
    );
    assert_eq!(actual.callables().len(), 2);
    assert_eq!(actual.source_roots().source_type_count(), 1);
    let signatures = actual
        .callables()
        .iter()
        .map(|c| (c.id().to_owned(), c.logical_signature(&b).unwrap()))
        .collect::<BTreeMap<_, _>>();
    actual
        .validate_source_call_signatures(&b, &signatures)
        .unwrap();
    for mutation in 0..4 {
        let mut changed = signatures.clone();
        let id = actual
            .callables()
            .iter()
            .find(|c| c.identity()["kind"] == "method")
            .unwrap()
            .id();
        match mutation {
            0 => {
                changed.get_mut(id).unwrap().argument_type_ids[0] =
                    "mpk.csharp.value.bool.v1".into()
            }
            1 => {
                changed.get_mut(id).unwrap().normal_result_type_id =
                    "mpk.csharp.value.bool.v1".into()
            }
            2 => changed.get_mut(id).unwrap().tag = ClosedOperationTag::ConstructorExecute,
            3 => {
                changed.remove(id);
            }
            _ => unreachable!(),
        }
        assert!(
            actual
                .validate_source_call_signatures(&b, &changed)
                .is_err(),
            "source signature mutation {mutation}"
        );
    }
    for mutation in 0..22 {
        let mut changed = facts.clone();
        match mutation {
            0 => changed["types"][0]["name"] = json!("Other"),
            1 => changed["types"][0]["source_sha256"] = json!("0".repeat(64)),
            2 => changed["callables"][0]["body_sha256"] = json!("0".repeat(64)),
            3 => changed["types"][0]["recursive_default"]["nodes"][0]["scalar"] = json!("1"),
            4 => changed["types"][0]["recursive_default"]["nodes"][1]["members"] = json!([1]),
            5 => changed["callables"][0]["parameters"] = json!([]),
            6 => {
                let t = changed["types"][0].clone();
                changed["types"].as_array_mut().unwrap().push(t);
            }
            7 => changed["callables"][0]["end_byte"] = json!(source.len() + 1),
            8 => changed["sources"][0]["size_bytes"] = json!(0),
            9 => changed["source_containers"] = json!([]),
            10 => changed["source_containers"][0]["name"] = json!("Other"),
            11 => {
                changed["source_containers"][0]["start_byte"] =
                    changed["source_containers"][0]["end_byte"].clone()
            }
            20 => changed["source_obligations"][0]["kind"] = json!("unknown_obligation"),
            21 => changed["source_obligations"][0]["family"] = json!("string"),
            12..=19 => {
                let callable = changed["callables"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| !c["initialization_plans"].as_array().unwrap().is_empty())
                    .unwrap();
                let plan = &mut callable["initialization_plans"][0];
                match mutation {
                    12 => callable["initialization_plans"] = json!([]),
                    13 => plan["node_ordinal"] = json!(0),
                    14 => plan["constructor_id"] = json!("mpk.csharp.source.invalid"),
                    15 => plan["steps"][0]["expression_ordinal"] = json!(0),
                    16 => {
                        let steps = plan["steps"].as_array_mut().unwrap();
                        let n = steps.len();
                        steps.swap(n - 1, n - 2);
                    }
                    17 => plan["steps"][2]["exceptional_exit"] = json!("no_value"),
                    18 => plan["possibly_assigned"] = json!(u32::MAX),
                    19 => plan["member_order"] = json!([]),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        assert!(
            ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(&changed).unwrap()
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn csharp_03_t03_w14_all_concrete_shared_routes_are_exact() {
    let b = bundle();
    let types = vec![
        instance("option", vec![primitive("i32")]),
        instance("ordered_set", vec![primitive("decimal")]),
        instance(
            "result",
            vec![primitive("string"), primitive("parse_error")],
        ),
    ];
    let (r, c, _) = fixture(&b, &types, json!({}));
    let routes = derive_data_type_routes(&b, &r, &c).unwrap();
    validate_data_type_routes(&b, &r, &c, &routes).unwrap();
    assert!(routes
        .iter()
        .any(|row| row.type_id == ty("decimal")
            && row.codecs == ["decimal.fixed", "decimal.normalized"]));
    for row in 0..routes.len() {
        let mut changed = routes.clone();
        changed.remove(row);
        assert!(validate_data_type_routes(&b, &r, &c, &changed).is_err());
        let mut changed = routes.clone();
        changed[row].equality = "custom.equal".into();
        assert!(validate_data_type_routes(&b, &r, &c, &changed).is_err());
    }
}

#[test]
fn csharp_03_t03_w14_captures_initialization_plans_with_constructor_provenance() {
    let b = bundle();
    let requests: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/object-construction-requests.json",
    ))
    .unwrap();
    let responses: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/object-construction-responses.json",
    ))
    .unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 8);
    assert_eq!(responses.as_array().unwrap().len(), 8);
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let facts = response
            .get("facts")
            .unwrap_or_else(|| panic!("capture {id}: {response}"));
        let (context, captures) = data_context::context(
            &b,
            request["roots"][0].as_str().unwrap(),
            request["inputs"][0]["utf8"].as_str().unwrap().as_bytes(),
        );
        let import = |facts: &Value| {
            ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(facts).unwrap(),
            )
        };
        let actual = import(facts).unwrap_or_else(|error| panic!("source {id}: {error:?}"));
        let emitted = emit_data_phase(&b, &context, &captures, &actual)
            .unwrap_or_else(|error| panic!("object emission {id}: {error:?}"));
        assert!(emitted
            .vir()
            .functions()
            .iter()
            .any(|f| f.object_protocol.is_some()));
        {
            use mpk_vc::csharp_practical_vir_validation as v;
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: emitted.closure().roots().canonical_json(),
                closed_instances_transport: emitted.closure().closed().canonical_json(),
                semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
                required_checks_transport: emitted.operations().required_checks().canonical_bytes(),
                operations_transport: emitted.operations().operations().canonical_bytes(),
            };
            for mutation in 0..5 {
                let mut functions = emitted.vir().functions().to_vec();
                let function = functions
                    .iter_mut()
                    .find(|f| f.id == request["roots"][0])
                    .unwrap();
                let plan = function.object_protocol.as_ref().unwrap().initializations[0].clone();
                match mutation {
                    0 => {}
                    1 => function.object_protocol = None,
                    2 => function
                        .object_protocol
                        .as_mut()
                        .unwrap()
                        .initializations
                        .clear(),
                    3 => {
                        let begin = function
                            .blocks
                            .iter()
                            .find(|b| b.node.id == plan.begin_node_id)
                            .unwrap()
                            .invocation
                            .as_ref()
                            .unwrap()
                            .result
                            .clone();
                        function
                            .blocks
                            .iter_mut()
                            .find(|b| b.node.id == plan.finalize_node_id)
                            .unwrap()
                            .invocation
                            .as_mut()
                            .unwrap()
                            .operands[0] = begin;
                    }
                    4 => {
                        let constructor = functions
                            .iter_mut()
                            .find(|f| {
                                f.object_protocol
                                    .as_ref()
                                    .is_some_and(|p| p.constructor_owner.is_some())
                            })
                            .unwrap();
                        constructor.object_protocol = None;
                    }
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        functions,
                        source_obligations: emitted.vir().source_obligations().to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_ok(),
                    mutation == 0,
                    "object import {id}/{mutation}"
                );
            }
            if id == "initializer_exception" || id == "constructor_exception" {
                let mut functions = emitted.vir().functions().to_vec();
                let function = functions
                    .iter_mut()
                    .find(|f| {
                        f.object_protocol
                            .as_ref()
                            .is_some_and(|p| !p.exceptional_discards.is_empty())
                    })
                    .unwrap();
                function
                    .object_protocol
                    .as_mut()
                    .unwrap()
                    .exceptional_discards
                    .clear();
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        functions,
                        source_obligations: emitted.vir().source_obligations().to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                    "object cleanup {id}"
                );
            }
        }
        let root = actual
            .callables()
            .iter()
            .find(|c| c.id() == request["roots"][0])
            .unwrap();
        assert_eq!(root.initialization_plans().len(), 1);
        let plan = &root.initialization_plans()[0];
        assert!(actual
            .constructor_assignment(&plan.constructor_id)
            .is_some());
        let constructor = actual
            .callables()
            .iter()
            .find(|c| c.id() == plan.constructor_id)
            .unwrap();
        let logical = constructor.logical_signature(&b).unwrap();
        let closed = derive_closed_instances(&b, actual.source_roots()).unwrap();
        let private =
            object_constructor_execution_signature(&b, actual.source_roots(), &closed, constructor)
                .unwrap();
        assert_eq!(logical.id, private.id);
        assert_eq!(logical.normal_result_type_id, plan.type_id);
        assert_eq!(private.tag, ClosedOperationTag::ConstructorExecute);
        assert_eq!(private.argument_type_ids[1..], logical.argument_type_ids);
        assert_eq!(private.argument_type_ids[0], private.normal_result_type_id);
        assert_ne!(private.normal_result_type_id, logical.normal_result_type_id);
        assert!(
            object_constructor_execution_signature(&b, actual.source_roots(), &closed, root)
                .is_err()
        );
        let roots: Value = serde_json::from_slice(actual.source_roots().canonical_json()).unwrap();
        assert!(roots["roots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["provenance_id"] == format!("{}.object_construction", plan.type_id)));
        for mutation in 0..5 {
            let mut changed = facts.clone();
            let callable = changed["callables"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|c| c["id"] == request["roots"][0])
                .unwrap();
            let plan = &mut callable["initialization_plans"][0];
            match mutation {
                0 => plan["steps"].as_array_mut().unwrap().reverse(),
                1 => {
                    plan["definitely_assigned"] =
                        json!(plan["definitely_assigned"].as_u64().unwrap() ^ 1)
                }
                2 => {
                    plan["possibly_assigned"] =
                        json!(plan["possibly_assigned"].as_u64().unwrap() ^ 1)
                }
                3 => plan["constructor_id"] = request["roots"][0].clone(),
                4 => {
                    let step = plan["steps"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|s| s["kind"] == "EvaluateInitializer")
                        .unwrap();
                    step["exceptional_exit"] = json!("no_value");
                }
                _ => unreachable!(),
            }
            assert!(import(&changed).is_err(), "plan mutation {id}/{mutation}");
        }
        if id == "implicit_required_string" {
            let mut changed = facts.clone();
            let constructor = changed["callables"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|c| c["is_synthesized_constructor"] == true)
                .unwrap();
            constructor["is_synthesized_constructor"] = json!(false);
            assert!(
                import(&changed).is_err(),
                "synthetic source identity cannot lose its marker"
            );
        }
        if id == "required_string" {
            check_object_protocol_mutations(&b, &actual, &closed);
        }
    }
}

fn check_object_protocol_mutations(
    b: &ValidatedFoundationBundle,
    source: &ValidatedDataSource,
    closed: &ClosedInstanceSet,
) {
    use mpk_vc::csharp_practical_vir_validation as v;
    let roots = source.source_roots();
    let callable = source
        .callables()
        .iter()
        .find(|c| c.id() == source.selected_root_ids()[0])
        .unwrap();
    let plan = &callable.initialization_plans()[0];
    let constructor = source
        .callables()
        .iter()
        .find(|c| c.id() == plan.constructor_id)
        .unwrap();
    let types = source.source_types();
    let members = types[&plan.type_id]["members"].as_array().unwrap();
    let member = |name: &str| {
        members.iter().find(|m| m["name"] == name).unwrap()["id"]
            .as_str()
            .unwrap()
    };
    let signatures = [
        object_construction_signature(roots, closed, &format!("object.begin.{}", plan.type_id))
            .unwrap(),
        object_constructor_execution_signature(b, roots, closed, constructor).unwrap(),
        object_construction_signature(roots, closed, &format!("object.write.{}", member("Name")))
            .unwrap(),
        object_construction_signature(roots, closed, &format!("object.finalize.{}", plan.type_id))
            .unwrap(),
        source_field_operation(roots, closed, member("Amount")).unwrap(),
    ];
    let node = |ordinal: usize| format!("{}.node.{ordinal:06}", callable.id());
    let value = |ordinal: usize, type_id: &str| TypedValueRef {
        id: format!("{}.value.{ordinal:06}", callable.id()),
        type_id: type_id.into(),
    };
    let results = signatures
        .iter()
        .enumerate()
        .map(|(i, s)| value(i, &s.normal_result_type_id))
        .collect::<Vec<_>>();
    let text = value(9, "mpk.csharp.value.string.v1");
    let mut blocks = (0..8)
        .map(|ordinal| v::PracticalVirBlock {
            node: ControlNode {
                id: node(ordinal),
                ordinal: ordinal as u32,
                tag: if ordinal == 0 {
                    ControlNodeTag::Entry
                } else if ordinal == 6 {
                    ControlNodeTag::Return
                } else if ordinal == 7 {
                    ControlNodeTag::Exit
                } else {
                    ControlNodeTag::Operation
                },
                condition_type_id: None,
                normal_successor_ids: if ordinal < 6 {
                    vec![node(ordinal + 1)]
                } else {
                    vec![]
                },
                exceptional_successors: vec![],
                abrupt: if ordinal == 6 {
                    Some(AbruptCompletion::Return {
                        value_type_id: Some("mpk.csharp.value.i32.v1".into()),
                    })
                } else if ordinal == 7 {
                    Some(AbruptCompletion::Normal)
                } else {
                    None
                },
                loop_id: None,
                region_stack: vec![],
            },
            phi_values: vec![],
            literal_values: vec![],
            exception_values: vec![],
            condition_value_id: None,
            return_value_ids: vec![],
            abrupt_value_id: None,
            handler_exception_source_id: None,
            handler_exception_value: None,
            invocation: None,
            ownership_in: vec![],
            construction_actions: vec![],
            ownership_out: vec![],
        })
        .collect::<Vec<_>>();
    for (i, signature) in signatures.iter().enumerate() {
        validate_closed_operation_signature(roots, closed, signature).unwrap();
        let operands = match i {
            0 => vec![],
            1 => vec![results[0].clone()],
            2 => vec![results[1].clone(), text.clone()],
            3 => vec![results[2].clone()],
            4 => vec![results[3].clone()],
            _ => unreachable!(),
        };
        blocks[i + 1].invocation = Some(OperationInvocation {
            operation_id: signature.id.clone(),
            operands,
            result: results[i].clone(),
            ordered_check_ids: vec![],
            normal_successor_id: node(i + 2),
            exceptional_successors: vec![],
        });
    }
    blocks[3].literal_values.push(v::PracticalVirLiteral {
        result: text,
        value: MonomorphicValue::String {
            type_id: "mpk.csharp.value.string.v1".into(),
            utf16: vec![111, 107],
        },
    });
    blocks[6].return_value_ids = vec![results[4].id.clone()];
    let mut operations = signatures
        .iter()
        .map(|s| (s.id.clone(), s.clone()))
        .collect::<BTreeMap<_, _>>();
    operations.insert(callable.id().into(), callable.logical_signature(b).unwrap());
    let function = v::PracticalVirFunction {
        control_protocol: None,
        id: callable.id().into(),
        parameter_values: vec![value(10, &plan.type_id)],
        result_type_ids: vec!["mpk.csharp.value.i32.v1".into()],
        blocks,
        loops: vec![],
        patterns: vec![],
        exception_regions: vec![],
        unwind_plans: vec![],
        object_protocol: Some(v::PracticalObjectProtocol {
            constructor_owner: None,
            initializations: vec![v::PracticalObjectInitialization {
                source_node_ordinal: plan.node_ordinal,
                begin_node_id: node(1),
                constructor_node_id: node(2),
                assignment_node_ids: vec![node(3)],
                finalize_node_id: node(4),
            }],
            exceptional_discards: vec![],
        }),
    };
    let check = |f: &v::PracticalVirFunction| {
        v::validate_object_construction_protocol(f, source, b, roots, closed, &operations)
    };
    check(&function).unwrap();
    for mutation in 0..7 {
        let mut changed = function.clone();
        match mutation {
            0 => changed.object_protocol = None,
            1 => changed.blocks[3].invocation.as_mut().unwrap().operands[0] = results[0].clone(),
            2 => changed.blocks[5].invocation.as_mut().unwrap().operands[0] = results[2].clone(),
            3 => {
                changed.object_protocol.as_mut().unwrap().initializations[0].source_node_ordinal =
                    usize::MAX
            }
            4 => changed.object_protocol.as_mut().unwrap().initializations[0]
                .assignment_node_ids
                .clear(),
            5 => changed
                .object_protocol
                .as_mut()
                .unwrap()
                .exceptional_discards
                .push(v::PracticalObjectDiscard {
                    exit_node_id: node(7),
                    origin_value_ids: vec![results[0].id.clone()],
                }),
            6 => changed.blocks[4].invocation.as_mut().unwrap().operands[0] = results[1].clone(),
            _ => unreachable!(),
        }
        assert!(
            check(&changed).is_err(),
            "object protocol mutation {mutation}"
        );
    }
}

#[test]
fn csharp_03_t03_w14_source_case_matrix_emits_and_independently_imports() {
    import_source_cases(&read(
        "develop/migrations/csharp-03/data-phase/data-source-cases.json",
    ));
}
fn import_source_cases(bytes: &[u8]) {
    let b = bundle();
    let cases: Value = serde_json::from_slice(bytes).unwrap();
    for case in cases.as_array().unwrap() {
        let source = case["source_utf8"].as_str().unwrap().as_bytes();
        let facts = &case["facts"];
        let fallible = case["name"].as_str().unwrap().starts_with("fallible_");
        let (context, captures) = if fallible {
            fallible_business_context(&b, facts, source)
        } else if case["name"] == "money_carrier" {
            money_carrier_context(&b, facts, source)
        } else {
            data_context::context(&b, facts["selected_root_ids"][0].as_str().unwrap(), source)
        };
        let regenerated = source_facts_for_captures(&captures, facts).unwrap();
        let facts = &regenerated;
        let actual = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap_or_else(|e| panic!("source {}: {e:?}", case["name"]));
        let first = emit_data_phase(&b, &context, &captures, &actual)
            .unwrap_or_else(|e| panic!("emission {}: {e:?}", case["name"]));
        let second = emit_data_phase(&b, &context, &captures, &actual).unwrap();
        assert_eq!(
            first.vir().canonical_bytes(),
            second.vir().canonical_bytes()
        );
        assert_eq!(
            first.artifacts().canonical_bytes(),
            second.artifacts().canonical_bytes()
        );
        assert_eq!(
            first.vir().functions().len(),
            facts["callables"].as_array().unwrap().len()
        );
        if case["name"] == "nullable_coalesce" || case["name"] == "nullable_fallback" {
            let function = first
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == facts["selected_root_ids"][0])
                .unwrap();
            let division = function
                .blocks
                .iter()
                .find(|b| {
                    b.invocation
                        .as_ref()
                        .is_some_and(|i| i.operation_id.starts_with("integer.i32.divide."))
                })
                .unwrap();
            fn reaches(
                function: &mpk_vc::csharp_practical_vir_validation::PracticalVirFunction,
                start: &str,
                target: &str,
            ) -> bool {
                let mut seen = std::collections::BTreeSet::new();
                let mut pending = vec![start];
                while let Some(id) = pending.pop() {
                    if id == target {
                        return true;
                    }
                    if seen.insert(id) {
                        pending.extend(
                            function
                                .blocks
                                .iter()
                                .find(|b| b.node.id == id)
                                .unwrap()
                                .node
                                .normal_successor_ids
                                .iter()
                                .map(String::as_str),
                        );
                    }
                }
                false
            }
            if case["name"] == "nullable_coalesce" {
                let branch = function
                    .blocks
                    .iter()
                    .find(|b| b.node.tag == ControlNodeTag::Branch)
                    .unwrap();
                assert!(!reaches(
                    function,
                    &branch.node.normal_successor_ids[0],
                    &division.node.id
                ));
                assert!(reaches(
                    function,
                    &branch.node.normal_successor_ids[1],
                    &division.node.id
                ));
            } else {
                let fallback = function
                    .blocks
                    .iter()
                    .find(|b| {
                        b.invocation
                            .as_ref()
                            .is_some_and(|i| i.operation_id.ends_with(".value_or"))
                    })
                    .unwrap();
                assert!(reaches(function, &division.node.id, &fallback.node.id));
                assert!(!reaches(function, &fallback.node.id, &division.node.id));
            }
        }
        if case["name"] == "array_store_order" || case["name"] == "array_update_order" {
            let function = first
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == facts["selected_root_ids"][0])
                .unwrap();
            let mut id = function.blocks[0].node.id.as_str();
            let mut calls = vec![];
            loop {
                let block = function.blocks.iter().find(|b| b.node.id == id).unwrap();
                if let Some(call) = &block.invocation {
                    calls.push(call.operation_id.as_str());
                }
                if block.node.normal_successor_ids.is_empty() {
                    break;
                }
                assert_eq!(block.node.normal_successor_ids.len(), 1);
                id = &block.node.normal_successor_ids[0];
            }
            let divide = calls
                .iter()
                .position(|id| id.starts_with("integer.i32.divide."))
                .unwrap();
            let read = calls.iter().position(|id| id.ends_with(".read")).unwrap();
            let write = calls
                .iter()
                .position(|id| id.ends_with(".rewrite"))
                .unwrap();
            assert!(divide < write);
            assert_eq!(read < divide, case["name"] == "array_update_order");
        }
        if case["name"] == "unused_assigned_getter" {
            for mutation in 0..2 {
                let mut changed = facts.clone();
                let callables = changed["callables"].as_array_mut().unwrap();
                if mutation == 0 {
                    callables
                        .iter_mut()
                        .find(|c| c["is_property_getter"] == true)
                        .unwrap()["is_property_getter"] = json!(false);
                } else {
                    callables
                        .iter_mut()
                        .find(|c| c["identity"]["name"] == "Run")
                        .unwrap()["is_property_getter"] = json!(true);
                }
                assert!(
                    ValidatedDataSource::import_captured_facts(
                        &b,
                        &context,
                        &captures,
                        &serde_json::to_vec(&changed).unwrap()
                    )
                    .is_err(),
                    "property root mutation {mutation}"
                );
            }
        }
        if case["name"] == "reference_nullable_call" {
            let function = first
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == facts["selected_root_ids"][0])
                .unwrap();
            let division = function
                .blocks
                .iter()
                .position(|b| {
                    b.invocation
                        .as_ref()
                        .is_some_and(|i| i.operation_id.starts_with("integer.i32.divide."))
                })
                .unwrap();
            let receiver_check = function
                .blocks
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    b.invocation
                        .as_ref()
                        .is_some_and(|i| i.operation_id.starts_with("reference.value."))
                })
                .map(|(i, _)| i)
                .max()
                .unwrap();
            assert!(
                division < receiver_check,
                "instance receiver check must follow argument evaluation"
            );
        }
        if case["name"] == "array_dynamic_reinitialize"
            || case["name"] == "array_symbolic_initialize"
        {
            let function = first
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == facts["selected_root_ids"][0])
                .unwrap();
            let query = function
                .blocks
                .iter()
                .find(|b| {
                    b.invocation
                        .as_ref()
                        .is_some_and(|i| i.operation_id.starts_with("construction.complete."))
                })
                .unwrap();
            let branch = function
                .blocks
                .iter()
                .find(|b| b.node.id == query.node.normal_successor_ids[0])
                .unwrap();
            assert_eq!(branch.node.tag, ControlNodeTag::Branch);
            for (target, operation) in branch
                .node
                .normal_successor_ids
                .iter()
                .zip([".rewrite", ".fill"])
            {
                let arm = function
                    .blocks
                    .iter()
                    .find(|b| &b.node.id == target)
                    .unwrap();
                assert!(arm
                    .invocation
                    .as_ref()
                    .unwrap()
                    .operation_id
                    .ends_with(operation));
            }
        }
        if case["name"] == "array_branch_write" {
            let function = first
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == facts["selected_root_ids"][0])
                .unwrap();
            let constructors = function
                .blocks
                .iter()
                .filter_map(|b| b.invocation.as_ref())
                .filter(|i| i.operation_id.ends_with(".allocate"))
                .map(|i| i.result.type_id.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            let phis = function
                .blocks
                .iter()
                .flat_map(|b| &b.phi_values)
                .filter(|p| constructors.contains(p.value.type_id.as_str()))
                .collect::<Vec<_>>();
            assert_eq!(phis.len(), 1);
            assert_eq!(phis[0].incoming.len(), 2);
            assert_ne!(phis[0].incoming[0].value_id, phis[0].incoming[1].value_id);
        }
        if !first.vir().source_obligations().is_empty() {
            use mpk_vc::csharp_practical_vc_model::{
                generate_csharp_practical_vc, PracticalVcSource,
            };
            let vc = generate_csharp_practical_vc(PracticalVcSource {
                artifact_context: &context,
                captured_inputs: &captures,
                vir: first.vir(),
            })
            .unwrap();
            assert_eq!(
                vc.obligation_groups()
                    .iter()
                    .flat_map(|g| g.subject_ids())
                    .filter(|id| id.starts_with("source_obligation:"))
                    .count(),
                first.vir().source_obligations().len()
            );
        }
        if case["name"] == "decimal_round_modes" {
            let ids = first
                .vir()
                .functions()
                .iter()
                .flat_map(|f| &f.blocks)
                .filter_map(|b| b.invocation.as_ref())
                .map(|i| i.operation_id.as_str())
                .filter(|id| id.starts_with("decimal.round."))
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                ids,
                std::collections::BTreeSet::from([
                    "decimal.round.ToEven.2",
                    "decimal.round.AwayFromZero.1"
                ])
            );
            for mutation in 0..4 {
                let mut changed = facts.clone();
                let callable = changed["callables"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| {
                        c["data_steps"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|s| s["operation"] == "decimal.round")
                    })
                    .unwrap();
                let steps = callable["data_steps"].as_array_mut().unwrap();
                let step = steps
                    .iter_mut()
                    .find(|s| s["operation"] == "decimal.round")
                    .unwrap();
                match mutation {
                    0 => step["rounding"] = json!("AwayFromZero"),
                    1 => step["rounding"] = json!(""),
                    2 => step["operand_ordinals"] = json!([999999]),
                    3 => step["family"] = json!("string"),
                    _ => unreachable!(),
                }
                assert!(
                    ValidatedDataSource::import_captured_facts(
                        &b,
                        &context,
                        &captures,
                        &serde_json::to_vec(&changed).unwrap()
                    )
                    .is_err(),
                    "source recipe mutation {mutation}"
                );
            }
        }
        if fallible {
            use mpk_vc::csharp_practical_vir_validation as v;
            let commutations = first.vir().binding_commutations();
            assert_eq!(commutations.len(), 1);
            assert_eq!(
                commutations[0]
                    .returned_result
                    .as_ref()
                    .unwrap()
                    .ordered_errors
                    .len(),
                if case["name"] == "fallible_money" {
                    3
                } else {
                    2
                }
            );
            assert!(commutations[0].source_operation.ordered_checks.is_empty());
            assert!(first.closure().obligations().iter().all(|o| !o.discharged));
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: first.closure().roots().canonical_json(),
                closed_instances_transport: first.closure().closed().canonical_json(),
                semantic_bindings_transport: first.closure().bindings().canonical_bytes(),
                required_checks_transport: first.operations().required_checks().canonical_bytes(),
                operations_transport: first.operations().operations().canonical_bytes(),
            };
            for mutation in 0..if case["name"] == "fallible_money" {
                10
            } else {
                5
            } {
                let mut changed = commutations.to_vec();
                let result_projection_id = changed[0].result_projection_id.clone();
                let returned = changed[0].returned_result.as_mut().unwrap();
                match mutation {
                    0 => returned.ordered_errors.swap(0, 1),
                    1 => {
                        returned.ordered_errors[0].source_carrier =
                            returned.ordered_errors[1].source_carrier.clone()
                    }
                    2 => returned.error_type_id = "mpk.csharp.value.i32.v1".into(),
                    3 => returned.success_projection_id = result_projection_id,
                    4 => changed[0].returned_result = None,
                    5 => changed[0].rounding_operands.clear(),
                    6 => changed[0].rounding_operands[0].ordinal = 0,
                    7 => changed[0].rounding_operands[0].source_type_id = ty("i32"),
                    8 => {
                        changed[0].rounding_operands[0]
                            .enum_arms
                            .insert("ToEven".into(), "20".into());
                    }
                    9 => {
                        changed[0].rounding_operands[0].enum_arms.remove("ToZero");
                    }
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        source_obligations: first.vir().source_obligations().to_vec(),
                        functions: first.vir().functions().to_vec(),
                        binding_projections: first.vir().binding_projections().to_vec(),
                        binding_commutations: changed,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                    "returned result mutation {mutation}"
                );
            }
        }
        if case["name"] == "property_constructor" || case["name"] == "string_concat" {
            use mpk_vc::csharp_practical_vir_validation as v;
            assert!(!first.vir().source_obligations().is_empty());
            assert_eq!(
                first.vir().source_obligations(),
                actual.source_obligations()
            );
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: first.closure().roots().canonical_json(),
                closed_instances_transport: first.closure().closed().canonical_json(),
                semantic_bindings_transport: first.closure().bindings().canonical_bytes(),
                required_checks_transport: first.operations().required_checks().canonical_bytes(),
                operations_transport: first.operations().operations().canonical_bytes(),
            };
            for mutation in 0..7 {
                let mut obligations = first.vir().source_obligations().to_vec();
                match mutation {
                    0 => {
                        obligations.clear();
                    }
                    1 => obligations[0].discharged = true,
                    2 => obligations[0].kind = "proved".into(),
                    3 => obligations[0].type_id = ty("i64"),
                    4 => obligations[0].start_byte += 1,
                    5 => {
                        let duplicate = obligations[0].clone();
                        obligations.push(duplicate);
                    }
                    6 => {}
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        functions: first.vir().functions().to_vec(),
                        source_obligations: obligations,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_ok(),
                    mutation == 6,
                    "obligation mutation {mutation}"
                );
            }
        }
        if case["name"] == "array_dynamic_read" {
            use mpk_vc::csharp_practical_vir_validation as v;
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: first.closure().roots().canonical_json(),
                closed_instances_transport: first.closure().closed().canonical_json(),
                semantic_bindings_transport: first.closure().bindings().canonical_bytes(),
                required_checks_transport: first.operations().required_checks().canonical_bytes(),
                operations_transport: first.operations().operations().canonical_bytes(),
            };
            for mutation in 0..6 {
                let mut functions = first.vir().functions().to_vec();
                let function = functions
                    .iter_mut()
                    .find(|f| f.id == facts["selected_root_ids"][0])
                    .unwrap();
                let allocation = function
                    .blocks
                    .iter()
                    .filter_map(|b| b.invocation.as_ref())
                    .find(|i| i.operation_id.ends_with(".allocate"))
                    .unwrap()
                    .result
                    .clone();
                match mutation {
                    0 => function
                        .blocks
                        .iter_mut()
                        .find(|b| b.node.tag == ControlNodeTag::Exit)
                        .unwrap()
                        .construction_actions
                        .clear(),
                    1 => function
                        .blocks
                        .iter_mut()
                        .find(|b| {
                            b.node.tag == ControlNodeTag::Operation
                                && !b.construction_actions.is_empty()
                        })
                        .unwrap()
                        .construction_actions
                        .clear(),
                    2 => function.result_type_ids = vec![allocation.type_id],
                    3 => {
                        let exit = function
                            .blocks
                            .iter_mut()
                            .find(|b| b.node.tag == ControlNodeTag::Exit)
                            .unwrap();
                        if let v::PracticalConstructionAction::Discard { actor_id, .. } =
                            &mut exit.construction_actions[0]
                        {
                            *actor_id = "other.function".into();
                        }
                    }
                    4 => {
                        let exit = function
                            .blocks
                            .iter_mut()
                            .find(|b| b.node.tag == ControlNodeTag::Exit)
                            .unwrap();
                        let duplicate = exit.construction_actions[0].clone();
                        exit.construction_actions.push(duplicate);
                    }
                    5 => {}
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        functions,
                        source_obligations: first.vir().source_obligations().to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_ok(),
                    mutation == 5,
                    "symbolic ownership mutation {mutation}"
                );
            }
        }
        if case["name"] == "array_initializer" {
            use mpk_vc::csharp_practical_vir_validation as v;
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: first.closure().roots().canonical_json(),
                closed_instances_transport: first.closure().closed().canonical_json(),
                semantic_bindings_transport: first.closure().bindings().canonical_bytes(),
                required_checks_transport: first.operations().required_checks().canonical_bytes(),
                operations_transport: first.operations().operations().canonical_bytes(),
            };
            for mutation in 0..4 {
                let mut functions = first.vir().functions().to_vec();
                let function = functions
                    .iter_mut()
                    .find(|f| f.id == facts["selected_root_ids"][0])
                    .unwrap();
                let allocation = function
                    .blocks
                    .iter()
                    .filter_map(|b| b.invocation.as_ref())
                    .find(|i| i.operation_id.ends_with(".allocate"))
                    .unwrap()
                    .result
                    .clone();
                let fills = function
                    .blocks
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| {
                        b.invocation
                            .as_ref()
                            .is_some_and(|i| i.operation_id.ends_with(".fill"))
                    })
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                assert_eq!(fills.len(), 2);
                match mutation {
                    0 => {
                        function.blocks[fills[0]].invocation = None;
                    }
                    1 => {
                        function.blocks[fills[1]]
                            .invocation
                            .as_mut()
                            .unwrap()
                            .operands[0] = allocation;
                    }
                    2 => {
                        function.blocks[fills[1]]
                            .invocation
                            .as_mut()
                            .unwrap()
                            .result
                            .type_id = ty("i32");
                    }
                    3 => {
                        function
                            .blocks
                            .iter_mut()
                            .find(|b| b.node.tag == ControlNodeTag::Exit)
                            .unwrap()
                            .construction_actions
                            .clear();
                    }
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    v::PracticalVirContents {
                        source_obligations: first.vir().source_obligations().to_vec(),
                        functions,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                    "array ownership mutation {mutation}"
                );
            }
        }
        if case["name"] == "division" {
            use mpk_vc::csharp_practical_vir_validation::{self as v, PracticalVirContents};
            let input = v::PracticalVirImportContext {
                data_source_facts: Some(actual.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: first.closure().roots().canonical_json(),
                closed_instances_transport: first.closure().closed().canonical_json(),
                semantic_bindings_transport: first.closure().bindings().canonical_bytes(),
                required_checks_transport: first.operations().required_checks().canonical_bytes(),
                operations_transport: first.operations().operations().canonical_bytes(),
            };
            for mutation in 0..4 {
                let mut functions = first.vir().functions().to_vec();
                let block = functions
                    .iter_mut()
                    .flat_map(|f| &mut f.blocks)
                    .find(|b| b.exception_values.len() == 2)
                    .unwrap();
                match mutation {
                    0 => {
                        block.exception_values.pop();
                    }
                    1 => {
                        block.exception_values.swap(0, 1);
                    }
                    2 => {
                        if let MonomorphicValue::ClosedException { tag, .. } =
                            &mut block.exception_values[0].value
                        {
                            *tag = 8;
                        }
                    }
                    3 => {
                        block.exception_values[0].check_id = "exception.range".into();
                    }
                    _ => unreachable!(),
                }
                let bytes = v::canonical_csharp_practical_vir_transport(
                    input,
                    PracticalVirContents {
                        source_obligations: first.vir().source_obligations().to_vec(),
                        functions,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(
                    v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                    "exception mutation {mutation}"
                );
            }
        }
        assert!(!first
            .vir()
            .functions()
            .iter()
            .flat_map(|f| &f.blocks)
            .any(|b| !b.ownership_out.is_empty()));
    }
}

#[test]
fn csharp_03_t03_w14_attaches_captured_method_contracts_before_artifacts() {
    use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
    use mpk_vc::hash_domain_separated_raw;
    let b = bundle();
    let source=b"namespace Data;public readonly struct Value{public readonly int Amount;public Value(int amount){Amount=amount;}}public static class Entry{public static int Run(Value value){return new Value(value.Amount).Amount;}}\n";
    let facts: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-source.json",
    ))
    .unwrap();
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    for mutation in 0..8 {
        let (context, captures) = data_context::context_with_sidecar(&b, root, source, |context| {
            let expression = match mutation {
                7 => J::object(vec![
                    ("tag", J::string("tagged_is")),
                    ("type_id", J::string(ty("bool"))),
                    (
                        "value",
                        J::object(vec![
                            ("tag", J::string("codec_parse")),
                            (
                                "type_id",
                                J::string(
                                    csharp_practical_closed_instance_id(
                                        &b,
                                        &instance(
                                            "result",
                                            vec![primitive("i32"), primitive("parse_error")],
                                        ),
                                    )
                                    .unwrap(),
                                ),
                            ),
                            ("codec_id", J::string("integer.i32")),
                            (
                                "codec_parameters",
                                J::object(vec![("scale", J::Null), ("rounding", J::Null)]),
                            ),
                            (
                                "text",
                                J::object(vec![
                                    ("tag", J::string("literal")),
                                    ("type_id", J::string(ty("string"))),
                                    ("value", J::string("42")),
                                ]),
                            ),
                        ]),
                    ),
                    ("arm", J::string("ok")),
                ]),
                2 => J::object(vec![
                    ("tag", J::string("variable")),
                    ("type_id", J::string("mpk.csharp.value.bool.v1")),
                    ("binding_id", J::string("unknown")),
                ]),
                3 => J::object(vec![
                    ("tag", J::string("result")),
                    ("type_id", J::string("mpk.csharp.value.i32.v1")),
                ]),
                _ => J::object(vec![
                    ("tag", J::string("literal")),
                    ("type_id", J::string("mpk.csharp.value.bool.v1")),
                    ("value", J::Bool(true)),
                ]),
            };
            let mut fields = vec![
                ("schema", J::string(a::METHOD_CONTRACT_SCHEMA)),
                ("semantic_context", context.semantic_context().clone()),
                ("compilation_id", J::string(context.compilation_id())),
                (
                    "callable_id",
                    J::string(if mutation == 1 {
                        "mpk.csharp.source.ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    } else {
                        root
                    }),
                ),
                (
                    "source_content_sha256",
                    J::string(if mutation == 4 {
                        "0".repeat(64)
                    } else {
                        facts["sources"][0]["raw_sha256"].as_str().unwrap().into()
                    }),
                ),
                ("termination", J::string("total")),
                ("requires", J::Array(vec![expression])),
                ("ensures", J::Array(vec![])),
                ("exceptional_cases", J::Array(vec![])),
                ("modifies", J::Array(vec![])),
                (
                    "loops",
                    J::Array(if mutation == 5 {
                        vec![J::object(vec![])]
                    } else {
                        vec![]
                    }),
                ),
            ];
            let digest = hash_domain_separated_raw(
                a::METHOD_CONTRACT_HASH_DOMAIN,
                &a::canonical_practical_json_bytes(&J::object(fields.clone())).unwrap(),
            )
            .unwrap()
            .to_hex();
            fields.push((
                "contract_sha256",
                J::string(if mutation == 6 {
                    "0".repeat(64)
                } else {
                    digest
                }),
            ));
            a::canonical_practical_json_bytes(&J::object(fields)).unwrap()
        });
        let regenerated = source_facts_for_captures(&captures, &facts).unwrap();
        let captured = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&regenerated).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &captured);
        if matches!(mutation, 0 | 7) {
            let emitted = emitted.unwrap();
            assert_eq!(emitted.vir().data_contracts().len(), 1);
            assert!(emitted.vir().data_contracts()[0].contains("\"requires\""));
            use mpk_vc::csharp_practical_vc_model::{
                generate_csharp_practical_vc, PracticalVcSource,
            };
            use mpk_vc::csharp_practical_vir_validation as v;
            let vc = generate_csharp_practical_vc(PracticalVcSource {
                artifact_context: &context,
                captured_inputs: &captures,
                vir: emitted.vir(),
            })
            .unwrap();
            assert!(vc
                .obligation_groups()
                .iter()
                .any(|g| g.function_id() == Some(root)
                    && g.proof_owner()
                        == mpk_vc::csharp_practical_vc_model::LaterProofOwner::DataAndCollections
                    && g.subject_ids()
                        .iter()
                        .any(|id| id.starts_with("data_contract:"))));
            let import = v::PracticalVirImportContext {
                data_source_facts: Some(captured.captured_facts()),
                artifact_context: &context,
                captured_inputs: &captures,
                foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                foundation_definitions_transport: registered_foundation_definitions_transport(),
                closed_roots_transport: emitted.closure().roots().canonical_json(),
                closed_instances_transport: emitted.closure().closed().canonical_json(),
                semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
                required_checks_transport: emitted.operations().required_checks().canonical_bytes(),
                operations_transport: emitted.operations().operations().canonical_bytes(),
            };
            let dropped = v::canonical_csharp_practical_vir_transport(
                import,
                v::PracticalVirContents {
                    functions: emitted.vir().functions().to_vec(),
                    source_obligations: emitted.vir().source_obligations().to_vec(),
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(v::import_csharp_practical_vir_json(&dropped, import).is_err());
            let no_source = v::PracticalVirImportContext {
                data_source_facts: None,
                ..import
            };
            assert!(v::import_csharp_practical_vir_json(
                emitted.vir().canonical_bytes(),
                no_source
            )
            .is_err());
            assert_eq!(
                emitted
                    .manifest()
                    .value()
                    .get("method_contracts")
                    .and_then(J::as_array)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert!(emitted.is_err(), "mutation {mutation}");
        }
    }
}

#[test]
fn csharp_03_t03_w14_captured_instant_binding_projects_original_source() {
    use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
    let b = bundle();
    let cases: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-source-cases.json",
    ))
    .unwrap();
    let case = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "instant_carrier")
        .unwrap();
    let source = case["source_utf8"].as_str().unwrap().as_bytes();
    let facts = &case["facts"];
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    let (plain, inputs) = data_context::context(&b, root, source);
    let captured = ValidatedDataSource::import_captured_facts(
        &b,
        &plain,
        &inputs,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let ty = captured
        .source_types()
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    for mutation in 0..5 {
        let producer_rejected = std::cell::Cell::new(false);
        let (context, captures) = data_context::context_with_sidecar(&b, root, source, |context| {
            let preliminary = a::capture_original_inputs(
                context,
                vec![
                    a::OriginalInput {
                        kind: a::OriginalInputKind::Source,
                        path: "src/Entry.cs".into(),
                        bytes: source.to_vec(),
                    },
                    a::OriginalInput {
                        kind: a::OriginalInputKind::Sidecar,
                        path: "contracts/data.json".into(),
                        bytes: vec![],
                    },
                ],
            )
            .unwrap();
            let binding = a::SemanticBindingInput {
                enum_arms: Default::default(),
                source_type_id: ty["id"].as_str().unwrap().into(),
                source_content_sha256: ty["source_sha256"].as_str().unwrap().into(),
                role: "instant".into(),
                member_map: vec![a::SemanticBindingMember {
                    role: "milliseconds".into(),
                    member_id: ty["members"][0]["id"].as_str().unwrap().into(),
                }],
                tag_arms: vec![],
                inferred_argument_ids: vec![],
                default_arm: "ineligible".into(),
                bounds: vec![],
                operation_map: vec![],
            };
            let artifact =
                a::build_semantic_bindings(context, &preliminary, vec![binding]).unwrap();
            if mutation == 0 {
                return artifact.canonical_bytes().to_vec();
            }
            // Rehash through the producer wherever possible: attachment must
            // reject wrong source facts, not only a stale outer hash.
            let mut input = DataSidecars::capture(
                context,
                &a::capture_original_inputs(
                    context,
                    vec![
                        a::OriginalInput {
                            kind: a::OriginalInputKind::Source,
                            path: "src/Entry.cs".into(),
                            bytes: source.to_vec(),
                        },
                        a::OriginalInput {
                            kind: a::OriginalInputKind::Sidecar,
                            path: "contracts/data.json".into(),
                            bytes: artifact.canonical_bytes().to_vec(),
                        },
                    ],
                )
                .unwrap(),
            )
            .unwrap()
            .bindings()[0]
                .clone();
            match mutation {
                1 => input.source_type_id = format!("mpk.csharp.source.{}", "f".repeat(64)),
                2 => {
                    input.member_map[0].member_id = format!("mpk.csharp.member.{}", "f".repeat(64))
                }
                3 => input.inferred_argument_ids = vec!["mpk.csharp.value.i32.v1".into()],
                4 => {
                    input.operation_map = vec![a::SemanticOperationMapping {
                        operation: "milliseconds".into(),
                        member_id: format!("mpk.csharp.source.{}", "f".repeat(64)),
                    }]
                }
                _ => unreachable!(),
            }
            match a::build_semantic_bindings(context, &preliminary, vec![input]) {
                Ok(artifact) => artifact.canonical_bytes().to_vec(),
                Err(_) => {
                    producer_rejected.set(true);
                    artifact.canonical_bytes().to_vec()
                }
            }
        });
        if producer_rejected.get() {
            assert_eq!(mutation, 3);
            continue;
        }
        let regenerated = match source_facts_for_captures(&captures, facts) {
            Ok(v) => v,
            Err(_) => {
                assert!(mutation > 0);
                continue;
            }
        };
        let captured = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&regenerated).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &captured);
        if mutation == 0 {
            let emitted = emitted.unwrap();
            assert_eq!(emitted.vir().binding_projections().len(), 1);
            assert_eq!(emitted.vir().functions().len(), 2);
            assert!(!emitted.closure().obligations().is_empty());
            assert!(emitted
                .closure()
                .obligations()
                .iter()
                .all(|o| !o.discharged));
            assert_eq!(
                emitted
                    .closure()
                    .bindings()
                    .value()
                    .get("bindings")
                    .and_then(J::as_array)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert!(emitted.is_err(), "binding mutation {mutation}");
        }
    }
}

#[test]
fn csharp_03_t03_w14_pinned_frontend_to_importer_twice_when_available() {
    if !cfg!(target_os = "linux") {
        return;
    }
    let package = file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    let cache=root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            cache
                .join(format!(
                    "{}.{}",
                    archive["id"].as_str().unwrap(),
                    archive["kind"].as_str().unwrap()
                ))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned cache"
    );
    if count == 0 {
        return;
    }
    let expected = read("develop/migrations/csharp-03/data-phase/data-source-cases.json");
    for _ in 0..2 {
        let output =
            std::process::Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
                .arg("--test-data-phase-cases")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .output()
                .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected, "captured source evidence is stale");
        let mut replay =
            std::process::Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
                .arg("--test-data-phase-requests")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
        {
            use std::io::Write;
            replay
                .stdin
                .take()
                .unwrap()
                .write_all(&read(
                    "develop/migrations/csharp-03/data-phase/data-sidecar-requests.json",
                ))
                .unwrap();
        }
        let replay = replay.wait_with_output().unwrap();
        assert!(
            replay.status.success(),
            "{}",
            String::from_utf8_lossy(&replay.stderr)
        );
        assert!(replay.stderr.is_empty());
        assert_eq!(
            replay.stdout,
            read("develop/migrations/csharp-03/data-phase/data-sidecar-responses.json"),
            "actual sidecar source evidence is stale"
        );
        import_source_cases(&output.stdout);
        let stages =
            std::process::Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
                .arg("--test-data-phase-replay")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .output()
                .unwrap();
        assert!(
            stages.status.success(),
            "{}",
            String::from_utf8_lossy(&stages.stderr)
        );
        assert!(stages.stderr.is_empty());
        assert_eq!(
            stages.stdout,
            read("develop/migrations/csharp-03/data-phase/data-stage-replay.json")
        );
        import_stage_source_cases(&stages.stdout);
    }
}

#[test]
fn csharp_03_t03_w14_attaches_type_contract_member_and_default_facts() {
    use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
    let b = bundle();
    let cases: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-source-cases.json",
    ))
    .unwrap();
    let case = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "identity")
        .unwrap();
    let source = case["source_utf8"].as_str().unwrap().as_bytes();
    let facts = &case["facts"];
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    let (plain, inputs) = data_context::context(&b, root, source);
    let captured = ValidatedDataSource::import_captured_facts(
        &b,
        &plain,
        &inputs,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let ty = captured
        .source_types()
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    for mutation in 0..5 {
        let (context, captures) = data_context::context_with_sidecar(&b, root, source, |context| {
            let mut fields = vec![
                ("schema", J::string(a::TYPE_CONTRACT_SCHEMA)),
                ("semantic_context", context.semantic_context().clone()),
                ("compilation_id", J::string(context.compilation_id())),
                ("source_type_id", J::string(ty["id"].as_str().unwrap())),
                (
                    "source_content_sha256",
                    J::string(ty["source_sha256"].as_str().unwrap()),
                ),
                (
                    "ordered_member_ids",
                    J::Array(if mutation == 1 {
                        vec![]
                    } else {
                        vec![J::string(ty["members"][0]["id"].as_str().unwrap())]
                    }),
                ),
                (
                    "recursive_default",
                    J::object(vec![(
                        "Amount",
                        J::string(if mutation == 2 { "1" } else { "0" }),
                    )]),
                ),
                ("default_eligible", J::Bool(mutation != 3)),
                ("required_member_ids", J::Array(vec![])),
                ("init_member_ids", J::Array(vec![])),
                ("construction_invariant", J::Null),
                ("invariants", J::Array(vec![])),
                ("structural_equality", J::string("field_complete")),
                (
                    "structural_order",
                    J::string(if mutation == 4 {
                        "ineligible"
                    } else {
                        "canonical_field_order"
                    }),
                ),
            ];
            let hash = mpk_vc::hash_domain_separated_raw(
                a::TYPE_CONTRACT_HASH_DOMAIN,
                &a::canonical_practical_json_bytes(&J::object(fields.clone())).unwrap(),
            )
            .unwrap()
            .to_hex();
            fields.push(("contract_sha256", J::string(hash)));
            a::canonical_practical_json_bytes(&J::object(fields)).unwrap()
        });
        let regenerated = source_facts_for_captures(&captures, facts).unwrap();
        let captured = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&regenerated).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &captured);
        if mutation == 0 {
            assert_eq!(
                emitted
                    .unwrap()
                    .manifest()
                    .value()
                    .get("type_contracts")
                    .and_then(J::as_array)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert!(emitted.is_err(), "type contract mutation {mutation}");
        }
    }
}

// This fixture supplies explicit role mappings; production never classifies an
// application enum or value by these names. The source bytes are Roslyn-captured.
fn fallible_business_context(
    b: &ValidatedFoundationBundle,
    facts: &Value,
    source: &[u8],
) -> (
    mpk_vc::csharp_practical_source_artifacts::PracticalArtifactContext,
    mpk_vc::csharp_practical_source_artifacts::CapturedInputSet,
) {
    use mpk_vc::csharp_practical_source_artifacts as a;
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    let ty = |name: &str| {
        facts["types"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .unwrap()
    };
    let money = facts["types"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "Money");
    let instant = ty(if money { "Money" } else { "Instant" });
    let semantic = if money {
        csharp_practical_closed_instance_id(b, &instance("money", vec![primitive("string")]))
            .unwrap()
    } else {
        "mpk.csharp.value.instant.v1".into()
    };
    let outcome = ty("Outcome");
    let fault = ty("Fault");
    let member = |t: &Value, role: &str, name: &str| {
        let m = t["members"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == name)
            .unwrap();
        a::SemanticBindingMember {
            role: role.into(),
            member_id: csharp_practical_stored_member_id(
                t["id"].as_str().unwrap(),
                name,
                &m["type"],
                m["storage"].as_str().unwrap(),
            )
            .unwrap(),
        }
    };
    data_context::context_with_sidecar(b, root, source, |context| {
        let preliminary = a::capture_original_inputs(
            context,
            vec![
                a::OriginalInput {
                    kind: a::OriginalInputKind::Source,
                    path: "src/Entry.cs".into(),
                    bytes: source.to_vec(),
                },
                a::OriginalInput {
                    kind: a::OriginalInputKind::Sidecar,
                    path: "contracts/data.json".into(),
                    bytes: vec![],
                },
            ],
        )
        .unwrap();
        let error_values = fault["enum_values"].as_array().unwrap();
        let mut enum_arms = if money {
            BTreeMap::from([(
                ty("Mode")["id"].as_str().unwrap().into(),
                BTreeMap::from([
                    ("ToEven".into(), "10".into()),
                    ("AwayFromZero".into(), "20".into()),
                    ("ToZero".into(), "30".into()),
                    ("ToNegativeInfinity".into(), "40".into()),
                    ("ToPositiveInfinity".into(), "50".into()),
                ]),
            )])
        } else {
            BTreeMap::new()
        };
        let labels = if money {
            vec!["invalid_scale", "invalid_rounding", "decimal_overflow"]
        } else {
            vec!["precision", "range"]
        };
        enum_arms.insert(
            fault["id"].as_str().unwrap().into(),
            labels
                .into_iter()
                .zip(error_values)
                .map(|(label, value)| (label.into(), value.as_str().unwrap().into()))
                .collect(),
        );
        let bindings = vec![
            a::SemanticBindingInput {
                source_type_id: instant["id"].as_str().unwrap().into(),
                source_content_sha256: instant["source_sha256"].as_str().unwrap().into(),
                role: if money { "money" } else { "instant" }.into(),
                member_map: if money {
                    vec![
                        member(instant, "amount", "Amount"),
                        member(instant, "currency", "Currency"),
                    ]
                } else {
                    vec![member(instant, "milliseconds", "Milliseconds")]
                },
                tag_arms: vec![],
                inferred_argument_ids: if money {
                    vec!["mpk.csharp.value.string.v1".into()]
                } else {
                    vec![]
                },
                default_arm: "ineligible".into(),
                bounds: vec![],
                operation_map: vec![a::SemanticOperationMapping {
                    operation: if money { "multiply" } else { "add_duration" }.into(),
                    member_id: root.into(),
                }],
                enum_arms,
            },
            a::SemanticBindingInput {
                source_type_id: outcome["id"].as_str().unwrap().into(),
                source_content_sha256: outcome["source_sha256"].as_str().unwrap().into(),
                role: "result".into(),
                member_map: vec![
                    member(outcome, "tag", "Tag"),
                    member(outcome, "value", "Value"),
                    member(outcome, "error", "Error"),
                ],
                tag_arms: vec![
                    a::SemanticArmMapping {
                        semantic_arm: "ok".into(),
                        source_tag: "0".into(),
                    },
                    a::SemanticArmMapping {
                        semantic_arm: "error".into(),
                        source_tag: "1".into(),
                    },
                ],
                inferred_argument_ids: vec![semantic.clone(), fault["id"].as_str().unwrap().into()],
                default_arm: "ineligible".into(),
                bounds: vec![],
                operation_map: vec![],
                enum_arms: BTreeMap::new(),
            },
        ];
        a::build_semantic_bindings(context, &preliminary, bindings)
            .unwrap()
            .canonical_bytes()
            .to_vec()
    })
}

fn source_facts_for_captures(
    captures: &mpk_vc::csharp_practical_source_artifacts::CapturedInputSet,
    fallback: &Value,
) -> Result<Value, String> {
    use mpk_vc::csharp_practical_source_artifacts::OriginalInputKind;
    if !captures
        .entries()
        .iter()
        .any(|e| e.kind() == OriginalInputKind::Sidecar)
    {
        return Ok(fallback.clone());
    }
    let rows: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-sidecar-responses.json",
    ))
    .unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == captures.snapshot_sha256())
        .unwrap_or_else(|| {
            panic!(
                "regenerate actual sidecar source facts for {}",
                captures.snapshot_sha256()
            )
        });
    row.get("facts")
        .cloned()
        .ok_or_else(|| row["reject"].as_str().unwrap().to_owned())
}

#[test]
fn csharp_03_t03_w14_source_facts_cannot_move_between_sidecar_snapshots() {
    let b = bundle();
    let cases: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-source-cases.json",
    ))
    .unwrap();
    let case = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "fallible_instant_a")
        .unwrap();
    let source = case["source_utf8"].as_str().unwrap().as_bytes();
    let (context, captures) = fallible_business_context(&b, &case["facts"], source);
    assert!(ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&case["facts"]).unwrap()
    )
    .is_err());
    let facts = source_facts_for_captures(&captures, &case["facts"]).unwrap();
    ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    for mutation in 0..4 {
        let mut changed = facts.clone();
        match mutation {
            0 => changed["compilation_id"] = json!("other"),
            1 => {
                changed["input_files"].as_array_mut().unwrap().pop();
            }
            2 => changed["input_files"][0]["raw_sha256"] = json!("0".repeat(64)),
            3 => changed["input_files"].as_array_mut().unwrap().swap(0, 1),
            _ => unreachable!(),
        }
        assert!(
            ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(&changed).unwrap()
            )
            .is_err(),
            "snapshot mutation {mutation}"
        );
    }
}

fn money_carrier_context(
    b: &ValidatedFoundationBundle,
    facts: &Value,
    source: &[u8],
) -> (
    mpk_vc::csharp_practical_source_artifacts::PracticalArtifactContext,
    mpk_vc::csharp_practical_source_artifacts::CapturedInputSet,
) {
    use mpk_vc::csharp_practical_source_artifacts as a;
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    let ty = &facts["types"][0];
    data_context::context_with_sidecar(b, root, source, |context| {
        let preliminary = a::capture_original_inputs(
            context,
            vec![
                a::OriginalInput {
                    kind: a::OriginalInputKind::Source,
                    path: "src/Entry.cs".into(),
                    bytes: source.to_vec(),
                },
                a::OriginalInput {
                    kind: a::OriginalInputKind::Sidecar,
                    path: "contracts/data.json".into(),
                    bytes: vec![],
                },
            ],
        )
        .unwrap();
        let input = a::SemanticBindingInput {
            source_type_id: ty["id"].as_str().unwrap().into(),
            source_content_sha256: ty["source_sha256"].as_str().unwrap().into(),
            role: "money".into(),
            member_map: ty["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| a::SemanticBindingMember {
                    role: match m["name"].as_str().unwrap() {
                        "Amount" => "amount",
                        "Currency" => "currency",
                        _ => panic!("unexpected money fixture member"),
                    }
                    .into(),
                    member_id: csharp_practical_stored_member_id(
                        ty["id"].as_str().unwrap(),
                        m["name"].as_str().unwrap(),
                        &m["type"],
                        m["storage"].as_str().unwrap(),
                    )
                    .unwrap(),
                })
                .collect(),
            tag_arms: vec![],
            inferred_argument_ids: vec!["mpk.csharp.value.string.v1".into()],
            default_arm: "ineligible".into(),
            bounds: vec![],
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        };
        a::build_semantic_bindings(context, &preliminary, vec![input])
            .unwrap()
            .canonical_bytes()
            .to_vec()
    })
}

#[test]
fn csharp_03_t03_w14_concrete_numeric_parameters_use_the_existing_relation() {
    let b = bundle();
    let (r, c, _) = fixture(&b, &[primitive("decimal")], json!({}));
    let input = MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        negative: false,
        scale: 1,
        coefficient: "25".into(),
    };
    for (mode, coefficient) in [
        ("ToEven", "2"),
        ("AwayFromZero", "3"),
        ("ToZero", "2"),
        ("ToNegativeInfinity", "2"),
        ("ToPositiveInfinity", "3"),
    ] {
        let concrete = NumericOperation::new(
            &format!("decimal.round.{mode}.1"),
            &[ty("decimal")],
            &ty("decimal"),
            None,
        )
        .unwrap();
        let original = NumericOperation::new(
            "decimal.round",
            &[ty("decimal")],
            &ty("decimal"),
            Some(mode),
        )
        .unwrap();
        let actual = concrete
            .evaluate(&b, &r, &c, std::slice::from_ref(&input))
            .unwrap();
        assert_eq!(
            actual,
            original
                .evaluate(&b, &r, &c, std::slice::from_ref(&input))
                .unwrap()
        );
        assert_eq!(
            actual,
            MonomorphicValue::DecimalBits {
                type_id: ty("decimal"),
                negative: false,
                scale: 0,
                coefficient: coefficient.into()
            }
        );
    }
    for id in [
        "decimal.round.Unknown.1",
        "decimal.round.ToEven.01",
        "decimal.round.ToEven.2",
        "decimal.round.ToEven.1.extra",
    ] {
        assert!(NumericOperation::new(id, &[ty("decimal")], &ty("decimal"), None).is_err());
    }
    assert!(NumericOperation::new(
        "decimal.round.ToEven.1",
        &[ty("decimal")],
        &ty("decimal"),
        Some("AwayFromZero")
    )
    .is_err());
    assert_eq!(
        evaluate_string_operation(
            "string.equals.instance.ordinal",
            &[
                StringOperand::Text { utf16: None },
                StringOperand::Text { utf16: None }
            ],
            false
        ),
        Err(StringError::NullReceiver)
    );
}

#[test]
fn csharp_03_t03_w14_existing_stage_source_replay() {
    import_stage_source_cases(&read(
        "develop/migrations/csharp-03/data-phase/data-stage-replay.json",
    ));
}
fn import_stage_source_cases(bytes: &[u8]) {
    let b = bundle();
    let rows: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 777);
    let mut ids = std::collections::BTreeSet::new();
    let mut stages = std::collections::BTreeSet::new();
    let mut emitted = 0;
    let mut rejected = 0;
    let resource_rows = BTreeMap::from([
        (
            "7ac772a19b64222055705e01dbd5c533f1d4718c11e4d18cafef25bc0fa1d052",
            33,
        ),
        (
            "d22013dfb75f1d1a83765a35b15f2b1345281757f3333740acd3396d26bdf5be",
            9,
        ),
    ]);
    let mut resource_seen = 0;
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(ids.insert(id));
        stages.extend(
            row["stages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap()),
        );
        let Some(facts) = row["outcome"].get("facts") else {
            assert_eq!(row["outcome"]["artifact_count"], 0);
            assert!(row["outcome"]["reject"]
                .as_str()
                .is_some_and(|s| s.starts_with("CSHARP_PRACTICAL_") || s.starts_with("sidecar/")));
            rejected += 1;
            continue;
        };
        let (context, captures) = data_context::replay_context(&b, row);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap_or_else(|e| panic!("stage source {id}: {e:?}"));
        let result = emit_data_phase(&b, &context, &captures, &source);
        if let Some(count) = resource_rows.get(id) {
            // The exact overflow rows of W08 live_limits/method_limits. The
            // boundary positives (8/32) are emitted by this same complete replay.
            assert_eq!(row["stages"], json!(["W08"]));
            assert_eq!(
                source
                    .callables()
                    .iter()
                    .flat_map(|c| source.body(c.id()).unwrap())
                    .filter(|n| n.kind() == "ArrayCreation")
                    .count(),
                *count
            );
            assert!(
                matches!(
                    result,
                    Err(DataPhaseError::Import {
                        phase: "resource",
                        code: "CSHARP_PRACTICAL_VIR_LIMIT"
                    })
                ),
                "resource source {id}"
            );
            resource_seen += 1;
        } else {
            result.unwrap_or_else(|e| panic!("stage emission {id}: {e:?}"));
            emitted += 1;
        }
    }
    assert_eq!(stages.len(), 13);
    assert_eq!((emitted, rejected, resource_seen), (403, 372, 2));
}

#[test]
fn csharp_03_t03_w14_interpolation_shapes_share_the_existing_relation() {
    let b = bundle();
    let (r, c, ids) = fixture(
        &b,
        &[instance("option", vec![primitive("string")])],
        json!({}),
    );
    for mask in 0..16 {
        let mut shape = String::new();
        let mut arguments = vec![];
        let mut operands = vec![];
        for bit in 0..4 {
            if mask & (1 << bit) == 0 {
                shape.push('s');
                arguments.push(ids[0].clone());
                operands.push(StringOperand::Text {
                    utf16: if bit == 0 { None } else { Some(vec![0xd800]) },
                });
            } else {
                shape.push('c');
                arguments.push(ty("char"));
                operands.push(StringOperand::Char { utf16: 0xdc00 });
            }
        }
        let signature = ClosedOperationSignature {
            id: format!("string.interpolation.restricted.{shape}"),
            tag: ClosedOperationTag::Data,
            argument_type_ids: arguments,
            normal_result_type_id: ty("string"),
            ordered_checks: vec![RequiredCheck {
                id: "obligation.output_bound".into(),
                tag: RequiredCheckTag::StaticObligation,
                failure_type_id: None,
            }],
        };
        validate_closed_operation_signature(&r, &c, &signature).unwrap();
        assert_eq!(
            evaluate_string_operation(&signature.id, &operands, false),
            evaluate_string_operation("string.interpolation.restricted", &operands, false)
        );
        for mutation in 0..3 {
            let mut changed = signature.clone();
            match mutation {
                0 => changed.id.push('s'),
                1 => changed.argument_type_ids[0] = ty("i32"),
                2 => changed.ordered_checks.clear(),
                _ => unreachable!(),
            }
            assert!(validate_closed_operation_signature(&r, &c, &changed).is_err());
        }
    }
    assert!(evaluate_string_operation("string.interpolation.restricted.bad", &[], false).is_err());
    assert!(evaluate_string_operation(
        "string.interpolation.restricted.empty",
        &[StringOperand::Char { utf16: 0 }],
        false
    )
    .is_err());
    assert_eq!(
        evaluate_string_operation("string.interpolation.restricted.empty", &[], false),
        evaluate_string_operation("string.interpolation.restricted", &[], false)
    );
}
