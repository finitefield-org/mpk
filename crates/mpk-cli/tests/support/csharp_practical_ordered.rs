// CSHARP-03-T03-W09 owner: csharp_practical_collections.rs.
use super::*;
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn primitive(id: &str) -> Value {
    json!({"kind":"primitive","id":id})
}
fn instance(template: &str, args: Vec<Value>) -> Value {
    json!({"kind":"instance","template":template,"arguments":args})
}
fn fixture(
    b: &ValidatedFoundationBundle,
    types: Vec<Value>,
) -> (ValidatedClosedRootSet, ClosedInstanceSet, Vec<String>) {
    let roots:Vec<_>=types.iter().enumerate().map(|(i,t)|json!({"origin":"semantic_binding","provenance_id":format!("binding.{i}"),"type":t})).collect();
    let transport = canonical_closed_root_set_transport(b, &json!(roots), &json!({})).unwrap();
    let r = validate_closed_root_set(b, &transport).unwrap();
    let c = derive_closed_instances(b, &r).unwrap();
    let ids = types
        .iter()
        .map(|t| csharp_practical_closed_instance_id(b, t).unwrap())
        .collect();
    (r, c, ids)
}
fn integer(n: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: "mpk.csharp.value.i32.v1".into(),
        value: n.to_string(),
    }
}
fn string(s: &str) -> MonomorphicValue {
    MonomorphicValue::String {
        type_id: "mpk.csharp.value.string.v1".into(),
        utf16: s.encode_utf16().collect(),
    }
}
fn map(id: &str, entries: Vec<(i32, MonomorphicValue)>) -> MonomorphicValue {
    MonomorphicValue::OrderedMap {
        type_id: id.into(),
        entries: entries
            .into_iter()
            .map(|(k, v)| MonomorphicMapEntry {
                key: Box::new(integer(k)),
                value: Box::new(v),
            })
            .collect(),
    }
}
fn set(id: &str, elements: Vec<MonomorphicValue>) -> MonomorphicValue {
    MonomorphicValue::OrderedSet {
        type_id: id.into(),
        elements,
    }
}
#[test]
fn csharp_03_t03_w09_replays_source_operation_handoff() {
    let b = bundle();
    let (r, c, ids) = fixture(
        &b,
        vec![
            instance("ordered_map", vec![primitive("i32"), primitive("i32")]),
            instance("ordered_set", vec![primitive("i32")]),
        ],
    );
    let source = json_file("develop/migrations/csharp-03/ordered/source-ordered.json");
    for id in ids {
        let model = OrderedCollectionModel::new(&b, &r, &c, &id).unwrap();
        let row = source
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["semantic_type_id"] == id)
            .unwrap();
        let operations:Vec<_>=model.operations().iter().map(|o|json!({"name":o.name,"argument_type_ids":o.argument_type_ids,"result_type_id":o.result_type_id,"ordered_outcomes":o.ordered_outcomes})).collect();
        assert_eq!(json!(operations), row["operations"]);
        model.validate_handoff(model.operations()).unwrap();
        for i in 0..5 {
            let mut candidate = model.operations().to_vec();
            match i {
                0 => candidate[0].id.push('x'),
                1 => candidate[0].name.push('x'),
                2 => candidate[0].argument_type_ids.reverse(),
                3 => candidate[0].result_type_id.push('x'),
                _ => candidate[0].ordered_outcomes.reverse(),
            }
            assert!(model.validate_handoff(&candidate).is_err());
        }
    }
}
#[test]
fn csharp_03_t03_w09_frozen_collection_relations_and_null_lookup() {
    let b = bundle();
    let option = instance("option", vec![primitive("string")]);
    let (r, c, ids) = fixture(
        &b,
        vec![
            instance("ordered_map", vec![primitive("i32"), primitive("string")]),
            instance("ordered_map", vec![primitive("i32"), option.clone()]),
            instance("bounded_sequence", vec![primitive("string")]),
        ],
    );
    let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]).unwrap();
    let initial = map(&ids[0], vec![(1, string("a")), (3, string("b"))]);
    assert_eq!(model.count(&initial), Ok(2));
    assert_eq!(model.contains(&initial, &integer(2)), Ok(false));
    assert_eq!(model.contains(&initial, &integer(3)), Ok(true));
    assert_eq!(
        model
            .update(&initial, integer(3), Some(string("c")), false, usize::MAX)
            .unwrap_err(),
        OrderedCollectionError::DuplicateKey
    );
    assert_eq!(
        model
            .update(&initial, integer(3), Some(string("c")), true, usize::MAX)
            .unwrap(),
        map(&ids[0], vec![(1, string("a")), (3, string("c"))])
    );
    let package = json_file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let mut count = 0;
    for vector in package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W09")
    {
        count += 1;
        assert_eq!(
            vector["production_test_owner"],
            "crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W09"
        );
        let name = vector["id"].as_str().unwrap();
        let result = match name {
            "collections.insert_sorted" => {
                model.update(&initial, integer(2), Some(string("c")), false, 4096)
            }
            "collections.replace_existing" => {
                model.update(&initial, integer(3), Some(string("c")), true, 2)
            }
            "collections.capacity" => {
                model.update(&initial, integer(2), Some(string("c")), false, 2)
            }
            "collections.duplicate_before_capacity" => {
                model.update(&initial, integer(3), Some(string("c")), false, 2)
            }
            "collections.missing_replace" => {
                model.update(&initial, integer(2), Some(string("c")), true, 2)
            }
            "collections.invalid_order" => model.update(
                &map(&ids[0], vec![(3, string("b")), (1, string("a"))]),
                integer(3),
                Some(string("c")),
                false,
                2,
            ),
            "collections.lookup_null_is_found" => {
                let null = MonomorphicValue::Option {
                    type_id: csharp_practical_closed_instance_id(&b, &option).unwrap(),
                    arm: OptionArm::None,
                    value: None,
                };
                let nullable = OrderedCollectionModel::new(&b, &r, &c, &ids[1]).unwrap();
                let stored = map(&ids[1], vec![(1, null.clone())]);
                let found = nullable.lookup(&stored, &integer(1)).unwrap();
                let missing = nullable.lookup(&stored, &integer(2)).unwrap();
                assert!(
                    matches!(found,MonomorphicValue::TaggedSum{ref arm,ref payload,..} if arm=="found" && payload==&vec![null])
                );
                assert!(
                    matches!(missing,MonomorphicValue::TaggedSum{ref arm,ref payload,..} if arm=="missing_key" && payload.is_empty())
                );
                assert_ne!(found, missing);
                assert_eq!(vector["expected"]["value"], true);
                continue;
            }
            "collections.validation_order_duplicates" => {
                // Error lists retain sequence order/duplicates; no set binding
                // may normalize them. Validation combinators remain W14/T04.
                let input = MonomorphicValue::Array {
                    type_id: ids[2].clone(),
                    elements: vec![string("a"), string("b"), string("a")],
                };
                let output = project_bounded_sequence_array(&b, &r, &c, &input).unwrap();
                let MonomorphicValue::Sequence { elements, .. } = output else {
                    panic!()
                };
                assert_eq!(
                    elements,
                    vector["expected"]["value"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| string(s.as_str().unwrap()))
                        .collect::<Vec<_>>()
                );
                continue;
            }
            _ => panic!("uncovered {name}"),
        };
        if let Some(error) = vector["expected"]["reject"].as_str() {
            assert_eq!(result.unwrap_err().as_str(), error);
        } else {
            let entries = vector["expected"]["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    (
                        e[0].as_i64().unwrap() as i32,
                        string(e[1].as_str().unwrap()),
                    )
                })
                .collect();
            assert_eq!(result.unwrap(), map(&ids[0], entries));
        }
    }
    assert_eq!(count, 8);
    assert_eq!(
        initial,
        map(&ids[0], vec![(1, string("a")), (3, string("b"))])
    );
}
#[test]
fn csharp_03_t03_w09_set_projection_order_duplicates_and_limits() {
    let b = bundle();
    let (r, c, ids) = fixture(
        &b,
        vec![
            instance("ordered_set", vec![primitive("i32")]),
            instance("bounded_sequence", vec![primitive("i32")]),
            instance("ordered_map", vec![primitive("i32"), primitive("string")]),
        ],
    );
    let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]).unwrap();
    for keys in [vec![], vec![1], vec![1, 3], vec![3, 1], vec![1, 1]] {
        let array = MonomorphicValue::Array {
            type_id: ids[1].clone(),
            elements: keys.iter().copied().map(integer).collect(),
        };
        let result = model.project(&array, None, None);
        assert_eq!(result.is_ok(), keys.windows(2).all(|p| p[0] < p[1]));
        if let Ok(value) = result {
            assert_eq!(model.count(&value), Ok(keys.len()));
        }
    }
    let value = set(&ids[0], vec![integer(1), integer(3)]);
    assert_eq!(
        model
            .update(&value, integer(1), None, false, 2)
            .unwrap_err(),
        OrderedCollectionError::DuplicateElement
    );
    assert_eq!(
        model.update(&value, integer(2), None, false, 3).unwrap(),
        set(&ids[0], vec![integer(1), integer(2), integer(3)])
    );
    assert!(model.lookup(&value, &integer(1)).is_err());
    assert!(model.update(&value, integer(1), None, true, 2).is_err());
    let full = set(&ids[0], (0..4096).map(integer).collect());
    assert_eq!(
        model
            .update(&full, integer(4096), None, false, usize::MAX)
            .unwrap_err(),
        OrderedCollectionError::Capacity
    );
    assert_eq!(
        model
            .update(&full, integer(0), None, false, usize::MAX)
            .unwrap_err(),
        OrderedCollectionError::DuplicateElement
    );
    let profile = json_file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let mut count = 0;
    let map_model = OrderedCollectionModel::new(&b, &r, &c, &ids[2]).unwrap();
    for vector in profile["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W09")
    {
        count += 1;
        let n = vector["inputs"]["value"].as_u64().unwrap() as usize;
        let accepted = if vector["inputs"]["counter"] == "ordered_map_set_entries" {
            let set_result =
                model.validate(&set(&ids[0], (0..n).map(|i| integer(i as i32)).collect()));
            let map_result = map_model.validate(&map(
                &ids[2],
                (0..n).map(|i| (i as i32, string(""))).collect(),
            ));
            assert_eq!(set_result.is_ok(), map_result.is_ok());
            set_result.is_ok()
        } else {
            // Root + four key/string headers + UTF-16 cells = n.
            let mut remaining = n - 9;
            let entries = (0..4)
                .map(|i| {
                    let length = remaining.min(16384);
                    remaining -= length;
                    (i, string(&"a".repeat(length)))
                })
                .collect();
            map_model.validate(&map(&ids[2], entries)).is_ok()
        };
        assert_eq!(
            accepted,
            vector["expected"]["accept"] == true,
            "{}",
            vector["id"]
        );
    }
    assert_eq!(count, 6);
}
#[test]
fn csharp_03_t03_w09_total_key_matrix_and_conditional_map_comparison() {
    let b = bundle();
    for primitive_key in [
        "unit",
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
        "string",
        "decimal",
        "guid",
        "date",
        "time",
        "duration",
        "instant",
        "day_of_week",
        "parse_error",
        "exception",
        "f32",
        "f64",
    ] {
        let ty = instance("ordered_set", vec![primitive(primitive_key)]);
        if matches!(primitive_key, "f32" | "f64" | "exception") {
            let transport = canonical_closed_root_set_transport(
                &b,
                &json!([{"origin":"semantic_binding","provenance_id":"bad.key","type":ty}]),
                &json!({}),
            )
            .unwrap();
            assert!(validate_closed_root_set(&b, &transport).is_err());
            continue;
        }
        let (r, c, ids) = fixture(&b, vec![ty]);
        let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]);
        assert_eq!(
            model.is_ok(),
            !matches!(primitive_key, "f32" | "f64" | "exception"),
            "{primitive_key}"
        );
    }
    let option = instance("option", vec![primitive("i32")]);
    let (r, c, ids) = fixture(
        &b,
        vec![
            instance("ordered_set", vec![option.clone()]),
            instance("ordered_set", vec![primitive("decimal")]),
            instance("ordered_map", vec![primitive("i32"), primitive("f32")]),
        ],
    );
    let option_id = csharp_practical_closed_instance_id(&b, &option).unwrap();
    let none = MonomorphicValue::Option {
        type_id: option_id.clone(),
        arm: OptionArm::None,
        value: None,
    };
    let some = MonomorphicValue::Option {
        type_id: option_id,
        arm: OptionArm::Some,
        value: Some(Box::new(integer(-1))),
    };
    let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]).unwrap();
    assert!(model
        .validate(&set(&ids[0], vec![none.clone(), some.clone()]))
        .is_ok());
    assert!(model.validate(&set(&ids[0], vec![some, none])).is_err());
    let decimal = |scale, coefficient: &str| MonomorphicValue::DecimalBits {
        type_id: "mpk.csharp.value.decimal.v1".into(),
        negative: false,
        scale,
        coefficient: coefficient.into(),
    };
    let decimal_model = OrderedCollectionModel::new(&b, &r, &c, &ids[1]).unwrap();
    assert!(decimal_model
        .validate(&set(&ids[1], vec![decimal(0, "1"), decimal(1, "10")]))
        .is_err());
    let float_model = OrderedCollectionModel::new(&b, &r, &c, &ids[2]).unwrap();
    assert!(float_model.operations().iter().any(|o| o.name == "equal"));
    assert!(!float_model.operations().iter().any(|o| o.name == "compare"));
    let nan = map(
        &ids[2],
        vec![(
            1,
            MonomorphicValue::F32Bits {
                type_id: "mpk.csharp.value.f32.v1".into(),
                bits: "7fc00000".into(),
            },
        )],
    );
    float_model.validate(&nan).unwrap();
    let p = generate_structural_program(&b, &r, &c, &ids[2]).unwrap();
    assert!(!p.structural_equal(&nan, &nan).unwrap());
    assert!(p.canonical_compare(&nan, &nan).is_err());
}

