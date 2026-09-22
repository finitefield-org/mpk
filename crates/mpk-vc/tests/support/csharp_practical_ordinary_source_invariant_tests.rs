use super::*;
use core_eval::{bit, run, sparse_cube, V};
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

// Check exact beta/eta equivalence to the independent stored-member projection.
// Only the final selector lambdas may disappear. Every fixed prefix, source
// type, target type and dependency must still agree in the two certificates.
fn same_direct_member(
    old: &Certificate,
    new: &Certificate,
    old_member: &OrdinaryProjection,
    new_member: &OrdinaryProjection,
) {
    assert_eq!(old_member.field_id, new_member.field_id);
    assert_eq!(old_member.depth, new_member.depth);
    let mut normalized = old.clone();
    let index = old
        .declarations
        .iter()
        .position(|d| old.name_table[d.name as usize] == old_member.definition)
        .unwrap();
    let DeclarationKind::Def { value, .. } = old.declarations[index].kind else {
        panic!()
    };
    let TermNode::Lam { ty, mut body } = old.term_table[value as usize] else {
        panic!()
    };
    for _ in 0..old_member.depth {
        let TermNode::Lam { body: next, .. } = old.term_table[body as usize] else {
            panic!("missing selector lambda")
        };
        body = next;
    }
    let arguments = match &old.term_table[body as usize] {
        TermNode::App {
            function,
            arguments,
        } => {
            assert_eq!(
                old.term_table[*function as usize],
                TermNode::Var(old_member.depth)
            );
            arguments.clone()
        }
        TermNode::Var(index) if *index == old_member.depth => vec![],
        _ => panic!("not a stored-member projection"),
    };
    let prefix_len = arguments
        .len()
        .checked_sub(old_member.depth as usize)
        .unwrap();
    for (i, arg) in arguments[prefix_len..].iter().enumerate() {
        assert_eq!(
            old.term_table[*arg as usize],
            TermNode::Var(old_member.depth - 1 - i as u32)
        );
    }
    let prefix = arguments[..prefix_len].to_vec();
    for arg in &prefix {
        let TermNode::Const { ref levels, .. } = old.term_table[*arg as usize] else {
            panic!("nonconstant field prefix")
        };
        assert!(levels.is_empty());
        // Evaluate the closed prefix leaf, never a supplied field result.
        bit(core_eval::eval(old, *arg, &[]));
    }
    let receiver = normalized.term_table.len() as u32;
    normalized.term_table.push(TermNode::Var(0));
    let body = if prefix.is_empty() {
        receiver
    } else {
        let body = normalized.term_table.len() as u32;
        normalized.term_table.push(TermNode::App {
            function: receiver,
            arguments: prefix,
        });
        body
    };
    let value = normalized.term_table.len() as u32;
    normalized.term_table.push(TermNode::Lam { ty, body });
    let DeclarationKind::Def {
        value: old_value, ..
    } = &mut normalized.declarations[index].kind
    else {
        panic!()
    };
    *old_value = value;
    assert!(!normalized.name_table.contains(&new_member.definition));
    normalized.name_table[old.declarations[index].name as usize] = new_member.definition.clone();
    structural_equivalence_tests::same_definition_closure(
        &normalized,
        new,
        &BTreeSet::from([new_member.definition.clone()]),
    )
    .unwrap();
}

#[test]
fn csharp_03_t06_w09_source_invariants_direct_member_selectors() {
    let bundle = b();
    let mut count = 0;
    let mut deep = 0;
    for (id, row, facts) in invariant_sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_source_invariants(emitted.vir()).unwrap();
        let storage = generate_csharp_practical_ordinary_structural(emitted.vir()).unwrap();
        let old = mpk_cert::decode_canonical_certificate(storage.certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for d in p
            .definitions()
            .iter()
            .filter(|d| !d.member_reads.is_empty())
        {
            let source = storage
                .definitions()
                .iter()
                .find(|s| s.carrier.type_id == d.invariant.type_id)
                .unwrap();
            let OrdinaryStructuralOperations::Product { operations } = &source.operations else {
                panic!()
            };
            for member in &d.member_reads {
                let old_member = operations
                    .fields
                    .iter()
                    .find(|m| m.field_id == member.field_id)
                    .unwrap();
                same_direct_member(&old, &cert, old_member, member);
                for value in [false, true] {
                    let input = if source.carrier.depth == 0 {
                        V::Bit(value)
                    } else {
                        V::UniformCube(value, source.carrier.depth)
                    };
                    let output = run(&cert, &member.definition, vec![input]);
                    match output {
                        V::UniformCube(observed, depth) => {
                            assert_eq!(observed, value);
                            assert_eq!(depth, member.depth);
                        }
                        V::Bit(observed) if member.depth == 0 => assert_eq!(observed, value),
                        _ => {
                            panic!("{id}: direct member must preserve a complete constant subcube")
                        }
                    }
                }
                count += 1;
                deep += usize::from(member.depth >= 30);
            }
        }
    }
    assert!(count > 0 && deep > 0);
    eprintln!("source invariant direct members: {count} exact beta/eta comparisons and constant-subcube checks, {deep} deep members");
}

