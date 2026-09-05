// CSHARP-03-T03-W08 owner: csharp_practical_collections.rs.
use super::*;
use std::cmp::Ordering;

fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn descriptor(template: &str) -> Value {
    json!({"kind":"instance","template":template,"arguments":[{"kind":"primitive","id":"i32"}]})
}
fn fixture(
    b: &ValidatedFoundationBundle,
) -> (ValidatedClosedRootSet, ClosedInstanceSet, String, String) {
    let transport=canonical_closed_root_set_transport(b,&json!([
        {"origin":"source_construction","provenance_id":"source.allocate","type":descriptor("sequence_construction")}
    ]), &json!({})).unwrap();
    let roots = validate_closed_root_set(b, &transport).unwrap();
    let closed = derive_closed_instances(b, &roots).unwrap();
    (
        roots,
        closed,
        csharp_practical_closed_instance_id(b, &descriptor("sequence_construction")).unwrap(),
        csharp_practical_closed_instance_id(b, &descriptor("bounded_sequence")).unwrap(),
    )
}
fn integer(n: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: "mpk.csharp.value.i32.v1".into(),
        value: n.to_string(),
    }
}
fn fill(index: i32) -> SequenceConstructionAction {
    SequenceConstructionAction::Fill {
        actor_id: "owner".into(),
        index,
        value_type_id: integer(0).type_id().into(),
    }
}
fn freeze(sequence: &str) -> SequenceConstructionAction {
    SequenceConstructionAction::Freeze {
        actor_id: "owner".into(),
        result_type_id: sequence.into(),
    }
}
#[test]
fn csharp_03_t03_w08_direct_publication_and_shared_operations() {
    let b = bundle();
    let (r, c, construction, sequence) = fixture(&b);
    let structural = generate_structural_program(&b, &r, &c, &sequence).unwrap();
    for size in [0, 1, 2, 4095, 4096] {
        let mut batch = SequenceConstructionBatch::new(&b, &r, &c);
        batch.allocate("a", &construction, "owner", size).unwrap();
        for index in 0..size as i32 {
            batch
                .apply("a", &fill(index), Some(integer(index)))
                .unwrap();
        }
        let result = batch.apply("a", &freeze(&sequence), None).unwrap().unwrap();
        assert_eq!(
            bounded_sequence_length(&b, &r, &c, &result),
            Ok(size as usize)
        );
        assert!(bounded_sequence_read(&b, &r, &c, &result, -1).is_err());
        assert!(bounded_sequence_read(&b, &r, &c, &result, size as i32).is_err());
        if size > 0 {
            assert_eq!(
                bounded_sequence_read(&b, &r, &c, &result, 0).unwrap(),
                &integer(0)
            );
        }
        assert!(structural.structural_equal(&result, &result).unwrap());
        assert_eq!(
            structural.canonical_compare(&result, &result).unwrap(),
            Ordering::Equal
        );
        assert_eq!(batch.finish().unwrap()["a"], result);
    }
    let a = MonomorphicValue::Sequence {
        type_id: sequence.clone(),
        elements: vec![integer(1), integer(9)],
    };
    let z = MonomorphicValue::Sequence {
        type_id: sequence.clone(),
        elements: vec![integer(2)],
    };
    assert_eq!(
        structural.canonical_compare(&a, &z).unwrap(),
        Ordering::Less
    );
    assert!(!structural.structural_equal(&a, &z).unwrap());
    let array = MonomorphicValue::Array {
        type_id: sequence,
        elements: vec![integer(1), integer(9)],
    };
    let projected = project_bounded_sequence_array(&b, &r, &c, &array).unwrap();
    assert!(structural.structural_equal(&a, &projected).unwrap());
    assert_eq!(
        structural.canonical_compare(&a, &projected).unwrap(),
        Ordering::Equal
    );
    assert!(project_bounded_sequence_array(&b, &r, &c, &integer(0)).is_err());
}
#[test]
fn csharp_03_t03_w08_rejects_partial_forged_and_residual_state() {
    let b = bundle();
    let (r, c, construction, sequence) = fixture(&b);
    let mut batch = SequenceConstructionBatch::new(&b, &r, &c);
    batch.allocate("a", &construction, "owner", 2).unwrap();
    let read = SequenceConstructionAction::Read {
        actor_id: "owner".into(),
        index: 0,
        result_type_id: integer(0).type_id().into(),
    };
    assert!(batch.apply("a", &read, None).is_err());
    assert!(batch.apply("a", &freeze(&sequence), None).is_err());
    assert!(batch
        .apply(
            "a",
            &fill(0),
            Some(MonomorphicValue::Signed {
                type_id: integer(0).type_id().into(),
                value: "2147483648".into()
            })
        )
        .is_err());
    assert!(batch.apply("a", &fill(0), None).is_err());
    assert_eq!(batch.state("a").unwrap().version, 0);
    batch.apply("a", &fill(0), Some(integer(7))).unwrap();
    assert_eq!(batch.apply("a", &read, None).unwrap(), Some(integer(7)));
    assert!(batch.apply("a", &fill(0), Some(integer(8))).is_err());
    assert!(batch
        .apply(
            "a",
            &SequenceConstructionAction::Transfer {
                actor_id: "owner".into(),
                new_owner_id: "next".into()
            },
            None
        )
        .is_err());
    batch.apply("a", &fill(1), Some(integer(8))).unwrap();
    let mut altered = batch.state("a").unwrap().clone();
    altered.owner_id = "alias".into();
    assert!(SequenceConstructionState::merge(&c, batch.state("a").unwrap(), &altered).is_err());
    altered = batch.state("a").unwrap().clone();
    altered.version += 1;
    assert!(SequenceConstructionState::merge(&c, batch.state("a").unwrap(), &altered).is_err());
    batch
        .apply(
            "a",
            &SequenceConstructionAction::Transfer {
                actor_id: "owner".into(),
                new_owner_id: "next".into(),
            },
            None,
        )
        .unwrap();
    assert!(batch.apply("a", &freeze(&sequence), None).is_err());
    batch
        .apply(
            "a",
            &SequenceConstructionAction::Freeze {
                actor_id: "next".into(),
                result_type_id: sequence,
            },
            None,
        )
        .unwrap();
    assert!(batch.apply("a", &fill(0), Some(integer(0))).is_err());
    batch
        .allocate("residual", &construction, "owner", 0)
        .unwrap();
    assert!(batch.finish().is_err());
}
#[test]
fn csharp_03_t03_w08_frozen_capacity_and_state_vectors() {
    let b = bundle();
    let (r, c, construction, sequence) = fixture(&b);
    let package = json_file("develop/specs/vectors/csharp-practical-profile-v1.json");
    let vectors: Vec<_> = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W08")
        .collect();
    assert_eq!(vectors.len(), 9);
    for vector in vectors {
        assert_eq!(
            vector["production_test_owner"],
            "crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W08"
        );
        let n = vector["inputs"]["value"].as_i64().unwrap();
        let name = vector["id"].as_str().unwrap();
        let mut batch = SequenceConstructionBatch::new(&b, &r, &c);
        let result = if name.contains("sequence_construction_capacity") {
            batch.allocate("a", &construction, "owner", n)
        } else {
            (0..n).try_for_each(|i| {
                let id = format!("a{i}");
                batch.allocate(&id, &construction, "owner", 0)?;
                if name.contains("construction_states_per_method") {
                    batch.apply(&id, &freeze(&sequence), None)?;
                }
                Ok(())
            })
        };
        assert_eq!(
            result.is_ok(),
            vector["expected"]["accept"] == true,
            "{name}"
        );
    }
    let mut batch = SequenceConstructionBatch::new(&b, &r, &c);
    assert!(batch
        .allocate("negative", &construction, "owner", -1)
        .is_err());
    batch
        .allocate("large", &construction, "owner", 4097)
        .unwrap();
    for i in 0..4097 {
        batch.apply("large", &fill(i), Some(integer(0))).unwrap();
    }
    assert!(batch.apply("large", &freeze(&sequence), None).is_err());
}
#[test]
fn csharp_03_t03_w08_exact_private_inputs_and_loop_routing() {
    let path = "develop/migrations/csharp-03/sequences/sequences-inputs.json";
    let manifest = json_file(path);
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W08");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w08.sequences_inputs.v1"
    );
    let expected = [
        "crates/mpk-cli/tests/csharp_practical_sequences_harness.cs",
        "csharp-tools/csharp2vir/PracticalArrays.cs",
        "csharp-tools/csharp2vir/PracticalCapture.cs",
        "csharp-tools/csharp2vir/PracticalConstruction.cs",
        "csharp-tools/csharp2vir/PracticalDataTypes.cs",
        "csharp-tools/csharp2vir/PracticalSequences.cs",
        "csharp-tools/csharp2vir/PracticalStructural.cs",
        "csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs",
        "develop/migrations/csharp-03/sequences/source-direct.json",
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
    let source = String::from_utf8(read("csharp-tools/csharp2vir/PracticalSequences.cs")).unwrap();
    assert!(source.contains("CSHARP-03-T04-W01/W02"));
    for path in [
        "csharp-tools/csharp2vir/csharp2vir.csproj",
        "csharp-tools/csharp2vir/Program.cs",
        "develop/migrations/csharp-03/build-inputs/build-inputs.json",
    ] {
        assert!(!String::from_utf8(read(path))
            .unwrap()
            .contains("PracticalSequences"));
    }
}

