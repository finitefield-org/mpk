use super::*;
use core_eval::{bit as observed_bit, run, sparse_cube, V};
use mpk_cert::Certificate;
use sequence_tests::{number, observe};

fn sources() -> Vec<(String, Value, Value)> {
    relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .chain(extra_sources())
        .collect()
}
pub(super) fn extra_sources() -> Vec<(String, Value, Value)> {
    let requests = read("ordinary-foundation/outcome-sources/requests.json");
    let responses = read("ordinary-foundation/outcome-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 3);
    assert_eq!(responses.as_array().unwrap().len(), 3);
    requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
        .map(|(request, response)| {
            assert_eq!(request["id"], response["id"]);
            assert!(response.get("reject").is_none());
            (
                format!("extra-{}", request["id"].as_str().unwrap()),
                request.clone(),
                response["facts"].clone(),
            )
        })
        .collect()
}

#[test]
fn csharp_03_t06_w09_outcome_source_requests() {
    let bundle = b();
    let type_id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"OutcomeCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let rep = type_id("Rep");
    let tag = type_id("Tag");
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"OutcomeCases","owner":type_id("Entry"),"name":"Run","parameter_type_ids":[rep.clone()],"result_type_id":rep})).unwrap();
    let mut requests = vec![];
    for (name, error_cs, error_type, extra) in [
        ("validation-bool", "bool", primitive("bool"), ""),
        (
            "validation-product",
            "Error",
            json!({"kind":"source","id":type_id("Error")}),
            "public readonly struct Error{public readonly int Code;public readonly bool Retry;}",
        ),
        ("lookup-nullable", "", primitive("i32"), ""),
    ] {
        let lookup = name == "lookup-nullable";
        let fields = if lookup {
            "public readonly int? Value;".into()
        } else {
            format!("public readonly int Value;public readonly {error_cs}[] Errors;")
        };
        let code=format!("namespace OutcomeCases;public enum Tag{{A=0,B=1}}{extra}public readonly struct Rep{{public readonly Tag Tag;{fields}}}public static class Entry{{public static Rep Run(Rep value){{return value;}}}}\n");
        let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
        let member = |role: &str, name: &str, ty: &Value| SemanticBindingMember {
            role: role.into(),
            member_id: csharp_practical_stored_member_id(&rep, name, ty, "readonly_field").unwrap(),
        };
        let nullable = instance("option", vec![primitive("i32")]);
        let integer = primitive("i32");
        let mut members = vec![
            member("tag", "Tag", &json!({"kind":"source","id":tag})),
            member("value", "Value", if lookup { &nullable } else { &integer }),
        ];
        if !lookup {
            members.push(member(
                "errors",
                "Errors",
                &instance("bounded_sequence", vec![error_type.clone()]),
            ));
        }
        let arg_id = |value: &Value| match value["kind"].as_str().unwrap() {
            "primitive" => ty(value["id"].as_str().unwrap()),
            "source" => value["id"].as_str().unwrap().to_owned(),
            "instance" => csharp_practical_closed_instance_id(&bundle, value).unwrap(),
            _ => panic!("closed payload type required"),
        };
        let binding = SemanticBindingInput {
            source_type_id: rep.clone(),
            source_content_sha256: captures.entries()[0].raw_sha256().into(),
            role: if lookup { "lookup" } else { "validation" }.into(),
            member_map: members,
            inferred_argument_ids: if lookup {
                vec![arg_id(&nullable)]
            } else {
                vec![ty("i32"), arg_id(&error_type)]
            },
            tag_arms: if lookup {
                vec![
                    SemanticArmMapping {
                        semantic_arm: "missing_key".into(),
                        source_tag: "0".into(),
                    },
                    SemanticArmMapping {
                        semantic_arm: "found".into(),
                        source_tag: "1".into(),
                    },
                ]
            } else {
                vec![
                    SemanticArmMapping {
                        semantic_arm: "valid".into(),
                        source_tag: "0".into(),
                    },
                    SemanticArmMapping {
                        semantic_arm: "invalid".into(),
                        source_tag: "1".into(),
                    },
                ]
            },
            default_arm: if lookup { "missing_key" } else { "ineligible" }.into(),
            bounds: if lookup {
                vec![]
            } else {
                vec![SemanticBound {
                    id: "errors".into(),
                    maximum: 256,
                }]
            },
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        };
        let sidecar = build_semantic_bindings(&plain, &captures, vec![binding])
            .unwrap()
            .canonical_bytes()
            .to_vec();
        let (context, captures) =
            support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
        requests.push(json!({"id":name,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(output) = std::env::var_os("MPK_W09_OUTCOME_REQUESTS_OUT") {
        fs::write(output, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/outcome-sources/requests.json")).unwrap(),bytes);
    }
}
fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn child_id(shape: &OrdinaryShape) -> &str {
    match shape {
        OrdinaryShape::Reference { type_id } => type_id,
        OrdinaryShape::RoleBound { value, .. } => child_id(value),
        _ => panic!("typed payload required"),
    }
}
fn op<'a>(d: &'a OrdinaryOutcomeDefinition, suffix: &str) -> &'a OrdinaryOutcomeOperation {
    d.operations
        .iter()
        .find(|o| o.operation_id == format!("{}.{suffix}", d.carrier.type_id))
        .unwrap()
}
fn fail(c: &Certificate, op: &OrdinaryOutcomeOperation, label: &str, args: &[V]) -> bool {
    let f = op.failures.iter().find(|f| f.label == label).unwrap();
    observed_bit(run(
        c,
        &f.definition,
        f.argument_indices
            .iter()
            .map(|&i| args[i].clone())
            .collect(),
    ))
}

#[test]
fn csharp_03_t06_w09_outcomes_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_OUTCOMES_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/outcome-operations");
    let mut metrics = vec![];
    let mut templates = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_outcomes(emitted.vir())
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
                        "mpk.csharp.semantic.option.v1"
                            | "mpk.csharp.semantic.lookup.v1"
                            | "mpk.csharp.semantic.result.v1"
                            | "mpk.csharp.semantic.validation.v1"
                            | "mpk.csharp.semantic.boundary_field.v1"
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
        templates.extend(p.definitions().iter().map(|d| d.template_id.clone()));
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_outcomes(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut m = data.clone();
            m[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_outcomes(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        for field in [
            "normal_definition",
            "failures",
            "argument_type_ids",
            "result_type_id",
        ] {
            let mut m = data.clone();
            m["definitions"][0]["operations"][0][field] = json!([]);
            assert!(import_csharp_practical_ordinary_outcomes(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut bytes = p.certificate_bytes().to_vec();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_outcomes(
            &p.canonical_bytes(),
            &bytes,
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
            "outcome certificate {id}: {} instances",
            p.definitions().len()
        );
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert_eq!(templates.len(), 5);
    assert_eq!(metrics.len(), 16);
    eprintln!(
        "outcome certificates: {} actual-source contexts; five template families",
        metrics.len()
    );
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

#[test]
fn csharp_03_t06_w09_outcomes_original_source_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(check_semantics)
        .unwrap()
        .join()
        .unwrap();
}
fn check_semantics() {
    let bundle = b();
    let mut templates = BTreeSet::new();
    let mut non_total = false;
    let mut observations = 0;
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
        let p = generate_csharp_practical_ordinary_outcomes(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            let model = OutcomeModel::new(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.carrier.type_id,
            )
            .unwrap();
            templates.insert(model.role().to_owned());
            eprintln!("outcome semantics {id}: {}", model.role());
            let relation = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.carrier.type_id,
            )
            .unwrap();
            let compare = d
                .operations
                .iter()
                .find(|o| o.operation_id.ends_with(".compare"));
            assert_eq!(compare.is_some(), relation.is_total());
            non_total |= !relation.is_total();
            let OrdinaryShape::Sum { arms } = &d.carrier.shape else {
                panic!()
            };
            let sample = |id: &str, seed| {
                relation_tests::sample(id, seed, &types, &facts, emitted.closure().closed())
            };
            let mut values = vec![];
            for arm in arms {
                let seeds = if arm.fields.is_empty() {
                    vec![0]
                } else if model.role() == "validation" && arm.tag == 1 {
                    vec![1, 2]
                } else {
                    vec![0, 1, 2]
                };
                for seed in seeds {
                    let payload = arm.fields.first().map(|f| sample(child_id(&f.shape), seed));
                    let value = model.construct(&arm.id, payload.clone()).unwrap();
                    let constructor = op(
                        d,
                        if arm.id == "missing_key" {
                            "missing"
                        } else {
                            &arm.id
                        },
                    );
                    let arguments = payload
                        .iter()
                        .map(|v| input(relation_tests::storage(v, &types)))
                        .collect::<Vec<_>>();
                    for f in &constructor.failures {
                        assert!(!fail(&c, constructor, &f.label, &arguments));
                    }
                    observe(
                        &c,
                        run(&c, &constructor.normal_definition, arguments),
                        &relation_tests::storage(&value, &types),
                    );
                    observations += 1;
                    values.push((arm.tag, value));
                }
            }
            for (tag, value) in &values {
                let encoded = input(relation_tests::storage(value, &types));
                for operation in &d.operations {
                    let suffix = operation.operation_id.rsplit('.').next().unwrap();
                    if operation
                        .failures
                        .iter()
                        .any(|f| f.label == "invalid_operation")
                    {
                        let active = match model.role() {
                            "option" | "lookup" => 1,
                            "result" => u32::from(suffix == "error_value"),
                            "validation" => u32::from(suffix == "errors"),
                            "boundary_field" => 2,
                            _ => panic!(),
                        };
                        assert_eq!(
                            fail(
                                &c,
                                operation,
                                "invalid_operation",
                                std::slice::from_ref(&encoded)
                            ),
                            *tag != active
                        );
                        let expected = if *tag == active {
                            relation_tests::storage(
                                model.read(value, &arms[active as usize].id).unwrap(),
                                &types,
                            )
                        } else {
                            vec![false; 1 << types[&operation.result_type_id].depth]
                        };
                        observe(
                            &c,
                            run(&c, &operation.normal_definition, vec![encoded.clone()]),
                            &expected,
                        );
                        observations += 1;
                    } else if matches!(suffix, "has_value" | "is_found" | "is_ok" | "is_valid") {
                        let target = if matches!(suffix, "is_ok" | "is_valid") {
                            0
                        } else {
                            1
                        };
                        assert_eq!(
                            observed_bit(run(
                                &c,
                                &operation.normal_definition,
                                vec![encoded.clone()]
                            )),
                            *tag == target
                        );
                        observations += 1;
                    } else if suffix == "tag" {
                        assert_eq!(
                            number(
                                &c,
                                run(&c, &operation.normal_definition, vec![encoded.clone()])
                            ),
                            *tag
                        );
                        observations += 1;
                    } else if suffix == "value_or" {
                        let fallback = sample(&operation.result_type_id, 2);
                        let expected = model.value_or(value, fallback.clone()).unwrap();
                        observe(
                            &c,
                            run(
                                &c,
                                &operation.normal_definition,
                                vec![
                                    encoded.clone(),
                                    input(relation_tests::storage(&fallback, &types)),
                                ],
                            ),
                            &relation_tests::storage(&expected, &types),
                        );
                        observations += 1;
                    }
                }
            }
            for (_, left) in &values {
                for (_, right) in &values {
                    let args = vec![
                        input(relation_tests::storage(left, &types)),
                        input(relation_tests::storage(right, &types)),
                    ];
                    assert_eq!(
                        observed_bit(run(&c, &op(d, "equal").normal_definition, args.clone())),
                        relation.structural_equal(left, right).unwrap()
                    );
                    observations += 1;
                    if let Some(compare) = compare {
                        let expected = match relation.canonical_compare(left, right).unwrap() {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        };
                        assert_eq!(
                            number(&c, run(&c, &compare.normal_definition, args)) as i32,
                            expected
                        );
                        observations += 1;
                    }
                }
            }
            // High tag bits cannot alias an active arm. Domain validity rejects
            // these inputs separately; here verify the exact active-tag gate.
            for operation in d
                .operations
                .iter()
                .filter(|o| o.failures.iter().any(|f| f.label == "invalid_operation"))
            {
                let mut bits = relation_tests::storage(&values.last().unwrap().1, &types);
                for high in 2..32 {
                    let address = high << (d.carrier.depth - 5);
                    bits[address] = true;
                    assert!(fail(
                        &c,
                        operation,
                        "invalid_operation",
                        &[input(bits.clone())]
                    ));
                    bits[address] = false;
                }
            }
        }
    }
    assert_eq!(templates.len(), 5);
    assert!(non_total && observations >= 100);
    eprintln!("outcome semantics: five template families, {observations} ordinary observations, non-total IEEE option covered");
}

#[test]
fn csharp_03_t06_w09_validation_error_append_and_bounds() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let (_, row, facts) = sources()
                .into_iter()
                .find(|(id, _, _)| id == "binding-vc-validation")
                .unwrap();
            let (context, captures) = support::replay_context(&bundle, &row);
            let source = ValidatedDataSource::import_captured_facts(
                &bundle,
                &context,
                &captures,
                &serde_json::to_vec(&facts).unwrap(),
            )
            .unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let p = generate_csharp_practical_ordinary_outcomes(emitted.vir()).unwrap();
            let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            let d = p
                .definitions()
                .iter()
                .find(|d| d.template_id == "mpk.csharp.semantic.validation.v1")
                .unwrap();
            let invalid = op(d, "invalid");
            let append = op(d, "append_errors");
            let sequence_id = &append.result_type_id;
            let model = OutcomeModel::new(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.carrier.type_id,
            )
            .unwrap();
            let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
                .unwrap()
                .carriers()
                .iter()
                .map(|c| (c.type_id.clone(), c.clone()))
                .collect::<BTreeMap<_, _>>();
            let OrdinaryShape::Sequence { element, .. } = &types[sequence_id].shape else {
                panic!()
            };
            assert_eq!(child_id(element), ty("i32"));
            let sequence = |length: usize, start: i32| MonomorphicValue::Sequence {
                type_id: sequence_id.clone(),
                elements: (0..length)
                    .map(|i| MonomorphicValue::Signed {
                        type_id: ty("i32"),
                        value: (start + i as i32).to_string(),
                    })
                    .collect(),
            };
            let encoded = |value: &MonomorphicValue| {
                let bits = relation_tests::storage(value, &types);
                sparse_cube(
                    bits.len().trailing_zeros(),
                    bits.into_iter()
                        .enumerate()
                        .filter_map(|(i, v)| v.then_some(i))
                        .collect(),
                )
            };
            for length in [0, 1, 255, 256, 257, 4096] {
                let value = sequence(length, 1);
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &value,
                )
                .unwrap();
                let input = encoded(&value);
                assert_eq!(
                    fail(&c, invalid, "empty_errors", std::slice::from_ref(&input)),
                    length == 0
                );
                assert_eq!(
                    fail(
                        &c,
                        invalid,
                        "validation_bound",
                        std::slice::from_ref(&input)
                    ),
                    length > 256
                );
                assert_eq!(
                    model.construct("invalid", Some(value.clone())).is_ok(),
                    (1..=256).contains(&length)
                );
                if (1..=256).contains(&length) {
                    // Check the constructor at both bounds; the append test below
                    // also checks every nonzero expected output leaf and neighbors.
                    let expected = model.construct("invalid", Some(value)).unwrap();
                    observe(
                        &c,
                        run(&c, &invalid.normal_definition, vec![input]),
                        &relation_tests::storage(&expected, &types),
                    );
                }
            }
            for (left_length, right_length) in [
                (0, 0),
                (0, 3),
                (3, 0),
                (2, 3),
                (3, 2),
                (1, 255),
                (255, 1),
                (128, 128),
                (256, 0),
                (256, 1),
                (4096, 0),
            ] {
                eprintln!("validation append: {left_length} + {right_length}");
                let left = sequence(left_length, 1);
                let right = sequence(right_length, 1001);
                let arguments = vec![encoded(&left), encoded(&right)];
                let exceeds = left_length + right_length > 256;
                assert_eq!(fail(&c, append, "validation_bound", &arguments), exceeds);
                if exceeds {
                    continue;
                }
                let MonomorphicValue::Sequence {
                    elements: mut joined,
                    ..
                } = left.clone()
                else {
                    panic!()
                };
                let MonomorphicValue::Sequence { elements, .. } = right.clone() else {
                    panic!()
                };
                joined.extend(elements);
                let expected = MonomorphicValue::Sequence {
                    type_id: sequence_id.clone(),
                    elements: joined,
                };
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &expected,
                )
                .unwrap();
                if left_length > 0 && right_length > 0 {
                    let lhs = model.construct("invalid", Some(left)).unwrap();
                    let rhs = model.construct("invalid", Some(right)).unwrap();
                    let model_result = model.append_errors(&lhs, &rhs).unwrap();
                    assert_eq!(model.read(&model_result, "invalid").unwrap(), &expected);
                }
                observe(
                    &c,
                    run(&c, &append.normal_definition, arguments),
                    &relation_tests::storage(&expected, &types),
                );
            }
            // Arbitrary invalid lengths must not wrap into a small accepted sum.
            let depth = types[sequence_id].depth;
            let malformed = |length: u32| {
                sparse_cube(
                    depth,
                    (0..32)
                        .filter(|i| length & (1 << i) != 0)
                        .map(|i| i << (depth - 5))
                        .collect(),
                )
            };
            for (a, z) in [
                (u32::MAX, 1),
                (u32::MAX, u32::MAX),
                (1 << 31, 1 << 31),
                (65536, 0),
            ] {
                assert!(fail(
                    &c,
                    append,
                    "validation_bound",
                    &[malformed(a), malformed(z)]
                ));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_validation_append_bool_and_product() {
    let bundle = b();
    let mut contexts = 0;
    for (id, row, facts) in extra_sources()
        .into_iter()
        .filter(|(id, _, _)| id.contains("validation"))
    {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_outcomes(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let d = p
            .definitions()
            .iter()
            .find(|d| d.template_id == "mpk.csharp.semantic.validation.v1")
            .unwrap();
        let append = op(d, "append_errors");
        let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let OrdinaryShape::Sequence { element, .. } = &types[&append.result_type_id].shape else {
            panic!()
        };
        let model = OutcomeModel::new(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &d.carrier.type_id,
        )
        .unwrap();
        let sequence = |length: usize, seed: usize| MonomorphicValue::Sequence {
            type_id: append.result_type_id.clone(),
            elements: (0..length)
                .map(|i| {
                    relation_tests::sample(
                        child_id(element),
                        seed + i,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    )
                })
                .collect(),
        };
        let encode = |value: &MonomorphicValue| {
            let bits = relation_tests::storage(value, &types);
            sparse_cube(
                bits.len().trailing_zeros(),
                bits.into_iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.then_some(i))
                    .collect(),
            )
        };
        for (a, z) in [(0, 2), (2, 0), (2, 3), (3, 2)] {
            let left = sequence(a, 0);
            let right = sequence(z, 1);
            for v in [&left, &right] {
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    v,
                )
                .unwrap();
            }
            let mut joined = match &left {
                MonomorphicValue::Sequence { elements, .. } => elements.clone(),
                _ => panic!(),
            };
            if let MonomorphicValue::Sequence { elements, .. } = &right {
                joined.extend(elements.clone());
            }
            let expected = MonomorphicValue::Sequence {
                type_id: append.result_type_id.clone(),
                elements: joined,
            };
            if a > 0 && z > 0 {
                let lhs = model.construct("invalid", Some(left.clone())).unwrap();
                let rhs = model.construct("invalid", Some(right.clone())).unwrap();
                let oracle = model.append_errors(&lhs, &rhs).unwrap();
                assert_eq!(model.read(&oracle, "invalid").unwrap(), &expected);
                let reversed = model.append_errors(&rhs, &lhs).unwrap();
                assert_ne!(
                    model.read(&reversed, "invalid").unwrap(),
                    &expected,
                    "{id}: order must be observable"
                );
            }
            let arguments = vec![encode(&left), encode(&right)];
            assert!(!fail(&c, append, "validation_bound", &arguments));
            observe(
                &c,
                run(&c, &append.normal_definition, arguments),
                &relation_tests::storage(&expected, &types),
            );
        }
        contexts += 1;
    }
    assert_eq!(contexts, 2);
}