fn invariant_sources() -> Vec<(String, Value, Value)> {
    let mut rows = sources();
    for family in [
        "construction-vc",
        "ordinary-foundation/public-domain-sources",
        "ordinary-foundation/conditional-clauses",
        "ordinary-foundation/structural-clauses",
    ] {
        let requests = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        for row in requests.as_array().unwrap() {
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == row["id"])
                .unwrap();
            if response.get("reject").is_some() {
                continue;
            }
            let id = format!(
                "{}-{}",
                family.split('/').next_back().unwrap(),
                row["id"].as_str().unwrap()
            );
            if !rows.iter().any(|(old, _, _)| old == &id) {
                rows.push((id, row.clone(), response["facts"].clone()));
            }
        }
    }
    rows
}

#[test]
fn csharp_03_t06_w09_source_invariants_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_SOURCE_INVARIANTS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/source-invariants");
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut conditions = 0;
    let mut definitions = 0;
    for (id, row, facts) in invariant_sources() {
        eprintln!("source invariant definitions start {id}");
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
        let p = generate_csharp_practical_ordinary_source_invariants(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.invariant)
                .collect::<Vec<_>>(),
            full.construction_vcs().types().iter().collect::<Vec<_>>()
        );
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind == "source_invariant")
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind != "source_invariant")
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_proof_ids(),
            vc.sequents()
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        conditions += p.conditions().len();
        definitions += p.definitions().len();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let representation = generate_csharp_practical_ordinary_domains(vir).unwrap();
        let public = generate_csharp_practical_ordinary_public_domains(vir).unwrap();
        assert_eq!(p.representation_domains(), representation.definitions());
        assert_eq!(p.public_domains(), public.definitions());
        for (bytes, roots) in [
            (
                representation.certificate_bytes(),
                representation
                    .definitions()
                    .iter()
                    .map(|d| d.valid_definition.clone())
                    .collect(),
            ),
            (
                public.certificate_bytes(),
                public
                    .definitions()
                    .iter()
                    .map(|d| d.valid_definition.clone())
                    .collect(),
            ),
        ] {
            let before = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        }
        for d in p.definitions() {
            assert_eq!(d.member_reads.len(), d.invariant.members.len());
            assert_eq!(d.clause_definitions.len(), d.invariant.public_clauses.len());
            assert_eq!(
                d.enum_cases
                    .iter()
                    .map(|c| c.value.clone())
                    .collect::<Vec<_>>(),
                d.invariant.enum_values.clone().unwrap_or_default()
            );
            let domain = p
                .public_domains()
                .iter()
                .find(|v| v.carrier.type_id == d.invariant.type_id)
                .unwrap();
            assert_ne!(d.body_definition, domain.valid_definition);
            assert_ne!(d.representation_domain, domain.valid_definition);
        }
        assert_eq!(
            import_csharp_practical_ordinary_source_invariants(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_source_invariants(meta, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "construction-vc-positive_constructor" {
            let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for key in meta.as_object().unwrap().keys() {
                let mut changed = meta.clone();
                changed[key] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_source_invariants(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err(),
                    "{key}"
                );
            }
            let mut changed = p.certificate_bytes().to_vec();
            *changed.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_source_invariants(
                &p.canonical_bytes(),
                &changed,
                vir
            )
            .is_err());
            let oversized = vec![0; 16 * 1024 * 1024 + 1];
            assert!(
                import_csharp_practical_ordinary_source_invariants(&oversized, &[], vir).is_err()
            );
            assert!(
                import_csharp_practical_ordinary_source_invariants(&[], &oversized, vir).is_err()
            );
        }
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!("source invariant definitions complete {id}");
    }
    assert_eq!(rows.len(), 53);
    assert_eq!(conditions, 35);
    let data = json!({"sources":rows,"conditions":conditions,"definitions":definitions});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/source-invariants/certificates.json"),
            data
        );
    }
}

