use super::*;

fn guard_symbol(s: &str) -> bool {
    [
        "SourceTag",
        "SemanticArm",
        "Payload",
        "MemberProjection",
        "Bound",
        "NonemptyInvalid",
    ]
    .iter()
    .any(|k| s.starts_with(&format!("Mpk.CSharp.Binding.{k}.")))
}
#[test]
fn csharp_03_t06_w09_binding_guards_original_source_certificates() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BINDING_GUARDS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-guards");
    let mut rows = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut demanded = 0;
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
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_binding_guards(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let base = generate_csharp_practical_ordinary_binding_relations(vir).unwrap();
        assert_eq!(p.projections(), base.projections());
        assert_eq!(p.agreements(), base.agreements());
        for d in base.predicates() {
            assert!(p.predicates().contains(d));
        }
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        for symbol in vc.definition_names().iter().filter(|s| guard_symbol(s)) {
            let d = p
                .predicates()
                .iter()
                .find(|d| &d.symbol == symbol)
                .unwrap_or_else(|| panic!("{id}: missing {symbol}"));
            assert_eq!(d.result_type_id, ty("bool"));
            assert!(!p.unresolved_vc_symbols().contains(symbol));
            demanded += 1;
        }
        let newly = p
            .predicates()
            .iter()
            .filter(|d| !base.predicates().contains(d))
            .map(|d| &d.symbol)
            .collect::<BTreeSet<_>>();
        assert!(newly.iter().all(|s| guard_symbol(s)));
        assert_eq!(
            p.unresolved_vc_symbols(),
            base.unresolved_vc_symbols()
                .iter()
                .filter(|s| !newly.contains(s))
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            import_csharp_practical_ordinary_binding_guards(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        assert!(import_csharp_practical_ordinary_binding_relations(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let mut mutated = meta.clone();
        mutated["predicates"] = json!([]);
        assert!(import_csharp_practical_ordinary_binding_guards(
            &serde_json::to_vec(&mutated).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_binding_guards(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_guards(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let old_cert = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
        let roots = base
            .predicates()
            .iter()
            .map(|p| p.definition.clone())
            .chain(
                base.projections()
                    .iter()
                    .map(|p| p.project_definition.clone()),
            )
            .chain(
                base.agreements()
                    .iter()
                    .map(|p| p.result_agreement_definition.clone()),
            )
            .collect();
        structural_equivalence_tests::same_definition_closure(&old_cert, &cert, &roots).unwrap();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!(
            "binding guards {id}: {} predicates; {} pending",
            p.predicates().len(),
            p.unresolved_vc_symbols().len()
        );
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(rows.len(), 45);
    assert!(demanded > 100);
    let data = json!({"sources":rows,"guard_symbol_occurrences":demanded});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-guards/certificates.json"),
            data
        );
    }
}

fn sum_value(v: &MonomorphicValue) -> Option<(&str, Vec<&MonomorphicValue>)> {
    match v {
        MonomorphicValue::Option { arm, value, .. } => Some((
            if *arm == OptionArm::None {
                "none"
            } else {
                "some"
            },
            value.iter().map(|v| v.as_ref()).collect(),
        )),
        MonomorphicValue::BoundaryPresence { arm, value, .. } => Some((
            match arm {
                BoundaryArm::Missing => "missing",
                BoundaryArm::Null => "null",
                BoundaryArm::Value => "value",
            },
            value.iter().map(|v| v.as_ref()).collect(),
        )),
        MonomorphicValue::TaggedSum { arm, payload, .. } => {
            Some((arm.as_str(), payload.iter().collect()))
        }
        _ => None,
    }
}
// Independent value observation: IEEE bits stay exact; decimal scale cohorts
// denote the same decimal value. This oracle does not evaluate generated terms.
fn observed(value: &impl serde::Serialize) -> Value {
    fn normalize(v: &mut Value) {
        match v {
            Value::Array(values) => {
                for value in values {
                    normalize(value);
                }
            }
            Value::Object(fields) => {
                if fields.get("kind").and_then(Value::as_str) == Some("decimal_bits") {
                    let mut coefficient = fields["coefficient"]
                        .as_str()
                        .unwrap()
                        .parse::<u128>()
                        .unwrap();
                    let mut scale = fields["scale"].as_u64().unwrap();
                    if coefficient == 0 {
                        scale = 0;
                        fields.insert("negative".into(), json!(false));
                    } else {
                        while scale > 0 && coefficient % 10 == 0 {
                            coefficient /= 10;
                            scale -= 1;
                        }
                    }
                    fields.insert("coefficient".into(), json!(coefficient.to_string()));
                    fields.insert("scale".into(), json!(scale));
                } else {
                    for value in fields.values_mut() {
                        normalize(value);
                    }
                }
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(value).unwrap();
    normalize(&mut v);
    v
}
fn expected_member(left: &MonomorphicValue, right: &MonomorphicValue, role: &str) -> bool {
    if let Some((la, lv)) = sum_value(left) {
        let (ra, rv) = sum_value(right).unwrap();
        let active_role = match la {
            "invalid" => "errors",
            "error" => "error",
            _ => "value",
        };
        return la == ra
            && (role == "tag" || role != active_role || observed(&lv) == observed(&rv));
    }
    match (left, right) {
        (
            MonomorphicValue::OrderedEntry {
                key: a, value: b, ..
            },
            MonomorphicValue::OrderedEntry {
                key: c, value: d, ..
            },
        ) => {
            if role == "key" {
                observed(a) == observed(c)
            } else {
                assert_eq!(role, "value");
                observed(b) == observed(d)
            }
        }
        (
            MonomorphicValue::Money {
                amount: a,
                currency: b,
                ..
            },
            MonomorphicValue::Money {
                amount: c,
                currency: d,
                ..
            },
        ) => {
            if role == "amount" {
                observed(a) == observed(c)
            } else {
                assert_eq!(role, "currency");
                observed(b) == observed(d)
            }
        }
        (
            MonomorphicValue::Transition {
                state: a,
                events: b,
                response: c,
                ..
            },
            MonomorphicValue::Transition {
                state: d,
                events: e,
                response: f,
                ..
            },
        ) => match role {
            "state" => observed(a) == observed(d),
            "events" => observed(b) == observed(e),
            "response" => observed(c) == observed(f),
            _ => panic!("unknown transition role {role}"),
        },
        _ => observed(left) == observed(right),
    }
}
#[test]
fn csharp_03_t06_w09_binding_guards_source_tags_and_members() {
    let bundle = b();
    let mut count = 0;
    let mut roles = BTreeSet::new();
    let mut saw_min_tag = false;
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
        let p = generate_csharp_practical_ordinary_binding_guards(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = ProjectionOracle {
            facts: &facts,
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for def in p
            .projections()
            .iter()
            .filter(|p| p.source_carrier.depth <= 10 && p.semantic_carrier.depth <= 10)
        {
            let Some(binding) = oracle
                .bindings
                .iter()
                .find(|b| b["source_type_id"] == def.source_carrier.type_id)
            else {
                continue;
            };
            roles.insert(binding["role"].as_str().unwrap().to_owned());
            let mut values = (0..4)
                .map(|seed| {
                    relation_tests::sample(
                        &def.source_carrier.type_id,
                        seed,
                        &types,
                        &facts,
                        oracle.closed,
                    )
                })
                .collect::<Vec<_>>();
            if id == "bool-float-entry" {
                for bits in ["80000000", "00000000", "7fc00000", "7fc00001", "7f800001"] {
                    let mut value = values[0].clone();
                    let MonomorphicValue::Product { fields, .. } = &mut value else {
                        panic!()
                    };
                    *fields.iter_mut().find(|f| f.name == "Value").unwrap().value =
                        MonomorphicValue::F32Bits {
                            type_id: ty("f32"),
                            bits: bits.into(),
                        };
                    values.push(value);
                }
            }
            if binding["member_map"].get("tag").is_some() {
                let mut unknown = values[0].clone();
                let original = facts["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == def.source_carrier.type_id)
                    .unwrap();
                let tag_member = original["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|m| {
                        csharp_practical_stored_member_id(
                            &def.source_carrier.type_id,
                            m["name"].as_str().unwrap(),
                            &m["type"],
                            m["storage"].as_str().unwrap(),
                        )
                        .unwrap()
                            == binding["member_map"]["tag"].as_str().unwrap()
                    })
                    .unwrap();
                let MonomorphicValue::Product { fields, .. } = &mut unknown else {
                    panic!()
                };
                let MonomorphicValue::Enum { carrier, .. } = fields
                    .iter_mut()
                    .find(|f| f.name == tag_member["name"].as_str().unwrap())
                    .unwrap()
                    .value
                    .as_mut()
                else {
                    panic!()
                };
                let absent = (0..128)
                    .map(|n| n.to_string())
                    .find(|n| {
                        !binding["tag_arms"]
                            .as_object()
                            .unwrap()
                            .values()
                            .any(|v| v.as_str() == Some(n.as_str()))
                    })
                    .unwrap();
                *carrier = absent;
                let (sd, sbits) = sparse_storage(&unknown, &types);
                let projected = oracle.project(&values[0], &def.semantic_carrier.type_id);
                let (td, tbits) = sparse_storage(&projected, &types);
                for pred in p.predicates().iter().filter(|p| {
                    p.symbol.starts_with(&format!(
                        "Mpk.CSharp.Binding.SourceTag.{}.",
                        def.projection.id
                    )) || p.symbol.starts_with(&format!(
                        "Mpk.CSharp.Binding.Payload.{}.",
                        def.projection.id
                    )) || p.symbol.starts_with(&format!(
                        "Mpk.CSharp.Binding.MemberProjection.{}.",
                        def.projection.id
                    ))
                }) {
                    let mut args = vec![sparse_cube(sd, sbits.clone())];
                    if pred.argument_type_ids.len() == 2 {
                        args.push(sparse_cube(td, tbits.clone()));
                    }
                    assert!(
                        !bit(run(&cert, &pred.definition, args)),
                        "{id}: unknown source tag: {}",
                        pred.symbol
                    );
                    count += 1;
                }
            }
            for value in &values {
                let projected = oracle.project(value, &def.semantic_carrier.type_id);
                for other in &values {
                    let y = oracle.project(other, &def.semantic_carrier.type_id);
                    for pred in p.predicates().iter().filter(|p| {
                        guard_symbol(&p.symbol)
                            && p.argument_type_ids
                                == [
                                    def.source_carrier.type_id.clone(),
                                    def.semantic_carrier.type_id.clone(),
                                ]
                    }) {
                        let expected = if let Some(arm) = pred.symbol.strip_prefix(&format!(
                            "Mpk.CSharp.Binding.Payload.{}.",
                            def.projection.id
                        )) {
                            let (actual, payload) = sum_value(&projected).unwrap();
                            let (right, rpayload) = sum_value(&y).unwrap();
                            actual == arm
                                && right == arm
                                && observed(&payload) == observed(&rpayload)
                        } else {
                            expected_member(&projected, &y, pred.symbol.rsplit('.').next().unwrap())
                        };
                        let (sd, sbits) = sparse_storage(value, &types);
                        let (td, tbits) = sparse_storage(&y, &types);
                        assert_eq!(
                            bit(run(
                                &cert,
                                &pred.definition,
                                vec![sparse_cube(sd, sbits), sparse_cube(td, tbits)]
                            )),
                            expected,
                            "{id}: {}\n{value:?}\n{y:?}",
                            pred.symbol
                        );
                        count += 1;
                    }
                }
                if let Some((arm, _)) = sum_value(&projected) {
                    let expected_source = binding["tag_arms"][arm].as_str().unwrap();
                    saw_min_tag |= expected_source == "-9223372036854775808";
                    for pred in p.predicates().iter().filter(|p| {
                        p.symbol.starts_with(&format!(
                            "Mpk.CSharp.Binding.SourceTag.{}.",
                            def.projection.id
                        ))
                    }) {
                        let (d, bits) = sparse_storage(value, &types);
                        assert_eq!(
                            bit(run(&cert, &pred.definition, vec![sparse_cube(d, bits)])),
                            pred.symbol.rsplit('.').next().unwrap() == expected_source,
                            "{id}: {}",
                            pred.symbol
                        );
                        count += 1;
                    }
                    for pred in p.predicates().iter().filter(|p| {
                        p.symbol.starts_with(&format!(
                            "Mpk.CSharp.Binding.SemanticArm.{}.",
                            def.semantic_carrier.type_id
                        ))
                    }) {
                        let (d, bits) = sparse_storage(&projected, &types);
                        assert_eq!(
                            bit(run(&cert, &pred.definition, vec![sparse_cube(d, bits)])),
                            pred.symbol.rsplit('.').next().unwrap() == arm
                        );
                        count += 1;
                    }
                }
            }
        }
    }
    assert!(saw_min_tag);
    for role in [
        "boundary_field",
        "option",
        "lookup",
        "result",
        "ordered_entry",
        "instant",
    ] {
        assert!(roles.contains(role), "{roles:?}");
    }
    eprintln!("binding guard source observations: {count}, roles: {roles:?}");
    assert!(count > 300);
}

#[test]
fn csharp_03_t06_w09_binding_guards_bounds_and_invalid_tags() {
    let bundle = b();
    let mut count = 0;
    let mut roles = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_binding_guards(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        for def in p.projections() {
            let Some(binding) = bindings["bindings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|b| b["source_type_id"] == def.source_carrier.type_id)
            else {
                continue;
            };
            let target = &def.semantic_carrier;
            let members = p
                .predicates()
                .iter()
                .filter(|p| {
                    p.symbol.starts_with(&format!(
                        "Mpk.CSharp.Binding.MemberProjection.{}.",
                        def.projection.id
                    ))
                })
                .collect::<Vec<_>>();
            if let OrdinaryShape::Sum { arms } = &target.shape {
                // No admitted semantic arm has all 32 tag bits set. Preserve a
                // zero payload while forging the tag outside the domain.
                let bits = (0..32)
                    .map(|i| i << (target.depth - 5))
                    .collect::<BTreeSet<_>>();
                for pred in p.predicates().iter().filter(|p| {
                    p.symbol.starts_with(&format!(
                        "Mpk.CSharp.Binding.SemanticArm.{}.",
                        target.type_id
                    ))
                }) {
                    assert!(!bit(run(
                        &cert,
                        &pred.definition,
                        vec![sparse_cube(target.depth, bits.clone())]
                    )));
                    count += 1;
                }
                for pred in &members {
                    assert!(!bit(run(
                        &cert,
                        &pred.definition,
                        vec![
                            sparse_cube(def.source_carrier.depth, BTreeSet::new()),
                            sparse_cube(target.depth, bits.clone())
                        ]
                    )));
                    count += 1;
                }
                assert!(arms.iter().all(|a| a.tag != u32::MAX));
            }
            for pred in p.predicates().iter().filter(|p| {
                p.argument_type_ids == [target.type_id.clone()]
                    && (p
                        .symbol
                        .starts_with(&format!("Mpk.CSharp.Binding.Bound.{}.", def.projection.id))
                        || p.symbol
                            == format!("Mpk.CSharp.Binding.NonemptyInvalid.{}", target.type_id))
            }) {
                let role = binding["role"].as_str().unwrap();
                roles.insert(role.to_owned());
                let nonempty = pred.symbol.contains(".NonemptyInvalid.");
                let max = if nonempty {
                    0
                } else {
                    pred.symbol
                        .rsplit('.')
                        .next()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap()
                };
                let (prefix, tags) = match &target.shape {
                    OrdinaryShape::Sequence { .. } => (0, vec![(None, true)]),
                    OrdinaryShape::Product { fields } => {
                        assert_eq!(role, "transition");
                        (
                            fields.iter().position(|f| f.id == "events").unwrap(),
                            vec![(None, true)],
                        )
                    }
                    OrdinaryShape::Sum { arms } => {
                        assert_eq!(role, "validation");
                        (
                            1,
                            vec![
                                (
                                    Some(arms.iter().find(|a| a.id == "invalid").unwrap().tag),
                                    true,
                                ),
                                (
                                    Some(arms.iter().find(|a| a.id == "valid").unwrap().tag),
                                    false,
                                ),
                                (Some(u32::MAX), false),
                            ],
                        )
                    }
                    _ => panic!("{id}: unexpected bound carrier"),
                };
                for (tag, uses_length) in tags {
                    for len in [0u32, 1, 255, 256, 257, 4095, 4096, 4097, u32::MAX] {
                        let mut bits = (0..32)
                            .filter(|i| len & (1u32 << i) != 0)
                            .map(|i| prefix | (i << (target.depth - 5)))
                            .collect::<BTreeSet<_>>();
                        if let Some(tag) = tag {
                            bits.extend(
                                (0..32)
                                    .filter(|i| tag & (1u32 << i) != 0)
                                    .map(|i| i << (target.depth - 5)),
                            );
                        }
                        let expected = if uses_length {
                            if nonempty {
                                len > 0
                            } else {
                                len <= max
                            }
                        } else {
                            tag != Some(u32::MAX)
                        };
                        assert_eq!(
                            bit(run(
                                &cert,
                                &pred.definition,
                                vec![sparse_cube(target.depth, bits)]
                            )),
                            expected,
                            "{id} {} tag={tag:?} length={len}",
                            pred.symbol
                        );
                        count += 1;
                    }
                }
            }
        }
        // The test consumes only carriers independently regenerated from VIR.
        assert_eq!(
            layouts
                .carriers()
                .iter()
                .map(|c| &c.type_id)
                .collect::<BTreeSet<_>>()
                .len(),
            layouts.carriers().len()
        );
    }
    assert_eq!(
        roles,
        [
            "bounded_sequence",
            "ordered_map",
            "ordered_set",
            "transition",
            "validation"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    assert!(count > 200);
    eprintln!("binding bound and invalid-tag observations: {count}");
}

#[test]
fn csharp_03_t06_w09_binding_guards_deep_member_observations() {
    let bundle = b();
    let mut count = 0;
    let mut roles = BTreeSet::new();
    for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
        matches!(
            id.as_str(),
            "binding-vc-money"
                | "binding-vc-validation"
                | "binding-vc-transition"
                | "remapped-boundary-sequence"
                | "binding-vc-ordered_map"
                | "binding-vc-ordered_set"
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
        let p = generate_csharp_practical_ordinary_binding_guards(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = ProjectionOracle {
            facts: &facts,
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for def in p
            .projections()
            .iter()
            .filter(|d| d.semantic_carrier.depth > 10)
        {
            let binding = oracle
                .bindings
                .iter()
                .find(|b| b["source_type_id"] == def.source_carrier.type_id)
                .unwrap();
            roles.insert(binding["role"].as_str().unwrap().to_owned());
            // Each side is obtained independently from captured fields and the
            // frozen role map; no generated projection supplies expected data.
            for seed in [0, 1] {
                let x = relation_tests::sample(
                    &def.source_carrier.type_id,
                    seed,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let px = oracle.project(&x, &def.semantic_carrier.type_id);
                for other in [seed, (seed + 1) % 2] {
                    let source_y = relation_tests::sample(
                        &def.source_carrier.type_id,
                        other,
                        &types,
                        &facts,
                        oracle.closed,
                    );
                    let y = oracle.project(&source_y, &def.semantic_carrier.type_id);
                    for pred in p.predicates().iter().filter(|p| {
                        p.symbol.starts_with(&format!(
                            "Mpk.CSharp.Binding.MemberProjection.{}.",
                            def.projection.id
                        )) || p.symbol.starts_with(&format!(
                            "Mpk.CSharp.Binding.Payload.{}.",
                            def.projection.id
                        ))
                    }) {
                        let expected = if let Some(arm) = pred.symbol.strip_prefix(&format!(
                            "Mpk.CSharp.Binding.Payload.{}.",
                            def.projection.id
                        )) {
                            let (left, lp) = sum_value(&px).unwrap();
                            let (right, rp) = sum_value(&y).unwrap();
                            left == arm && right == arm && observed(&lp) == observed(&rp)
                        } else {
                            expected_member(&px, &y, pred.symbol.rsplit('.').next().unwrap())
                        };
                        let (sd, sbits) = sparse_storage(&x, &types);
                        let (td, tbits) = sparse_storage(&y, &types);
                        eprintln!("deep binding guard {id} seed={seed} other={other} {} expected={expected}",pred.symbol);
                        assert_eq!(
                            bit(run(
                                &cert,
                                &pred.definition,
                                vec![sparse_cube(sd, sbits), sparse_cube(td, tbits)]
                            )),
                            expected,
                            "{id}: {}",
                            pred.symbol
                        );
                        count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        roles,
        [
            "money",
            "validation",
            "transition",
            "bounded_sequence",
            "ordered_map",
            "ordered_set"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    assert!(count > 40);
    eprintln!("deep binding guard observations: {count}");
}

#[path = "csharp_practical_ordinary_binding_order_tests.rs"]
mod order_tests;
