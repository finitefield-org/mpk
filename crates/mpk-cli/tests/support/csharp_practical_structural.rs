// CSHARP-03-T03-W06: the owner remains csharp_practical_types.rs.
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeSet;

fn primitive(name: &str) -> String {
    format!("mpk.csharp.value.{name}.v1")
}
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn fixture(b: &ValidatedFoundationBundle) -> (ValidatedClosedRootSet, ClosedInstanceSet) {
    let mut sources = Map::new();
    let routes = super::json("develop/migrations/csharp-03/structural/source-routes.json");
    for projection in routes.as_array().unwrap() {
        let id = projection["id"].as_str().unwrap();
        if !id.starts_with("mpk.csharp.source.") {
            continue;
        }
        let identity = ["Scalars","Product","Containers","Choice","Key"].into_iter().map(|name| json!({
            "kind":"type","namespace":"Business","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""
        })).find(|identity| csharp_practical_declaration_id(identity).unwrap()==id).unwrap();
        let mut defaults = Map::new();
        let members:Vec<_>=projection["members"].as_array().unwrap().iter().enumerate().map(|(ordinal,m)| {
            let storage=if projection["kind"]=="sealed_class" {"init_auto"} else {"readonly_field"};
            let member_id=csharp_practical_stored_member_id(id,m["name"].as_str().unwrap(),&m["type"],storage).unwrap();
            defaults.insert(member_id.clone(),json!(0));
            json!({"id":member_id,"name":m["name"],"type":m["type"],"storage":storage,"ordinal":ordinal,"required":false})
        }).collect();
        sources.insert(id.to_owned(),json!({"id":id,"identity":identity,"kind":projection["kind"],"members":members,
            "enum_values":projection["enum_values"],"enum_underlying":if projection["kind"]=="enum" {projection["carrier"].clone()} else {Value::Null},
            "actual_default":defaults,"public_default":false,"identity_sensitive":false,"source_sha256":"0".repeat(64)}));
    }
    let mut roots:Vec<_>=sources.keys().enumerate().map(|(i,id)|json!({"origin":"contract","provenance_id":format!("source.{i}"),"type":{"kind":"source","id":id}})).collect();
    for (name, args) in [
        ("bounded_sequence", vec!["i32"]),
        ("bounded_sequence", vec!["f32"]),
        ("option", vec!["f32"]),
        ("option", vec!["string"]),
        ("ordered_map", vec!["decimal", "f32"]),
        ("ordered_set", vec!["decimal"]),
        ("ordered_entry", vec!["i32", "f32"]),
        ("result", vec!["i32", "f32"]),
        ("lookup", vec!["i32"]),
        ("validation", vec!["i32", "string"]),
        ("boundary_field", vec!["i32"]),
        ("money", vec!["string"]),
        ("transition", vec!["i32", "i32", "i32"]),
    ] {
        roots.push(json!({"origin":"semantic_binding","provenance_id":format!("semantic.{}",roots.len()),
            "type":{"kind":"instance","template":name,"arguments":args.into_iter().map(|id|json!({"kind":"primitive","id":id})).collect::<Vec<_>>()}}));
    }
    let transport =
        canonical_closed_root_set_transport(b, &json!(roots), &Value::Object(sources)).unwrap();
    let roots = validate_closed_root_set(b, &transport).unwrap();
    let set = derive_closed_instances(b, &roots).unwrap();
    (roots, set)
}
fn instance(b: &ValidatedFoundationBundle, name: &str, args: &[&str]) -> String {
    csharp_practical_closed_instance_id(b,&json!({"kind":"instance","template":name,"arguments":args.iter().map(|id|json!({"kind":"primitive","id":id})).collect::<Vec<_>>() })).unwrap()
}
fn signed(v: i64) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: primitive("i32"),
        value: v.to_string(),
    }
}
fn decimal(negative: bool, scale: u8, n: &str) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: primitive("decimal"),
        negative,
        scale,
        coefficient: n.to_owned(),
    }
}
fn floating(bits: &str) -> MonomorphicValue {
    MonomorphicValue::F32Bits {
        type_id: primitive("f32"),
        bits: bits.to_owned(),
    }
}
fn laws(p: &StructuralProgram<'_>, values: &[MonomorphicValue]) {
    for a in values {
        for b in values {
            assert_eq!(
                p.structural_equal(a, b).unwrap(),
                p.structural_equal(b, a).unwrap()
            );
            let ab = p.canonical_compare(a, b).unwrap();
            assert_eq!(ab, p.canonical_compare(b, a).unwrap().reverse());
            assert_eq!(ab == Ordering::Equal, p.structural_equal(a, b).unwrap());
            for c in values {
                if ab != Ordering::Greater
                    && p.canonical_compare(b, c).unwrap() != Ordering::Greater
                {
                    assert_ne!(p.canonical_compare(a, c).unwrap(), Ordering::Greater);
                }
            }
        }
    }
}
#[test]
fn csharp_03_t03_w06_frozen_matrix_and_actual_source_routing() {
    let b = bundle();
    let (r, s) = fixture(&b);
    let frozen = super::json("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let vectors: Vec<_> = frozen["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W06")
        .collect();
    assert_eq!(vectors.len(), 23);
    for v in vectors {
        assert_eq!(
            v["production_test_owner"],
            "crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W06"
        );
        let id = primitive(v["id"].as_str().unwrap().strip_prefix("ordering.").unwrap());
        let p = generate_structural_program(&b, &r, &s, &id).unwrap();
        assert_eq!(json!(p.is_total()), v["expected"]["value"]);
        let sig = ClosedOperationSignature {
            id: "contract.compare".into(),
            tag: ClosedOperationTag::CanonicalCompare,
            argument_type_ids: vec![id.clone(), id],
            normal_result_type_id: primitive("i32"),
            ordered_checks: vec![],
        };
        assert_eq!(
            validate_closed_operation_signature(&r, &s, &sig).is_ok(),
            p.is_total()
        );
    }
    let mut observed = BTreeSet::new();
    let routes = super::json("develop/migrations/csharp-03/structural/source-routes.json");
    for route in routes.as_array().unwrap() {
        let p = generate_structural_program(&b, &r, &s, route["id"].as_str().unwrap()).unwrap();
        observed.extend(p.recipes().keys().cloned());
        if route["members"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["name"] == "F32" || m["name"] == "Value" || m["name"] == "Optional")
        {
            assert!(!p.is_total(), "nested float must taint products");
        }
    }
    for name in [
        "bool",
        "i8",
        "u8",
        "i16",
        "u16",
        "i32",
        "u32",
        "i64",
        "u64",
        "char",
        "f32",
        "f64",
        "decimal",
        "string",
        "date",
        "time",
        "duration",
        "guid",
        "day_of_week",
    ] {
        assert!(
            observed.contains(&primitive(name)),
            "missing actual source route {name}"
        );
    }
    for entry in s.entries() {
        assert!(
            generate_structural_program(&b, &r, &s, entry["instance_id"].as_str().unwrap()).is_ok()
        );
    }
}
#[test]
fn csharp_03_t03_w06_algebraic_and_runtime_corner_vectors() {
    let b = bundle();
    let (r, s) = fixture(&b);
    let integers: Vec<_> = (-2..=2).map(signed).collect();
    laws(
        &generate_structural_program(&b, &r, &s, &primitive("i32")).unwrap(),
        &integers,
    );
    let decimals = [
        decimal(false, 0, "0"),
        decimal(true, 28, "0"),
        decimal(false, 0, "1"),
        decimal(false, 2, "100"),
        decimal(true, 1, "1"),
        decimal(false, 0, "79228162514264337593543950335"),
        decimal(false, 28, "1"),
    ];
    laws(
        &generate_structural_program(&b, &r, &s, &primitive("decimal")).unwrap(),
        &decimals,
    );
    let fp = generate_structural_program(&b, &r, &s, &primitive("f32")).unwrap();
    for bits in ["7fc00001", "7f800001", "ffc00000"] {
        assert!(!fp
            .structural_equal(&floating(bits), &floating(bits))
            .unwrap());
    }
    assert!(fp
        .structural_equal(&floating("00000000"), &floating("80000000"))
        .unwrap());
    assert!(fp
        .canonical_compare(&floating("00000000"), &floating("00000000"))
        .is_err());
    let double = generate_structural_program(&b, &r, &s, &primitive("f64")).unwrap();
    let nan = MonomorphicValue::F64Bits {
        type_id: primitive("f64"),
        bits: "7ff8000000000001".into(),
    };
    assert!(!double.structural_equal(&nan, &nan).unwrap());
    let strings: Vec<_> = [
        vec![],
        vec![0],
        vec![65],
        vec![65, 0],
        vec![0xd800],
        vec![0xe000],
    ]
    .into_iter()
    .map(|utf16| MonomorphicValue::String {
        type_id: primitive("string"),
        utf16,
    })
    .collect();
    laws(
        &generate_structural_program(&b, &r, &s, &primitive("string")).unwrap(),
        &strings,
    );
    let guids: Vec<_> = [
        "00000001000000000000000000000000",
        "00000100000000000000000000000000",
        "ffffffff000000000000000000000000",
    ]
    .into_iter()
    .map(|n| MonomorphicValue::Guid {
        type_id: primitive("guid"),
        n: n.into(),
    })
    .collect();
    let gp = generate_structural_program(&b, &r, &s, &primitive("guid")).unwrap();
    laws(&gp, &guids);
    assert_eq!(
        gp.canonical_compare(&guids[0], &guids[1]).unwrap(),
        Ordering::Less
    );
    let id = instance(&b, "bounded_sequence", &["i32"]);
    let seq: Vec<_> = [
        vec![],
        vec![signed(0)],
        vec![signed(0), signed(-1)],
        vec![signed(1)],
    ]
    .into_iter()
    .map(|elements| MonomorphicValue::Sequence {
        type_id: id.clone(),
        elements,
    })
    .collect();
    laws(&generate_structural_program(&b, &r, &s, &id).unwrap(), &seq);
    let id = instance(&b, "option", &["string"]);
    let mut optional = vec![MonomorphicValue::Option {
        type_id: id.clone(),
        arm: OptionArm::None,
        value: None,
    }];
    optional.extend(strings.into_iter().map(|v| MonomorphicValue::Option {
        type_id: id.clone(),
        arm: OptionArm::Some,
        value: Some(Box::new(v)),
    }));
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    laws(&p, &optional);
    assert_eq!(
        p.canonical_compare(&optional[0], &optional[1]).unwrap(),
        Ordering::Less
    );
}
#[test]
fn csharp_03_t03_w06_recursive_nan_and_canonical_collection_rejection() {
    let b = bundle();
    let (r, s) = fixture(&b);
    for name in ["option", "bounded_sequence"] {
        let id = instance(&b, name, &["f32"]);
        let absent = if name == "option" {
            MonomorphicValue::Option {
                type_id: id.clone(),
                arm: OptionArm::None,
                value: None,
            }
        } else {
            MonomorphicValue::Sequence {
                type_id: id.clone(),
                elements: vec![],
            }
        };
        let present = if name == "option" {
            MonomorphicValue::Option {
                type_id: id.clone(),
                arm: OptionArm::Some,
                value: Some(Box::new(floating("7fc00001"))),
            }
        } else {
            MonomorphicValue::Sequence {
                type_id: id.clone(),
                elements: vec![floating("7fc00001")],
            }
        };
        let p = generate_structural_program(&b, &r, &s, &id).unwrap();
        assert!(p.structural_equal(&absent, &absent).unwrap());
        assert!(!p.structural_equal(&present, &present).unwrap());
        assert!(p.canonical_compare(&absent, &absent).is_err());
    }
    let id = instance(&b, "ordered_set", &["decimal"]);
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    let duplicate = MonomorphicValue::OrderedSet {
        type_id: id.clone(),
        elements: vec![decimal(false, 0, "1"), decimal(false, 2, "100")],
    };
    assert!(p.structural_equal(&duplicate, &duplicate).is_err());
    let reversed = MonomorphicValue::OrderedSet {
        type_id: id,
        elements: vec![decimal(false, 0, "2"), decimal(false, 0, "1")],
    };
    assert!(p.structural_equal(&reversed, &reversed).is_err());
    let id = instance(&b, "ordered_map", &["decimal", "f32"]);
    let map = MonomorphicValue::OrderedMap {
        type_id: id.clone(),
        entries: vec![MonomorphicMapEntry {
            key: Box::new(decimal(false, 0, "1")),
            value: Box::new(floating("7fc00001")),
        }],
    };
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    assert!(!p.structural_equal(&map, &map).unwrap());
    assert!(p.canonical_compare(&map, &map).is_err());
    assert!(generate_structural_program(&b, &r, &s, "unknown").is_err());
    assert!(generate_structural_program(&b, &r, &s, &primitive("i32"))
        .unwrap()
        .structural_equal(&signed(1), &decimal(false, 0, "1"))
        .is_err());
}

#[test]
fn csharp_03_t03_w06_business_sums_and_products_use_shared_recipes() {
    let b = bundle();
    let (r, s) = fixture(&b);
    let id = instance(&b, "result", &["i32", "f32"]);
    let good = MonomorphicValue::TaggedSum {
        type_id: id.clone(),
        arm: "ok".into(),
        payload: vec![signed(1)],
    };
    let bad = MonomorphicValue::TaggedSum {
        type_id: id.clone(),
        arm: "error".into(),
        payload: vec![floating("7fc00001")],
    };
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    assert!(p.structural_equal(&good, &good).unwrap());
    assert!(!p.structural_equal(&bad, &bad).unwrap());
    assert!(!p.structural_equal(&good, &bad).unwrap());
    assert!(p.canonical_compare(&good, &good).is_err());
    let id = instance(&b, "boundary_field", &["i32"]);
    let values = [
        MonomorphicValue::BoundaryPresence {
            type_id: id.clone(),
            arm: BoundaryArm::Missing,
            value: None,
        },
        MonomorphicValue::BoundaryPresence {
            type_id: id.clone(),
            arm: BoundaryArm::Null,
            value: None,
        },
        MonomorphicValue::BoundaryPresence {
            type_id: id.clone(),
            arm: BoundaryArm::Value,
            value: Some(Box::new(signed(0))),
        },
    ];
    laws(
        &generate_structural_program(&b, &r, &s, &id).unwrap(),
        &values,
    );
    let id = instance(&b, "money", &["string"]);
    let values: Vec<_> = [("EUR", "100"), ("USD", "1"), ("USD", "2")]
        .into_iter()
        .map(|(currency, n)| MonomorphicValue::Money {
            type_id: id.clone(),
            amount: Box::new(decimal(false, 0, n)),
            currency: Box::new(MonomorphicValue::String {
                type_id: primitive("string"),
                utf16: currency.encode_utf16().collect(),
            }),
        })
        .collect();
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    laws(&p, &values);
    assert_eq!(
        p.canonical_compare(&values[0], &values[1]).unwrap(),
        Ordering::Less
    );
    let id = instance(&b, "transition", &["i32", "i32", "i32"]);
    let values: Vec<_> = [(0, vec![], 9), (0, vec![signed(0)], -1), (1, vec![], 0)]
        .into_iter()
        .map(|(state, events, response)| MonomorphicValue::Transition {
            type_id: id.clone(),
            state: Box::new(signed(state)),
            events,
            response: Box::new(signed(response)),
        })
        .collect();
    laws(
        &generate_structural_program(&b, &r, &s, &id).unwrap(),
        &values,
    );
}

#[test]
fn csharp_03_t03_w06_declaration_order_and_same_type_enum_carriers() {
    let b = bundle();
    let (r, s) = fixture(&b);
    let routes = super::json("develop/migrations/csharp-03/structural/source-routes.json");
    let id = routes
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["members"][0]["name"] == "Z")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let values: Vec<_> = [(0, 9), (1, -1), (1, 0)]
        .into_iter()
        .map(|(z, a)| MonomorphicValue::Product {
            type_id: id.clone(),
            fields: vec![
                NamedMonomorphicValue {
                    name: "Z".into(),
                    value: Box::new(signed(z)),
                },
                NamedMonomorphicValue {
                    name: "A".into(),
                    value: Box::new(signed(a)),
                },
            ],
        })
        .collect();
    let p = generate_structural_program(&b, &r, &s, &id).unwrap();
    laws(&p, &values);
    assert_eq!(
        p.canonical_compare(&values[0], &values[1]).unwrap(),
        Ordering::Less
    );
    let mut wrong = values[0].clone();
    if let MonomorphicValue::Product { fields, .. } = &mut wrong {
        fields.reverse();
    }
    assert!(p.structural_equal(&wrong, &wrong).is_err());
    let id = routes
        .as_array()
        .unwrap()
        .iter()
        .find(|t| {
            t["id"].as_str().unwrap().starts_with("mpk.csharp.source.") && t["kind"] == "enum"
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let values: Vec<_> = ["0", "1"]
        .into_iter()
        .map(|carrier| MonomorphicValue::Enum {
            type_id: id.clone(),
            underlying: "i32".into(),
            carrier: carrier.into(),
        })
        .collect();
    laws(
        &generate_structural_program(&b, &r, &s, &id).unwrap(),
        &values,
    );
}