// The independently compiled recursive public domain supplies a second
// implementation of the invariant equation. Its closure is checked above.
#[test]
fn csharp_03_t06_w09_source_invariants_original_equations() {
    let bundle = b();
    let mut observations = 0;
    let mut enum_rejections = 0;
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
        let p = generate_csharp_practical_ordinary_source_invariants(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = p
            .public_domains()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            let domain = p
                .public_domains()
                .iter()
                .find(|v| v.carrier.type_id == d.invariant.type_id)
                .unwrap();
            let mut values = (0..3)
                .map(|seed| {
                    let value = relation_tests::sample(
                        &d.invariant.type_id,
                        seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let (depth, ones) = projection_tests::sparse_storage(&value, &types);
                    sparse_cube(depth, ones)
                })
                .collect::<Vec<_>>();
            // A depth-zero carrier is a Bool leaf, not a function cube. Empty
            // source products still need a well-typed nonzero-padding input.
            values.push(if domain.carrier.depth == 0 {
                V::Bit(true)
            } else {
                V::UniformCube(true, domain.carrier.depth)
            });
            if let Some(arms) = &d.invariant.enum_values {
                let OrdinaryShape::Bits { width } = domain.carrier.shape else {
                    panic!()
                };
                let mut numbers = arms
                    .iter()
                    .map(|v| v.parse::<i128>().unwrap())
                    .collect::<Vec<_>>();
                numbers.extend([-1, 0, 1, 2, 3]);
                numbers.sort();
                numbers.dedup();
                let mask = (1u128 << width) - 1;
                for n in numbers {
                    let value = sparse_cube(
                        domain.carrier.depth,
                        (0..width)
                            .filter(|i| (n as u128) & (1u128 << i) != 0)
                            .map(|i| i as usize)
                            .collect(),
                    );
                    let expected = arms
                        .iter()
                        .any(|v| (v.parse::<i128>().unwrap() as u128) & mask == (n as u128) & mask);
                    assert_eq!(
                        bit(run(&cert, &d.body_definition, vec![value.clone()])),
                        expected,
                        "{id}:enum {n}"
                    );
                    for arm in &d.enum_cases {
                        assert_eq!(
                            bit(run(&cert, &arm.definition, vec![value.clone()])),
                            (arm.value.parse::<i128>().unwrap() as u128) & mask
                                == (n as u128) & mask
                        );
                    }
                    enum_rejections += usize::from(!expected);
                    values.push(value);
                }
            }
            for (i, value) in values.into_iter().enumerate() {
                let expected = bit(run(&cert, &domain.valid_definition, vec![value.clone()]));
                assert_eq!(
                    bit(run(&cert, &d.body_definition, vec![value.clone()])),
                    expected,
                    "{id}:{}:{i}",
                    d.invariant.type_id
                );
                for condition in p
                    .conditions()
                    .iter()
                    .filter(|c| c.sequent.subjects[0].type_id == d.invariant.type_id)
                {
                    assert!(condition.sequent.assumptions.is_empty());
                    assert!(
                        bit(run(
                            &cert,
                            &condition.condition_definition,
                            vec![value.clone()]
                        )),
                        "{id}:{}:{i}",
                        condition.sequent.id
                    );
                }
                observations += 1;
            }
            eprintln!(
                "source invariant observations {id}: {}",
                d.invariant.type_id
            );
        }
    }
    assert!(observations > 100);
    assert!(enum_rejections > 0);
    eprintln!("source invariant original equations: {observations} observations, {enum_rejections} enum rejection cases");
}