#[test]
fn csharp_03_t03_w08_wrapper_projection_revalidates_binding_and_representation() {
    let b = bundle();
    let identity = json!({"kind":"type","namespace":"Business","owner":"","name":"Box","parameter_type_ids":[],"result_type_id":""});
    let id = csharp_practical_declaration_id(&identity).unwrap();
    let member = csharp_practical_stored_member_id(
        &id,
        "Items",
        &descriptor("bounded_sequence"),
        "readonly_field",
    )
    .unwrap();
    let tag_type = json!({"kind":"primitive","id":"i32"});
    let tag = csharp_practical_stored_member_id(&id, "Tag", &tag_type, "readonly_field").unwrap();
    let source = json!({"id":id,"identity":identity,"kind":"sealed_class","members":[
        {"id":member,"name":"Items","type":descriptor("bounded_sequence"),"storage":"readonly_field","ordinal":0,"required":false},
        {"id":tag,"name":"Tag","type":tag_type,"storage":"readonly_field","ordinal":1,"required":false}
    ],"enum_values":[],"enum_underlying":null,"actual_default":{member.clone():null,tag:0},"public_default":false,"identity_sensitive":false,"source_sha256":"a".repeat(64)});
    let transport=canonical_closed_root_set_transport(&b,&json!([{ "origin":"semantic_binding","provenance_id":"wrapper.source","type":{"kind":"source","id":id}}]),&json!({id.clone():source})).unwrap();
    let r = validate_closed_root_set(&b, &transport).unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    let sequence =
        csharp_practical_closed_instance_id(&b, &descriptor("bounded_sequence")).unwrap();
    let value = MonomorphicValue::Product {
        type_id: id.clone(),
        fields: vec![
            NamedMonomorphicValue {
                name: "Items".into(),
                value: Box::new(MonomorphicValue::Array {
                    type_id: sequence.clone(),
                    elements: vec![integer(4), integer(5)],
                }),
            },
            NamedMonomorphicValue {
                name: "Tag".into(),
                value: Box::new(integer(2)),
            },
        ],
    };
    let binding = SequenceWrapperBinding {
        source_type_id: id,
        source_content_sha256: "a".repeat(64),
        elements_member_id: member,
        sequence_type_id: sequence.clone(),
    };
    let projected = project_bounded_sequence_wrapper(&b, &r, &c, &value, &binding).unwrap();
    assert_eq!(
        bounded_sequence_read(&b, &r, &c, &projected, 1).unwrap(),
        &integer(5)
    );
    let p = generate_structural_program(&b, &r, &c, &sequence).unwrap();
    assert!(p.structural_equal(&projected, &projected).unwrap());
    for variant in 0..4 {
        let mut changed = binding.clone();
        match variant {
            0 => changed.source_type_id.push('x'),
            1 => changed.source_content_sha256 = "b".repeat(64),
            2 => changed.elements_member_id.push('x'),
            _ => changed.sequence_type_id = integer(0).type_id().into(),
        }
        assert!(project_bounded_sequence_wrapper(&b, &r, &c, &value, &changed).is_err());
    }
    let mut changed = value.clone();
    if let MonomorphicValue::Product { fields, .. } = &mut changed {
        *fields[0].value = integer(0);
    }
    assert!(project_bounded_sequence_wrapper(&b, &r, &c, &changed, &binding).is_err());
}

