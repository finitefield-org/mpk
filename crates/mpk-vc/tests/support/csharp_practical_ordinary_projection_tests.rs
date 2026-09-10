pub(super) fn projection_sources() -> Vec<(String, Value, Value)> {
    let mut sources = structural_foundation_tests::sources();
    let requests = read("ordinary-foundation/projection-sources/requests.json");
    let responses = read("ordinary-foundation/projection-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    sources.push((
        "remapped-boundary-sequence".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    sources
}
use super::*;

#[test]
fn csharp_03_t06_w09_binding_projections_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_PROJECTIONS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-projections");
    let mut rows = vec![];
    let mut total = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in projection_sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_binding_projections(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(p.definitions().len(), vir.binding_projections().len());
        for (def, original) in p.definitions().iter().zip(vir.binding_projections()) {
            assert_eq!(&def.projection, original);
            assert_eq!(def.source_carrier.type_id, original.source_type_id);
            assert_eq!(def.semantic_carrier.type_id, original.semantic_type_id);
            assert_eq!(
                def.reconstruct_definition.is_some(),
                original.binding_id == "binding.identity"
            );
            if original.binding_id != "binding.identity" {
                let OrdinaryShape::Product { fields } = &def.source_carrier.shape else {
                    panic!()
                };
                assert_eq!(
                    def.reconstruction_member_ids,
                    fields.iter().map(|f| f.id.clone()).collect::<Vec<_>>()
                );
            }
        }
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_binding_projections(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "binding_vc_sha256",
            "definitions",
            "certificate_sha256",
        ] {
            let mut changed = metadata.clone();
            changed[field] = json!("forged");
            assert!(
                import_csharp_practical_ordinary_binding_projections(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err(),
                "{id} {field}"
            );
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_binding_projections(
            &p.canonical_bytes(),
            &changed,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_projections(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"program":metadata,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        total += p.definitions().len();
        eprintln!(
            "binding projection {id}: {} definitions, {} terms",
            p.definitions().len(),
            cert.term_table.len()
        );
    }
    assert_eq!(rows.len(), 44);
    assert!(total > 0);
    let result = json!({"sources":rows,"projections":total});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-projections/certificates.json"),
            result
        );
    }
    eprintln!("binding projections: {total}");
}

// The oracle reads captured source names and frozen binding/instance rows. It
// never consults the generated getter, converter or certificate to project a value.
pub(super) struct ProjectionOracle<'a> {
    pub(super) facts: &'a Value,
    pub(super) bindings: Vec<Value>,
    pub(super) closed: &'a ClosedInstanceSet,
}
impl ProjectionOracle<'_> {
    fn instance(&self, role: &str, args: &[String]) -> String {
        let found = self
            .closed
            .entries()
            .iter()
            .filter(|e| {
                e["template_id"] == format!("mpk.csharp.semantic.{role}.v1")
                    && e["argument_ids"] == json!(args)
            })
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1);
        found[0]["instance_id"].as_str().unwrap().into()
    }
    pub(super) fn project(&self, v: &MonomorphicValue, to: &str) -> MonomorphicValue {
        if v.type_id() == to {
            return v.clone();
        }
        let entry = self
            .closed
            .entries()
            .iter()
            .find(|e| e["instance_id"] == to);
        let args = entry
            .map(|e| {
                e["argument_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| a.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let role = entry
            .map(|e| {
                e["template_id"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("mpk.csharp.semantic.")
                    .trim_end_matches(".v1")
            })
            .unwrap_or("instant");
        let binding = self
            .bindings
            .iter()
            .find(|b| b["source_type_id"] == v.type_id());
        let member = |role: &str| {
            let binding = binding.unwrap();
            let source = self.facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == v.type_id())
                .unwrap();
            let name = source["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| {
                    csharp_practical_stored_member_id(
                        v.type_id(),
                        m["name"].as_str().unwrap(),
                        &m["type"],
                        m["storage"].as_str().unwrap(),
                    )
                    .unwrap()
                        == binding["member_map"][role].as_str().unwrap()
                })
                .unwrap()["name"]
                .as_str()
                .unwrap();
            let MonomorphicValue::Product { fields, .. } = v else {
                panic!()
            };
            fields
                .iter()
                .find(|f| f.name == name)
                .unwrap()
                .value
                .as_ref()
        };
        let own = to.to_owned();
        match role {
            "instant" => {
                let MonomorphicValue::Signed { value, .. } = member("milliseconds") else {
                    panic!()
                };
                MonomorphicValue::Instant {
                    type_id: own,
                    milliseconds: value.clone(),
                }
            }
            "money" => MonomorphicValue::Money {
                type_id: own,
                amount: Box::new(self.project(member("amount"), &ty("decimal"))),
                currency: Box::new(self.project(member("currency"), &args[0])),
            },
            "ordered_entry" => MonomorphicValue::OrderedEntry {
                type_id: own,
                key: Box::new(self.project(member("key"), &args[0])),
                value: Box::new(self.project(member("value"), &args[1])),
            },
            "transition" => {
                let seq = self.instance("bounded_sequence", &args[1..2]);
                let MonomorphicValue::Sequence { elements, .. } =
                    self.project(member("events"), &seq)
                else {
                    panic!()
                };
                MonomorphicValue::Transition {
                    type_id: own,
                    state: Box::new(self.project(member("state"), &args[0])),
                    events: elements,
                    response: Box::new(self.project(member("response"), &args[2])),
                }
            }
            "bounded_sequence" | "ordered_set" | "ordered_map" => {
                let input = if binding.is_some() {
                    member(if role == "ordered_map" {
                        "entries"
                    } else {
                        "elements"
                    })
                } else {
                    v
                };
                let (MonomorphicValue::Sequence { elements, .. }
                | MonomorphicValue::Array { elements, .. }) = input
                else {
                    panic!("sequence source: {input:?}")
                };
                let element_id = if role == "ordered_map" {
                    self.instance("ordered_entry", &args)
                } else {
                    args[0].clone()
                };
                let values = elements
                    .iter()
                    .map(|v| self.project(v, &element_id))
                    .collect::<Vec<_>>();
                match role {
                    "ordered_map" => MonomorphicValue::OrderedMap {
                        type_id: own,
                        entries: values
                            .into_iter()
                            .map(|v| {
                                let MonomorphicValue::OrderedEntry { key, value, .. } = v else {
                                    panic!()
                                };
                                MonomorphicMapEntry { key, value }
                            })
                            .collect(),
                    },
                    "ordered_set" => MonomorphicValue::OrderedSet {
                        type_id: own,
                        elements: values,
                    },
                    _ => MonomorphicValue::Sequence {
                        type_id: own,
                        elements: values,
                    },
                }
            }
            "option" | "lookup" | "result" | "validation" | "boundary_field" => {
                let (arm, payload) = if let Some(binding) = binding {
                    let MonomorphicValue::Enum { carrier, .. } = member("tag") else {
                        panic!()
                    };
                    let arm = binding["tag_arms"]
                        .as_object()
                        .unwrap()
                        .iter()
                        .find(|(_, v)| v.as_str() == Some(carrier))
                        .unwrap()
                        .0
                        .as_str();
                    let payload = match (role, arm) {
                        ("option", "none")
                        | ("lookup", "missing_key")
                        | ("boundary_field", "missing" | "null") => None,
                        ("result", "error") => Some(member("error")),
                        ("validation", "invalid") => Some(member("errors")),
                        _ => Some(member("value")),
                    };
                    (arm, payload)
                } else {
                    match v {
                        MonomorphicValue::Option { arm, value, .. } => (
                            if *arm == OptionArm::None {
                                "none"
                            } else {
                                "some"
                            },
                            value.as_deref(),
                        ),
                        MonomorphicValue::TaggedSum { arm, payload, .. } => {
                            (arm.as_str(), payload.first())
                        }
                        _ => panic!("sum source {v:?}"),
                    }
                };
                let payload = payload.map(|p| {
                    let target = match (role, arm) {
                        ("validation", "invalid") => self.instance("bounded_sequence", &args[1..2]),
                        ("result", "error") => args[1].clone(),
                        _ => args[0].clone(),
                    };
                    Box::new(self.project(p, &target))
                });
                match role {
                    "option" => MonomorphicValue::Option {
                        type_id: own,
                        arm: if arm == "none" {
                            OptionArm::None
                        } else {
                            OptionArm::Some
                        },
                        value: payload,
                    },
                    "boundary_field" => MonomorphicValue::BoundaryPresence {
                        type_id: own,
                        arm: match arm {
                            "missing" => BoundaryArm::Missing,
                            "null" => BoundaryArm::Null,
                            _ => BoundaryArm::Value,
                        },
                        value: payload,
                    },
                    _ => MonomorphicValue::TaggedSum {
                        type_id: own,
                        arm: arm.into(),
                        payload: payload.into_iter().map(|v| *v).collect(),
                    },
                }
            }
            _ => panic!("unhandled projection {role}"),
        }
    }
}

pub(super) fn sparse_storage(
    v: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> (u32, BTreeSet<usize>) {
    let depth = types[v.type_id()].depth;
    let pack = |children: Vec<(u32, BTreeSet<usize>)>| {
        let roles = if children.len() <= 1 {
            0
        } else {
            usize::BITS - (children.len() - 1).leading_zeros()
        };
        let max = children.iter().map(|c| c.0).max().unwrap_or(0);
        let mut ones = BTreeSet::new();
        for (i, (d, bits)) in children.into_iter().enumerate() {
            ones.extend(bits.into_iter().map(|n| i | (n << (roles + max - d))));
        }
        (roles + max, ones)
    };
    let packed = match v {
        MonomorphicValue::Product { fields, .. } => pack(
            fields
                .iter()
                .map(|f| sparse_storage(&f.value, types))
                .collect(),
        ),
        MonomorphicValue::OrderedEntry { key, value, .. } => pack(vec![
            sparse_storage(key, types),
            sparse_storage(value, types),
        ]),
        MonomorphicValue::Sequence { elements, .. }
        | MonomorphicValue::Array { elements, .. }
        | MonomorphicValue::OrderedSet { elements, .. } => sparse_sequence(
            depth,
            &types[v.type_id()].shape,
            elements.iter().map(|v| sparse_storage(v, types)).collect(),
        ),
        MonomorphicValue::OrderedMap { entries, .. } => sparse_sequence(
            depth,
            &types[v.type_id()].shape,
            entries
                .iter()
                .map(|e| {
                    pack(vec![
                        sparse_storage(&e.key, types),
                        sparse_storage(&e.value, types),
                    ])
                })
                .collect(),
        ),
        _ => {
            assert!(
                depth <= 22,
                "explicit sparse representation needed for {}",
                v.type_id()
            );
            (
                depth,
                relation_tests::storage(v, types)
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, b)| b.then_some(i))
                    .collect(),
            )
        }
    };
    assert_eq!(packed.0, depth);
    packed
}
fn sparse_sequence(
    depth: u32,
    shape: &OrdinaryShape,
    children: Vec<(u32, BTreeSet<usize>)>,
) -> (u32, BTreeSet<usize>) {
    let OrdinaryShape::Sequence { capacity, .. } = shape else {
        panic!()
    };
    let index_bits = u32::BITS - (capacity - 1).leading_zeros();
    let mut ones = BTreeSet::new();
    for bit in 0..32 {
        if children.len() & (1 << bit) != 0 {
            ones.insert(bit << (depth - 5));
        }
    }
    for (i, (d, bits)) in children.into_iter().enumerate() {
        let offset = depth - index_bits - d;
        ones.extend(
            bits.into_iter()
                .map(|n| 1 | (i << offset) | (n << (offset + index_bits))),
        );
    }
    (depth, ones)
}

