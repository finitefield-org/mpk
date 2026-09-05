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