#[test]
fn csharp_03_t03_w08_pinned_source_harness_when_available() {
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
        .arg("--test-sequences")
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
fn csharp_03_t03_w08_replays_the_actual_source_handoff() {
    let b = bundle();
    let (r, c, construction, sequence) = fixture(&b);
    // The pinned Roslyn harness regenerates this entire projection from the
    // direct-result source, including length and ordered initializer values.
    let source = json_file("develop/migrations/csharp-03/sequences/source-direct.json");
    assert_eq!(source["construction_type_id"], construction);
    assert_eq!(source["sequence_type_id"], sequence);
    assert_eq!(source["element_type_id"], integer(0).type_id());
    let run = || {
        let mut batch = SequenceConstructionBatch::new(&b, &r, &c);
        batch
            .allocate(
                "result",
                source["construction_type_id"].as_str().unwrap(),
                "owner",
                source["length"].as_i64().unwrap(),
            )
            .unwrap();
        for (i, n) in source["values"].as_array().unwrap().iter().enumerate() {
            batch
                .apply(
                    "result",
                    &fill(i as i32),
                    Some(integer(n.as_i64().unwrap() as i32)),
                )
                .unwrap();
        }
        batch.apply("result", &freeze(&sequence), None).unwrap();
        batch.finish().unwrap()
    };
    assert_eq!(run(), run());
    assert_eq!(
        bounded_sequence_read(&b, &r, &c, &run()["result"], 1).unwrap(),
        &integer(2)
    );
}

#[test]
fn csharp_03_t03_w08_every_positive_two_pass_vector_belongs_to_t04() {
    let package = json_file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let vectors: Vec<_> = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["inputs"]["operation"] == "source.array_two_pass")
        .collect();
    assert!(!vectors.is_empty());
    for vector in vectors {
        assert_eq!(vector["implementation_owner"], "CSHARP-03-T04-W02");
        assert_eq!(
            vector["production_test_owner"],
            "crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W02"
        );
    }
}