#[test]
fn csharp_03_t03_w09_exact_private_inputs_and_loop_routing() {
    let path = "develop/migrations/csharp-03/ordered/ordered-inputs.json";
    let manifest = json_file(path);
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W09");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w09.ordered_inputs.v1"
    );
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_ordered_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalOrderedCollections.cs",
        "csharp-tools/csharp2vir/PracticalSequences.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/ordered/source-ordered.json",
    ];
    let records = manifest["files"].as_array().unwrap();
    assert_eq!(records.len(), expected.len());
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(canonical, read(path));
    for (record, path) in records.iter().zip(expected) {
        let bytes = read(path);
        assert_eq!(record["path"], path);
        assert_eq!(record["size_bytes"], bytes.len());
        assert_eq!(record["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    let source = String::from_utf8(read(
        "csharp-tools/csharp2vir/PracticalOrderedCollections.cs",
    ))
    .unwrap();
    assert!(source.contains(ORDERED_COLLECTION_LOOP_OWNER));
    let package = json_file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let loops: Vec<_> = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["family"] == "loops")
        .collect();
    assert!(!loops.is_empty());
    for vector in loops {
        assert!(vector["implementation_owner"]
            .as_str()
            .unwrap()
            .starts_with("CSHARP-03-T04-"));
        assert!(vector["production_test_owner"]
            .as_str()
            .unwrap()
            .contains("csharp_practical_control.rs"));
    }
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!String::from_utf8(read(path))
            .unwrap()
            .contains("PracticalOrderedCollections"));
    }
}