#[test]
fn csharp_03_t06_w09_source_invariants_nested_public_clauses() {
    let bundle = b();
    let mut count = 0;
    for (id, row, facts) in invariant_sources().into_iter().filter(|(id, _, _)| {
        [
            "public-domain-sources-",
            "conditional-clauses-",
            "structural-clauses-",
        ]
        .iter()
        .any(|prefix| id.starts_with(prefix))
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
        let p = generate_csharp_practical_ordinary_source_invariants(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = p
            .public_domains()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        let envelope = p
            .definitions()
            .iter()
            .find(|d| d.invariant.members.len() == 3)
            .unwrap();
        let template = relation_tests::sample(
            &envelope.invariant.type_id,
            0,
            &types,
            &facts,
            emitted.closure().closed(),
        );
        let MonomorphicValue::Product { fields, .. } = &template else {
            panic!()
        };
        let direct = fields
            .iter()
            .find(|f| f.name == "Direct")
            .unwrap()
            .value
            .as_ref();
        let inner = p
            .definitions()
            .iter()
            .find(|d| d.invariant.type_id == direct.type_id())
            .unwrap();
        let positive = |n: i32| {
            let mut v = direct.clone();
            let MonomorphicValue::Product { fields, .. } = &mut v else {
                panic!()
            };
            let MonomorphicValue::Signed { value, .. } = fields[0].value.as_mut() else {
                panic!()
            };
            *value = n.to_string();
            v
        };
        let expected_clause = |n: i32| {
            if id.starts_with("public-domain-sources-") {
                n > 0
            } else if id.starts_with("conditional-clauses-") {
                n == 1 || n == -1
            } else {
                n == 1 || n < 0
            }
        };
        for n in [-2, -1, 0, 1, 2] {
            let (depth, ones) = projection_tests::sparse_storage(&positive(n), &types);
            let value = sparse_cube(depth, ones);
            assert!(bit(run(
                &cert,
                &inner.representation_domain,
                vec![value.clone()]
            )));
            assert_eq!(
                bit(run(&cert, &inner.body_definition, vec![value])),
                expected_clause(n),
                "{id}:{n}"
            );
            count += 1;
        }
        for (direct, items, optional, bad_inactive) in [
            (1, vec![], None, false),
            (0, vec![], None, false),
            (1, vec![1], None, false),
            (1, vec![0], None, false),
            (1, vec![1, -1], None, false),
            (1, vec![], Some(1), false),
            (1, vec![], Some(0), false),
            (1, vec![], None, true),
        ] {
            let mut v = template.clone();
            let MonomorphicValue::Product { fields, .. } = &mut v else {
                panic!()
            };
            for field in fields {
                match field.name.as_str() {
                    "Direct" => *field.value = positive(direct),
                    "Items" => {
                        let MonomorphicValue::Sequence { elements, .. } = field.value.as_mut()
                        else {
                            panic!()
                        };
                        *elements = items.iter().map(|n| positive(*n)).collect();
                    }
                    "Optional" => {
                        let MonomorphicValue::Option { arm, value, .. } = field.value.as_mut()
                        else {
                            panic!()
                        };
                        *arm = if optional.is_some() {
                            OptionArm::Some
                        } else {
                            OptionArm::None
                        };
                        *value = optional
                            .or(if bad_inactive { Some(1) } else { None })
                            .map(|n| Box::new(positive(n)));
                    }
                    _ => panic!(),
                }
            }
            let (depth, ones) = projection_tests::sparse_storage(&v, &types);
            let value = sparse_cube(depth, ones);
            assert_eq!(
                bit(run(
                    &cert,
                    &envelope.representation_domain,
                    vec![value.clone()]
                )),
                !bad_inactive
            );
            let expected = !bad_inactive
                && expected_clause(direct)
                && items.iter().all(|n| expected_clause(*n))
                && optional.is_none_or(expected_clause);
            assert_eq!(
                bit(run(&cert, &envelope.body_definition, vec![value.clone()])),
                expected,
                "{id}:{direct}:{items:?}:{optional:?}:{bad_inactive}"
            );
            let public = p
                .public_domains()
                .iter()
                .find(|d| d.carrier.type_id == envelope.invariant.type_id)
                .unwrap();
            assert_eq!(
                bit(run(&cert, &public.valid_definition, vec![value])),
                expected
            );
            count += 1;
        }
    }
    assert_eq!(count, 39);
    eprintln!("source invariant nested public clauses: {count} observations");
}

#[test]
fn csharp_03_t06_w09_source_invariants_preserve_partial_clause_obligations() {
    let bundle = b();
    let requests = read("ordinary-foundation/partial-read-clauses/requests.json");
    let responses = read("ordinary-foundation/partial-read-clauses/responses.json");
    let row = &requests[0];
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == row["id"])
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let clauses = generate_csharp_practical_ordinary_source_clauses(emitted.vir()).unwrap();
    assert!(clauses
        .definitions()
        .iter()
        .any(|c| c.definedness_definition.is_some()));
    assert_eq!(
        generate_csharp_practical_ordinary_source_invariants(emitted.vir()),
        Err(OrdinaryCarrierError::Linkage)
    );
}