#[test]
fn csharp_03_t06_w09_binding_projections_original_source_semantics() {
    use core_eval::{apply, bit, run, sparse_cube, V};
    let bundle = b();
    let mut count = 0;
    let mut bit_count = 0;
    let mut roles = BTreeSet::new();
    for (id, row, facts) in projection_sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_binding_projections(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        roles.extend(
            bindings["bindings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b["role"].as_str().unwrap().to_owned()),
        );
        let oracle = ProjectionOracle {
            facts: &facts,
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for def in p.definitions() {
            for seed in 0..4 {
                let source = relation_tests::sample(
                    &def.source_carrier.type_id,
                    seed,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let expected = oracle.project(&source, &def.semantic_carrier.type_id);
                let (sd, input) = sparse_storage(&source, &types);
                let (td, wanted) = sparse_storage(&expected, &types);
                let output = run(&cert, &def.project_definition, vec![sparse_cube(sd, input)]);
                let mut addresses = if td <= 10 {
                    (0..1 << td).collect::<BTreeSet<_>>()
                } else {
                    BTreeSet::from([0, (1 << td) - 1])
                };
                addresses.extend((0..td).map(|i| 1 << i));
                for &at in &wanted {
                    addresses.extend([at, at.saturating_sub(1), (at + 1).min((1 << td) - 1)]);
                }
                if id == "remapped-boundary-sequence" {
                    if let MonomorphicValue::Sequence { elements, .. } = &expected {
                        let entry = oracle
                            .closed
                            .entries()
                            .iter()
                            .find(|e| e["instance_id"] == expected.type_id())
                            .unwrap();
                        let child_depth = types[entry["argument_ids"][0].as_str().unwrap()].depth;
                        assert_eq!((td, child_depth), (19, 6));
                        for index in [elements.len(), elements.len() + 1, 4095] {
                            for leaf in 0..1usize << child_depth {
                                addresses.insert(1 | (index << 1) | (leaf << 13));
                            }
                        }
                    }
                }
                let mut outputs = vec![output];
                if id == "remapped-boundary-sequence" {
                    let mut changed = source.clone();
                    let MonomorphicValue::Product { fields, .. } = &mut changed else {
                        panic!()
                    };
                    let extra = fields.iter_mut().find(|f| f.name == "Extra").unwrap();
                    *extra.value = MonomorphicValue::Signed {
                        type_id: ty("i32"),
                        value: "73".into(),
                    };
                    assert_eq!(
                        oracle.project(&changed, &def.semantic_carrier.type_id),
                        expected
                    );
                    let (changed_depth, changed_bits) = sparse_storage(&changed, &types);
                    assert_ne!(sparse_storage(&source, &types).1, changed_bits);
                    assert!(def.reconstruct_definition.is_none());
                    assert_eq!(def.reconstruction_member_ids.len(), fields_count(&source));
                    outputs.push(run(
                        &cert,
                        &def.project_definition,
                        vec![sparse_cube(changed_depth, changed_bits)],
                    ));
                    count += 1;
                }
                for output in outputs {
                    for &at in &addresses {
                        bit_count += 1;
                        let mut leaf = output.clone();
                        for i in 0..td {
                            leaf = apply(&cert, leaf, V::Bit(at & (1 << i) != 0));
                        }
                        assert_eq!(
                            bit(leaf),
                            wanted.contains(&at),
                            "{id} seed {seed} address {at}"
                        );
                    }
                }
                count += 1;
            }
            eprintln!("projection values {id}: {}", def.source_carrier.type_id);
        }
    }
    assert_eq!(count, 144);
    assert_eq!(
        roles,
        [
            "instant",
            "money",
            "ordered_entry",
            "ordered_map",
            "ordered_set",
            "bounded_sequence",
            "option",
            "result",
            "lookup",
            "validation",
            "boundary_field",
            "transition"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    eprintln!("projection observations: {count}, observed bits: {bit_count}");
}

#[test]
fn csharp_03_t06_w09_binding_projection_source_requests() {
    let bundle = b();
    let id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"ProjectionCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let item = id("Item");
    let rep = id("Root");
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"ProjectionCases","owner":id("Entry"),"name":"Run","parameter_type_ids":[rep.clone()],"result_type_id":rep})).unwrap();
    let code="namespace ProjectionCases;public enum Tag:long{Present=0,Null=1,Missing=-9223372036854775808L}public readonly struct Item{public readonly Tag Tag;public readonly int Value;public readonly int Extra;}public readonly struct Root{public readonly Item[] Items;public readonly int Extra;}public static class Entry{public static Root Run(Root value){return value;}}\n";
    let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
    let bind = |source: &str, role: &str, members: Vec<(&str, &str, Value)>, args: Vec<String>| {
        SemanticBindingInput {
            source_type_id: source.into(),
            source_content_sha256: captures.entries()[0].raw_sha256().into(),
            role: role.into(),
            member_map: members
                .into_iter()
                .map(|(role, name, value)| SemanticBindingMember {
                    role: role.into(),
                    member_id: csharp_practical_stored_member_id(
                        source,
                        name,
                        &value,
                        "readonly_field",
                    )
                    .unwrap(),
                })
                .collect(),
            inferred_argument_ids: args,
            tag_arms: if role == "boundary_field" {
                vec![
                    SemanticArmMapping {
                        semantic_arm: "null".into(),
                        source_tag: "1".into(),
                    },
                    SemanticArmMapping {
                        semantic_arm: "value".into(),
                        source_tag: "0".into(),
                    },
                    SemanticArmMapping {
                        semantic_arm: "missing".into(),
                        source_tag: "-9223372036854775808".into(),
                    },
                ]
            } else {
                vec![]
            },
            default_arm: "ineligible".into(),
            bounds: if role == "bounded_sequence" {
                vec![SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }]
            } else {
                vec![]
            },
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        }
    };
    let option = csharp_practical_closed_instance_id(
        &bundle,
        &instance("boundary_field", vec![primitive("i32")]),
    )
    .unwrap();
    let rows = vec![
        bind(
            &item,
            "boundary_field",
            vec![
                ("tag", "Tag", json!({"kind":"source","id":id("Tag")})),
                ("value", "Value", primitive("i32")),
            ],
            vec![ty("i32")],
        ),
        bind(
            &rep,
            "bounded_sequence",
            vec![(
                "elements",
                "Items",
                instance("bounded_sequence", vec![json!({"kind":"source","id":item})]),
            )],
            vec![option],
        ),
    ];
    let sidecar = build_semantic_bindings(&plain, &captures, rows)
        .unwrap()
        .canonical_bytes()
        .to_vec();
    let (context, captures) =
        support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
    let requests = json!([{"id":"remapped-boundary-sequence","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_PROJECTION_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/projection-sources/requests.json")).unwrap(),bytes);
    }
}

fn fields_count(v: &MonomorphicValue) -> usize {
    let MonomorphicValue::Product { fields, .. } = v else {
        panic!()
    };
    fields.len()
}

#[path = "csharp_practical_ordinary_binding_relation_tests.rs"]
mod binding_relation_tests;
