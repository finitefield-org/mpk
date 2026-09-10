use super::*;
use core_eval::{apply, bit as observed_bit, run, sparse_cube, V};
use relation_tests::{sample, storage};
use sequence_tests::number;

#[test]
fn csharp_03_t06_w09_collection_source_requests() {
    let bundle = b();
    let source_id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"CollectionCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let source_type = |name: &str| json!({"kind":"source","id":source_id(name)});
    let rep = source_id("Rep");
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"CollectionCases","owner":source_id("Entry"),"name":"Run","parameter_type_ids":[rep.clone()],"result_type_id":rep})).unwrap();
    let nullable = instance("option", vec![primitive("i32")]);
    let mut requests = vec![];
    for (id, value_cs, value_type, multi) in [
        ("bool-float-map", "float", primitive("f32"), false),
        ("bool-nullable-map", "int?", nullable, false),
        ("shared-lookup-maps", "int", primitive("i32"), true),
    ] {
        let body = if multi {
            "public readonly struct PairA{public readonly bool Key;public readonly int Value;}public readonly struct MapA{public readonly PairA[] Items;}public readonly struct PairB{public readonly int Key;public readonly int Value;}public readonly struct MapB{public readonly PairB[] Items;}public readonly struct Rep{public readonly MapA Left;public readonly MapB Right;}".into()
        } else {
            format!("public readonly struct Pair{{public readonly bool Key;public readonly {value_cs} Value;}}public readonly struct Rep{{public readonly Pair[] Items;}}")
        };
        let code=format!("namespace CollectionCases;{body}public static class Entry{{public static Rep Run(Rep value){{return value;}}}}\n");
        let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
        let argument_id = |value: &Value| match value["kind"].as_str().unwrap() {
            "primitive" => ty(value["id"].as_str().unwrap()),
            "instance" => csharp_practical_closed_instance_id(&bundle, value).unwrap(),
            _ => panic!(),
        };
        let binding = |source: &str,
                       role: &str,
                       members: Vec<(&str, &str, Value)>,
                       arguments: Vec<String>| SemanticBindingInput {
            source_type_id: source_id(source),
            source_content_sha256: captures.entries()[0].raw_sha256().into(),
            role: role.into(),
            member_map: members
                .into_iter()
                .map(|(role, name, value)| SemanticBindingMember {
                    role: role.into(),
                    member_id: csharp_practical_stored_member_id(
                        &source_id(source),
                        name,
                        &value,
                        "readonly_field",
                    )
                    .unwrap(),
                })
                .collect(),
            inferred_argument_ids: arguments,
            tag_arms: vec![],
            default_arm: "ineligible".into(),
            bounds: if role == "ordered_map" {
                vec![SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }]
            } else {
                vec![]
            },
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        };
        let rows = if multi {
            vec![
                ("PairA", "MapA", primitive("bool")),
                ("PairB", "MapB", primitive("i32")),
            ]
        } else {
            vec![("Pair", "Rep", primitive("bool"))]
        };
        let mut bindings = vec![];
        for (pair, map, key) in rows {
            let args = vec![argument_id(&key), argument_id(&value_type)];
            bindings.push(binding(
                pair,
                "ordered_entry",
                vec![("key", "Key", key), ("value", "Value", value_type.clone())],
                args.clone(),
            ));
            bindings.push(binding(
                map,
                "ordered_map",
                vec![(
                    "entries",
                    "Items",
                    instance("bounded_sequence", vec![source_type(pair)]),
                )],
                args,
            ));
        }
        let sidecar = build_semantic_bindings(&plain, &captures, bindings)
            .unwrap()
            .canonical_bytes()
            .to_vec();
        let (context, captures) =
            support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(output) = std::env::var_os("MPK_W09_COLLECTION_REQUESTS_OUT") {
        fs::write(output, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/collection-sources/requests.json")).unwrap(),bytes);
    }
}

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let requests = read("ordinary-foundation/collection-sources/requests.json");
    let responses = read("ordinary-foundation/collection-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 3);
    assert_eq!(responses.as_array().unwrap().len(), 3);
    let extra = requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
        .map(|(row, response)| {
            assert_eq!(row["id"], response["id"]);
            assert!(response.get("reject").is_none());
            (
                format!("extra-{}", row["id"].as_str().unwrap()),
                row.clone(),
                response["facts"].clone(),
            )
        });
    relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .chain(extra)
        .collect()
}
fn packed(
    value: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> (u32, BTreeSet<usize>) {
    if matches!(value, MonomorphicValue::OrderedMap { .. }) {
        domain_tests::sparse_map_storage(value, types)
    } else {
        let bits = storage(value, types);
        (
            types[value.type_id()].depth,
            bits.into_iter()
                .enumerate()
                .filter_map(|(i, b)| b.then_some(i))
                .collect(),
        )
    }
}
fn input(value: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    let (depth, bits) = packed(value, types);
    sparse_cube(depth, bits)
}
fn observe(
    c: &mpk_cert::encode::Certificate,
    actual: V,
    expected: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) {
    let (depth, bits) = packed(expected, types);
    let length = 1usize << depth;
    let mut indices = BTreeSet::from([0, length - 1]);
    for bit in 0..depth {
        indices.insert(1 << bit);
    }
    for &i in &bits {
        indices.insert(i);
        if i > 0 {
            indices.insert(i - 1);
        }
        if i + 1 < length {
            indices.insert(i + 1);
        }
    }
    // Exhaust small carriers; large inactive regions are sampled, never
    // claimed as universally proved by these finite observations.
    if depth <= 10 {
        indices.extend(0..length);
    }
    for i in indices {
        let mut result = actual.clone();
        for bit in 0..depth {
            result = apply(c, result, V::Bit(i & (1 << bit) != 0));
        }
        assert_eq!(
            observed_bit(result),
            bits.contains(&i),
            "{} leaf {i}",
            expected.type_id()
        );
    }
}
fn operation<'a>(
    d: &'a OrdinaryCollectionDefinition,
    suffix: &str,
) -> &'a OrdinaryCollectionOperation {
    d.operations
        .iter()
        .find(|op| op.operation_id.ends_with(&format!(".{suffix}")))
        .unwrap()
}
#[test]
fn csharp_03_t06_w09_collections_original_source_certificates() {
    let bundle = b();
    let mut metrics = vec![];
    let output = std::env::var_os("MPK_W09_COLLECTIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/collection-operations");
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_collections(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let expected = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| {
                matches!(
                    e["template_id"].as_str(),
                    Some(
                        "mpk.csharp.semantic.ordered_map.v1" | "mpk.csharp.semantic.ordered_set.v1"
                    )
                )
            })
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected,
            p.definitions()
                .iter()
                .map(|d| d.carrier.type_id.as_str())
                .collect()
        );
        if p.definitions().is_empty() {
            continue;
        }
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        for d in p.definitions() {
            for op in d.operations.iter().filter(|op| {
                op.operation_id.ends_with(".add") || op.operation_id.ends_with(".replace")
            }) {
                let name = format!("{}.At", op.normal_definition);
                let declaration = c
                    .declarations
                    .iter()
                    .find(|decl| c.name_table[decl.name as usize] == name)
                    .unwrap();
                let mpk_cert::encode::DeclarationKind::Def { value, .. } = declaration.kind else {
                    panic!()
                };
                let mut term = value;
                let mut binders = 0;
                while let mpk_cert::encode::TermNode::Lam { body, .. } = c.term_table[term as usize]
                {
                    binders += 1;
                    term = body;
                }
                assert_eq!(
                    binders,
                    d.carrier.depth + 3,
                    "C253 must fit within 256 binders"
                );
            }
        }
        assert_eq!(
            import_csharp_practical_ordinary_collections(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for key in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut m = data.clone();
            m[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_collections(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        for key in [
            "key_type_id",
            "value_type_id",
            "lower_bound_definition",
            "operations",
        ] {
            let mut m = data.clone();
            m["definitions"][0][key] = json!("forged");
            assert!(import_csharp_practical_ordinary_collections(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_collections(
            &p.canonical_bytes(),
            &corrupt,
            emitted.vir()
        )
        .is_err());
        let file = format!("{id}.hex");
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(output) = &output {
            fs::create_dir_all(output).unwrap();
            fs::write(output.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        eprintln!(
            "collection certificate {id}: {} terms, {} declarations",
            c.term_table.len(),
            c.declarations.len()
        );
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert_eq!(metrics.len(), 9);
    if let Some(output) = output {
        fs::write(
            output.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join("certificates.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}
fn collection_semantics(extra_only: bool) {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(move ||{
        let bundle=b();let mut contexts=0;let mut observations=0;let mut non_total=false;let mut nullable=false;
        for (id,row,facts) in sources().into_iter().filter(|(id,_,_)|!extra_only || id.starts_with("extra-")) {
            let (context,captures)=support::replay_context(&bundle,&row);
            let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();
            let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();
            let p=generate_csharp_practical_ordinary_collections(emitted.vir()).unwrap();
            let c=mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            let types=generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap().carriers().iter().map(|c|(c.type_id.clone(),c.clone())).collect::<BTreeMap<_,_>>();
            for d in p.definitions() {
                contexts+=1;eprintln!("collection semantics {id}: {}",d.carrier.type_id);
                let model=OrderedCollectionModel::new(&bundle,emitted.closure().roots(),emitted.closure().closed(),&d.carrier.type_id).unwrap();
                let order=generate_structural_program(&bundle,emitted.closure().roots(),emitted.closure().closed(),&d.key_type_id).unwrap();
                let relation=generate_structural_program(&bundle,emitted.closure().roots(),emitted.closure().closed(),&d.carrier.type_id).unwrap();
                let mut keys=(0..5).map(|seed|sample(&d.key_type_id,seed,&types,&facts,emitted.closure().closed())).collect::<Vec<_>>();
                keys.sort_by(|a,b|order.canonical_compare(a,b).unwrap());
                keys.dedup_by(|a,b|order.structural_equal(a,b).unwrap());keys.truncate(3);
                assert!(keys.len()>=2);
                let value=d.value_type_id.as_ref().map(|value_id| {
                    let seed=if value_id==&ty("f32") || id=="extra-bool-nullable-map" {0}else{1};
                    sample(value_id,seed,&types,&facts,emitted.closure().closed())
                });
                non_total |= !relation.is_total();
                nullable |= matches!(&value,Some(MonomorphicValue::Option{value:None,..}));
                let replacement=d.value_type_id.as_ref().map(|id|sample(id,2,&types,&facts,emitted.closure().closed()));
                let make=|indices:&[usize]| if d.value_type_id.is_some() {MonomorphicValue::OrderedMap{type_id:d.carrier.type_id.clone(),entries:indices.iter().map(|&i|MonomorphicMapEntry{key:Box::new(keys[i].clone()),value:Box::new(value.clone().unwrap())}).collect()}} else {MonomorphicValue::OrderedSet{type_id:d.carrier.type_id.clone(),elements:indices.iter().map(|&i|keys[i].clone()).collect()}};
                for indices in [vec![],vec![keys.len()-1],vec![0,keys.len()-1]] {
                    let current=make(&indices);model.validate(&current).unwrap();
                    let encoded=input(&current,&types);
                    assert_eq!(number(&c,run(&c,&operation(d,"count").normal_definition,vec![encoded.clone()])),indices.len() as u32);observations+=1;
                    for query in &keys {
                        let position=indices.iter().take_while(|&&i|order.canonical_compare(&keys[i],query).unwrap()==std::cmp::Ordering::Less).count();
                        let found=model.contains(&current,query).unwrap();
                        let arguments=vec![encoded.clone(),input(query,&types)];
                        assert_eq!(number(&c,run(&c,&d.lower_bound_definition,arguments.clone())),position as u32);
                        assert_eq!(observed_bit(run(&c,&operation(d,"contains").normal_definition,arguments.clone())),found);observations+=2;
                        if d.value_type_id.is_some() {
                            let lookup=model.lookup(&current,query).unwrap();
                            observe(&c,run(&c,&operation(d,"lookup").normal_definition,arguments.clone()),&lookup,&types);observations+=1;
                        }
                        let mut update_arguments=arguments.clone();if let Some(v)=&replacement{update_arguments.push(input(v,&types));}
                        let add=operation(d,"add");
                        assert_eq!(observed_bit(run(&c,&add.failures[1].definition,arguments.clone())),found);
                        assert!(!observed_bit(run(&c,&add.failures[2].definition,vec![encoded.clone()])));observations+=2;
                        if !found {
                            let result=model.update(&current,query.clone(),replacement.clone(),false,4096).unwrap();
                            observe(&c,run(&c,&add.normal_definition,update_arguments.clone()),&result,&types);observations+=1;
                        }
                        if d.value_type_id.is_some() {
                            let replace=operation(d,"replace");
                            assert_eq!(observed_bit(run(&c,&replace.failures[1].definition,arguments)),!found);observations+=1;
                            if found {
                                // The query is the stored key here. Different equal decimal
                                // representations get a separate storage-preservation case.
                                let stored_query=keys[indices[position]].clone();
                                let result=model.update(&current,stored_query,replacement.clone(),true,4096).unwrap();
                                observe(&c,run(&c,&replace.normal_definition,update_arguments),&result,&types);observations+=1;
                            }
                        }
                    }
                    for other in [make(&[]),current.clone()] {
                        let arguments=vec![encoded.clone(),input(&other,&types)];
                        assert_eq!(observed_bit(run(&c,&operation(d,"equal").normal_definition,arguments.clone())),relation.structural_equal(&current,&other).unwrap());observations+=1;
                        if relation.is_total() {
                            let expected=match relation.canonical_compare(&current,&other).unwrap(){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1};
                            assert_eq!(number(&c,run(&c,&operation(d,"compare").normal_definition,arguments)) as i32,expected);observations+=1;
                        } else {assert!(!d.operations.iter().any(|op|op.operation_id.ends_with(".compare")));}
                    }
                }
            }
        }
        assert_eq!(contexts,if extra_only {4}else{10});assert!(non_total && nullable);eprintln!("collection semantics: {contexts} source/instance contexts, {observations} ordinary observations");
    }).unwrap().join().unwrap();
}

#[test]
fn csharp_03_t06_w09_collections_original_source_semantics() {
    collection_semantics(false);
}

#[test]
fn csharp_03_t06_w09_collections_extra_source_semantics() {
    collection_semantics(true);
}

#[test]
fn csharp_03_t06_w09_collections_maximum_updates() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let mut contexts = 0;
            for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
                matches!(
                    id.as_str(),
                    "binding-vc-ordered_map" | "binding-vc-ordered_set" | "boundary-map-decimal"
                )
            }) {
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let p = generate_csharp_practical_ordinary_collections(emitted.vir()).unwrap();
                let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
                    .unwrap()
                    .carriers()
                    .iter()
                    .map(|c| (c.type_id.clone(), c.clone()))
                    .collect::<BTreeMap<_, _>>();
                let integer = |n: i32| MonomorphicValue::Signed {
                    type_id: ty("i32"),
                    value: n.to_string(),
                };
                for d in p.definitions() {
                    contexts += 1;
                    if id == "boundary-map-decimal" {
                        let original = sample(
                            &d.key_type_id,
                            0,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        );
                        let equivalent = sample(
                            &d.key_type_id,
                            1,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        );
                        let order = generate_structural_program(
                            &bundle,
                            emitted.closure().roots(),
                            emitted.closure().closed(),
                            &d.key_type_id,
                        )
                        .unwrap();
                        assert_ne!(original, equivalent);
                        assert!(order.structural_equal(&original, &equivalent).unwrap());
                        let map = |value| MonomorphicValue::OrderedMap {
                            type_id: d.carrier.type_id.clone(),
                            entries: vec![MonomorphicMapEntry {
                                key: Box::new(original.clone()),
                                value: Box::new(integer(value)),
                            }],
                        };
                        let current = map(7);
                        let result = map(9);
                        eprintln!(
                            "collection replacement {id}: retain exact 1.00 key for equal 1 query"
                        );
                        observe(
                            &c,
                            run(
                                &c,
                                &operation(d, "replace").normal_definition,
                                vec![
                                    input(&current, &types),
                                    input(&equivalent, &types),
                                    input(&integer(9), &types),
                                ],
                            ),
                            &result,
                            &types,
                        );
                        continue;
                    }
                    let map = d.value_type_id.is_some();
                    let make = |length| {
                        if map {
                            MonomorphicValue::OrderedMap {
                                type_id: d.carrier.type_id.clone(),
                                entries: (0..length)
                                    .map(|i| MonomorphicMapEntry {
                                        key: Box::new(integer(2 * i)),
                                        value: Box::new(integer(7)),
                                    })
                                    .collect(),
                            }
                        } else {
                            MonomorphicValue::OrderedSet {
                                type_id: d.carrier.type_id.clone(),
                                elements: (0..length).map(|i| integer(2 * i)).collect(),
                            }
                        }
                    };
                    let model = OrderedCollectionModel::new(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &d.carrier.type_id,
                    )
                    .unwrap();
                    let current = make(4095);
                    let value = map.then(|| integer(9));
                    for key in [-1, 4095, 8190] {
                        let result = model
                            .update(&current, integer(key), value.clone(), false, 4096)
                            .unwrap();
                        let mut arguments =
                            vec![input(&current, &types), input(&integer(key), &types)];
                        if let Some(value) = &value {
                            arguments.push(input(value, &types));
                        }
                        eprintln!("collection maximum update {id}: 4095 -> 4096, key {key}");
                        observe(
                            &c,
                            run(&c, &operation(d, "add").normal_definition, arguments),
                            &result,
                            &types,
                        );
                    }
                    if map {
                        let current = make(4096);
                        let result = model
                            .update(&current, integer(8190), value, true, 4096)
                            .unwrap();
                        observe(
                            &c,
                            run(
                                &c,
                                &operation(d, "replace").normal_definition,
                                vec![
                                    input(&current, &types),
                                    input(&integer(8190), &types),
                                    input(&integer(9), &types),
                                ],
                            ),
                            &result,
                            &types,
                        );
                    }
                }
            }
            assert_eq!(contexts, 3);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_collections_integer_domains_and_capacity() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let mut contexts = 0;
            for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
                matches!(
                    id.as_str(),
                    "binding-vc-ordered_map" | "binding-vc-ordered_set"
                )
            }) {
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let p = generate_csharp_practical_ordinary_collections(emitted.vir()).unwrap();
                let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
                    .unwrap()
                    .carriers()
                    .iter()
                    .map(|c| (c.type_id.clone(), c.clone()))
                    .collect::<BTreeMap<_, _>>();
                for d in p.definitions() {
                    contexts += 1;
                    assert_eq!(d.key_type_id, ty("i32"));
                    let integer = |n: i32| MonomorphicValue::Signed {
                        type_id: ty("i32"),
                        value: n.to_string(),
                    };
                    let make = |keys: Vec<i32>| {
                        if d.value_type_id.is_some() {
                            MonomorphicValue::OrderedMap {
                                type_id: d.carrier.type_id.clone(),
                                entries: keys
                                    .into_iter()
                                    .map(|k| MonomorphicMapEntry {
                                        key: Box::new(integer(k)),
                                        value: Box::new(integer(1)),
                                    })
                                    .collect(),
                            }
                        } else {
                            MonomorphicValue::OrderedSet {
                                type_id: d.carrier.type_id.clone(),
                                elements: keys.into_iter().map(integer).collect(),
                            }
                        }
                    };
                    let valid = operation(d, "validate");
                    let add = operation(d, "add");
                    assert_eq!(
                        add.failures
                            .iter()
                            .map(|f| f.label.as_str())
                            .collect::<Vec<_>>(),
                        vec![
                            "invalid_representation",
                            if d.value_type_id.is_some() {
                                "duplicate_key"
                            } else {
                                "duplicate_element"
                            },
                            "capacity"
                        ]
                    );
                    for (keys, expected) in [
                        (vec![], true),
                        (vec![-1, 1], true),
                        (vec![1, 1], false),
                        (vec![1, -1], false),
                    ] {
                        let value = make(keys);
                        let encoded = input(&value, &types);
                        assert_eq!(
                            observed_bit(run(&c, &valid.normal_definition, vec![encoded.clone()])),
                            expected
                        );
                        assert_eq!(
                            observed_bit(run(&c, &add.failures[0].definition, vec![encoded])),
                            !expected
                        );
                    }
                    // Corrupt zero tail/count padding and every high length bit.
                    let empty = make(vec![]);
                    let (depth, bits) = packed(&empty, &types);
                    assert!(bits.is_empty());
                    for i in [1usize, 2usize] {
                        assert!(!observed_bit(run(
                            &c,
                            &valid.normal_definition,
                            vec![sparse_cube(depth, BTreeSet::from([i]))]
                        )));
                    }
                    for bit in 13..32 {
                        let address = bit << (depth - 5);
                        assert!(!observed_bit(run(
                            &c,
                            &valid.normal_definition,
                            vec![sparse_cube(depth, BTreeSet::from([address]))]
                        )));
                    }
                    let full = make((0..4096).collect());
                    let encoded = input(&full, &types);
                    eprintln!("collection domain {id}: full 4096");
                    assert!(observed_bit(run(
                        &c,
                        &valid.normal_definition,
                        vec![encoded.clone()]
                    )));
                    assert!(observed_bit(run(
                        &c,
                        &add.failures[2].definition,
                        vec![encoded.clone()]
                    )));
                    // At capacity both duplicate and capacity gates are true, and
                    // the frozen list selects the duplicate failure first.
                    assert!(observed_bit(run(
                        &c,
                        &add.failures[1].definition,
                        vec![encoded.clone(), input(&integer(0), &types)]
                    )));
                    assert_eq!(
                        number(
                            &c,
                            run(
                                &c,
                                &d.lower_bound_definition,
                                vec![encoded.clone(), input(&integer(4095), &types)]
                            )
                        ),
                        4095
                    );
                    assert_eq!(
                        number(
                            &c,
                            run(
                                &c,
                                &d.lower_bound_definition,
                                vec![encoded.clone(), input(&integer(4096), &types)]
                            )
                        ),
                        4096
                    );
                    if d.value_type_id.is_some() {
                        let model = OrderedCollectionModel::new(
                            &bundle,
                            emitted.closure().roots(),
                            emitted.closure().closed(),
                            &d.carrier.type_id,
                        )
                        .unwrap();
                        let result = model
                            .update(&full, integer(4095), Some(integer(-3)), true, 4096)
                            .unwrap();
                        observe(
                            &c,
                            run(
                                &c,
                                &operation(d, "replace").normal_definition,
                                vec![
                                    encoded,
                                    input(&integer(4095), &types),
                                    input(&integer(-3), &types),
                                ],
                            ),
                            &result,
                            &types,
                        );
                    }
                }
            }
            assert_eq!(contexts, 2);
        })
        .unwrap()
        .join()
        .unwrap();
}