#[test]
fn csharp_03_t03_w08_float_sequences_have_equality_but_no_order() {
    let b = bundle();
    let ty = json!({"kind":"instance","template":"bounded_sequence","arguments":[{"kind":"primitive","id":"f32"}]});
    let transport = canonical_closed_root_set_transport(
        &b,
        &json!([{ "origin":"source_array","provenance_id":"float.source","type":ty}]),
        &json!({}),
    )
    .unwrap();
    let r = validate_closed_root_set(&b, &transport).unwrap();
    let c = derive_closed_instances(&b, &r).unwrap();
    let id = csharp_practical_closed_instance_id(&b, &ty).unwrap();
    let p = generate_structural_program(&b, &r, &c, &id).unwrap();
    let nan = MonomorphicValue::Sequence {
        type_id: id,
        elements: vec![MonomorphicValue::F32Bits {
            type_id: "mpk.csharp.value.f32.v1".into(),
            bits: "7fc00000".into(),
        }],
    };
    assert!(!p.is_total());
    assert!(!p.structural_equal(&nan, &nan).unwrap());
    assert!(p.canonical_compare(&nan, &nan).is_err());
    let invalid = MonomorphicValue::Sequence {
        type_id: integer(0).type_id().into(),
        elements: vec![integer(0)],
    };
    assert!(bounded_sequence_length(&b, &r, &c, &invalid).is_err());
    assert!(bounded_sequence_read(&b, &r, &c, &invalid, 0).is_err());
}