#[test]
fn csharp_03_t03_w09_pinned_source_harness_when_available() {
    let package = json_file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
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
    let output = Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
        .arg("--test-ordered")
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
fn csharp_03_t03_w09_entry_projection_is_bound_to_source_members_and_hash() {
    let b = bundle();
    let identity = json!({"kind":"type","namespace":"Business","owner":"","name":"Pair","parameter_type_ids":[],"result_type_id":""});
    let id = csharp_practical_declaration_id(&identity).unwrap();
    let member = |name| {
        csharp_practical_stored_member_id(&id, name, &primitive("i32"), "readonly_field").unwrap()
    };
    let key = member("Key");
    let value = member("Value");
    let extra = member("Extra");
    let source = json!({"id":id,"identity":identity,"kind":"readonly_struct","members":[
        {"id":key,"name":"Key","type":primitive("i32"),"storage":"readonly_field","ordinal":0,"required":false},
        {"id":value,"name":"Value","type":primitive("i32"),"storage":"readonly_field","ordinal":1,"required":false},
        {"id":extra,"name":"Extra","type":primitive("i32"),"storage":"readonly_field","ordinal":2,"required":false}
    ],"enum_values":[],"enum_underlying":null,"actual_default":{key.clone():0,value.clone():0,extra:0},"public_default":true,"identity_sensitive":false,"source_sha256":"a".repeat(64)});
    let sequence = instance("bounded_sequence", vec![json!({"kind":"source","id":id})]);
    let box_identity = json!({"kind":"type","namespace":"Business","owner":"","name":"Box","parameter_type_ids":[],"result_type_id":""});
    let box_id = csharp_practical_declaration_id(&box_identity).unwrap();
    let items =
        csharp_practical_stored_member_id(&box_id, "Items", &sequence, "readonly_field").unwrap();
    let wrapper_source = json!({"id":box_id,"identity":box_identity,"kind":"sealed_class","members":[
        {"id":items,"name":"Items","type":sequence,"storage":"readonly_field","ordinal":0,"required":false}
    ],"enum_values":[],"enum_underlying":null,"actual_default":{items.clone():null},"public_default":false,"identity_sensitive":false,"source_sha256":"b".repeat(64)});
    let map_ty = instance("ordered_map", vec![primitive("i32"), primitive("i32")]);
    let set_ty = instance("ordered_set", vec![json!({"kind":"source","id":id})]);
    let transport = canonical_closed_root_set_transport(
        &b,
        &json!([
            {"origin":"semantic_binding","provenance_id":"entries","type":sequence},
            {"origin":"semantic_binding","provenance_id":"map","type":map_ty},
            {"origin":"semantic_binding","provenance_id":"set","type":set_ty},
            {"origin":"semantic_binding","provenance_id":"wrapper","type":{"kind":"source","id":box_id}}
        ]),
        &json!({id.clone():source,box_id.clone():wrapper_source}),
    )
    .unwrap();
    let r = validate_closed_root_set(&b, &transport).unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    let map_id = csharp_practical_closed_instance_id(&b, &map_ty).unwrap();
    let model = OrderedCollectionModel::new(&b, &r, &c, &map_id).unwrap();
    let binding = OrderedEntryBinding {
        source_type_id: id.clone(),
        source_content_sha256: "a".repeat(64),
        key_member_id: key,
        value_member_id: value,
    };
    let product = |k| MonomorphicValue::Product {
        type_id: id.clone(),
        fields: vec![
            NamedMonomorphicValue {
                name: "Key".into(),
                value: Box::new(integer(k)),
            },
            NamedMonomorphicValue {
                name: "Value".into(),
                value: Box::new(integer(10 * k)),
            },
            NamedMonomorphicValue {
                name: "Extra".into(),
                value: Box::new(integer(9)),
            },
        ],
    };
    let array = |keys: Vec<i32>| MonomorphicValue::Array {
        type_id: csharp_practical_closed_instance_id(&b, &sequence).unwrap(),
        elements: keys.into_iter().map(product).collect(),
    };
    assert_eq!(
        model
            .project(&array(vec![1, 3]), None, Some(&binding))
            .unwrap(),
        map(&map_id, vec![(1, integer(10)), (3, integer(30))])
    );
    let wrapper_binding = SequenceWrapperBinding {
        source_type_id: box_id.clone(),
        source_content_sha256: "b".repeat(64),
        elements_member_id: items,
        sequence_type_id: csharp_practical_closed_instance_id(&b, &sequence).unwrap(),
    };
    let wrapped = MonomorphicValue::Product {
        type_id: box_id,
        fields: vec![NamedMonomorphicValue {
            name: "Items".into(),
            value: Box::new(array(vec![1, 3])),
        }],
    };
    assert_eq!(
        model
            .project(&wrapped, Some(&wrapper_binding), Some(&binding))
            .unwrap(),
        map(&map_id, vec![(1, integer(10)), (3, integer(30))])
    );
    for mutation in 0..4 {
        let mut bad = wrapper_binding.clone();
        match mutation {
            0 => bad.source_type_id.push('x'),
            1 => bad.source_content_sha256 = "c".repeat(64),
            2 => bad.elements_member_id.push('x'),
            _ => bad.sequence_type_id.push('x'),
        };
        assert!(model.project(&wrapped, Some(&bad), Some(&binding)).is_err());
    }
    for keys in [vec![], vec![1, 3]] {
        for mutation in 0..4 {
            let mut bad = binding.clone();
            match mutation {
                0 => bad.source_type_id.push('x'),
                1 => bad.source_content_sha256 = "b".repeat(64),
                2 => bad.key_member_id = bad.value_member_id.clone(),
                _ => bad.value_member_id.push('x'),
            };
            assert!(model
                .project(&array(keys.clone()), None, Some(&bad))
                .is_err());
        }
        assert!(model.project(&array(keys), None, None).is_err());
    }
    for keys in [vec![3, 1], vec![1, 1]] {
        assert_eq!(
            model
                .project(&array(keys), None, Some(&binding))
                .unwrap_err(),
            OrderedCollectionError::InvalidRepresentation
        );
    }
    let set_id = csharp_practical_closed_instance_id(&b, &set_ty).unwrap();
    let set_model = OrderedCollectionModel::new(&b, &r, &c, &set_id).unwrap();
    assert!(set_model.project(&array(vec![1, 3]), None, None).is_ok());
    assert!(set_model
        .project(&array(vec![1, 3]), None, Some(&binding))
        .is_err());
}

#[test]
fn csharp_03_t03_w09_canonical_scalar_keys_and_lexicographic_sequence_keys() {
    let b = bundle();
    for name in [
        "unit",
        "bool",
        "i8",
        "i16",
        "i32",
        "i64",
        "u8",
        "u16",
        "u32",
        "u64",
        "char",
        "string",
        "decimal",
        "guid",
        "date",
        "time",
        "duration",
        "instant",
        "day_of_week",
        "parse_error",
    ] {
        let (r, c, ids) = fixture(&b, vec![instance("ordered_set", vec![primitive(name)])]);
        let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]).unwrap();
        let key = |second: bool| {
            let mut v = match name {
                "unit" => json!({"kind":"unit"}),
                "bool" => json!({"kind":"bool","value":second}),
                "i8" | "i16" | "i32" | "i64" => {
                    json!({"kind":"signed","value":if second{"1"}else{"-1"}})
                }
                "u8" | "u16" | "u32" | "u64" => {
                    json!({"kind":"unsigned","value":if second{"1"}else{"0"}})
                }
                "char" => json!({"kind":"char","utf16":if second{0xe000}else{0xd800}}),
                "string" => {
                    json!({"kind":"string","utf16":if second{vec![0xe000]}else{vec![0xd800,0xdc00]}})
                }
                "decimal" => {
                    json!({"kind":"decimal_bits","negative":!second,"scale":0,"coefficient":"1"})
                }
                "guid" => {
                    json!({"kind":"guid","n":if second{"80000000000000000000000000000000"}else{"7fffffff000000000000000000000000"}})
                }
                "date" => json!({"kind":"date","day_number":if second{3652058}else{0}}),
                "time" => json!({"kind":"time","ticks":if second{"863999999999"}else{"0"}}),
                "duration" => json!({"kind":"duration","ticks":if second{"1"}else{"-1"}}),
                "instant" => json!({"kind":"instant","milliseconds":if second{"1"}else{"-1"}}),
                "day_of_week" => {
                    json!({"kind":"enum","underlying":"i32","carrier":if second{"6"}else{"0"}})
                }
                "parse_error" => {
                    json!({"kind":"parse_error","arm":if second{"range"}else{"input_bound"}})
                }
                _ => unreachable!(),
            };
            v["type_id"] = json!(format!("mpk.csharp.value.{name}.v1"));
            serde_json::from_value::<MonomorphicValue>(v).unwrap()
        };
        let a = key(false);
        let z = key(true);
        assert!(
            model.validate(&set(&ids[0], vec![a.clone()])).is_ok(),
            "{name}"
        );
        if name != "unit" {
            assert!(
                model
                    .validate(&set(&ids[0], vec![a.clone(), z.clone()]))
                    .is_ok(),
                "{name}"
            );
        }
        assert!(
            model.validate(&set(&ids[0], vec![z, a.clone()])).is_err(),
            "{name}"
        );
        assert!(
            model.validate(&set(&ids[0], vec![a.clone(), a])).is_err(),
            "{name}"
        );
    }
    let key_ty = instance("bounded_sequence", vec![primitive("i32")]);
    let (r, c, ids) = fixture(&b, vec![instance("ordered_set", vec![key_ty.clone()])]);
    let model = OrderedCollectionModel::new(&b, &r, &c, &ids[0]).unwrap();
    let key = |values: Vec<i32>| MonomorphicValue::Sequence {
        type_id: csharp_practical_closed_instance_id(&b, &key_ty).unwrap(),
        elements: values.into_iter().map(integer).collect(),
    };
    let values = vec![key(vec![]), key(vec![1]), key(vec![1, 9]), key(vec![2])];
    model.validate(&set(&ids[0], values.clone())).unwrap();
    let mut reversed = values;
    reversed.reverse();
    assert!(model.validate(&set(&ids[0], reversed)).is_err());
}
