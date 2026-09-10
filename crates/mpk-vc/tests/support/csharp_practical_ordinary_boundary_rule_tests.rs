//! Original W07 contract rules, tested against source projection and typed values.
use super::*;
use core_eval::{bit, run, V};

fn sevens(value: &mut MonomorphicValue) {
    match value {
        MonomorphicValue::Signed { value, .. } => *value = "7".into(),
        MonomorphicValue::Product { fields, .. } => {
            for f in fields {
                sevens(&mut f.value);
            }
        }
        _ => (),
    }
}

#[test]
fn csharp_03_t06_w09_boundary_rules_original_sources() {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let accepted = read("boundary-vc/goldens.json");
    assert_eq!(accepted.as_array().unwrap().len(), 15);
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/boundary-rules");
    let output = std::env::var_os("MPK_W09_BOUNDARY_RULES_OUT").map(std::path::PathBuf::from);
    let mut rows = vec![];
    let mut observations = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for accepted in accepted.as_array().unwrap() {
        let id = accepted["id"].as_str().unwrap();
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, request);
        let facts = &response["facts"];
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_boundary_rules(&emitted)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_boundary_rules(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                &emitted
            )
            .unwrap(),
            p
        );
        for field in ["predicates", "values", "unresolved_boundary_vc_symbols"] {
            let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            bad[field] = json!(["mutated"]);
            assert!(import_csharp_practical_ordinary_boundary_rules(
                &serde_json::to_vec(&bad).unwrap(),
                p.certificate_bytes(),
                &emitted
            )
            .is_err());
        }
        if let Some((metadata, certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_boundary_rules(
                metadata,
                certificate,
                &emitted
            )
            .is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let old = generate_csharp_practical_ordinary_binding_relations(emitted.vir()).unwrap();
        for pred in old.predicates() {
            assert!(p.predicates().contains(pred));
        }
        let old_cert = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
        let names = old_cert
            .declarations
            .iter()
            .map(|d| old_cert.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&old_cert, &cert, &names).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = projection_tests::ProjectionOracle {
            facts,
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        let mut demanded = BTreeSet::new();
        for contract in emitted.boundaries() {
            let cid = contract
                .artifact()
                .value()
                .get("contract_sha256")
                .unwrap()
                .as_str()
                .unwrap();
            for field in contract.input_fields() {
                let st = field.source_type_id();
                let mt = emitted
                    .closure()
                    .projections()
                    .get(st)
                    .map(String::as_str)
                    .unwrap_or(st);
                let mut samples = (0..3)
                    .map(|seed| {
                        relation_tests::sample(st, seed, &types, facts, emitted.closure().closed())
                    })
                    .collect::<Vec<_>>();
                let mut extra = samples.clone();
                extra.iter_mut().for_each(sevens);
                samples.extend(extra);
                if let BoundaryMissingRule::FrozenDefault(value) = field.missing_rule() {
                    if st == mt {
                        samples.push(value.clone());
                        if let MonomorphicValue::DecimalBits {
                            type_id,
                            negative,
                            scale,
                            coefficient,
                        } = value
                        {
                            let n: u128 = coefficient.parse().unwrap();
                            if *scale > 0 && n.is_multiple_of(10) {
                                samples.push(MonomorphicValue::DecimalBits {
                                    type_id: type_id.clone(),
                                    negative: *negative,
                                    scale: scale - 1,
                                    coefficient: (n / 10).to_string(),
                                });
                            }
                        }
                    }
                }
                for value in samples {
                    let projected = oracle.project(&value, mt);
                    for kind in ["MissingRule", "NullRule"] {
                        let symbol = format!("Mpk.CSharp.Boundary.{kind}.{cid}.{}", field.id());
                        demanded.insert(symbol.clone());
                        let pred = p.predicates().iter().find(|d| d.symbol == symbol).unwrap();
                        assert_eq!(pred.argument_type_ids, [st]);
                        assert!(!p.unresolved_boundary_vc_symbols().contains(&symbol));
                        let expected = if kind == "NullRule" {
                            field.nullable()
                                && matches!(
                                    &projected,
                                    MonomorphicValue::Option {
                                        arm: OptionArm::None,
                                        ..
                                    } | MonomorphicValue::BoundaryPresence {
                                        arm: BoundaryArm::Null,
                                        ..
                                    }
                                )
                        } else if field.required() {
                            false
                        } else {
                            match field.missing_rule() {
                                BoundaryMissingRule::Reject => false,
                                BoundaryMissingRule::ExposeMissing => matches!(
                                    &projected,
                                    MonomorphicValue::BoundaryPresence {
                                        arm: BoundaryArm::Missing,
                                        ..
                                    }
                                ),
                                BoundaryMissingRule::FrozenDefault(default) => {
                                    generate_structural_program(
                                        &bundle,
                                        emitted.closure().roots(),
                                        emitted.closure().closed(),
                                        mt,
                                    )
                                    .unwrap()
                                    .structural_equal(&projected, default)
                                    .unwrap()
                                }
                            }
                        };
                        let bits = relation_tests::storage(&value, &types);
                        let input = if bits.len() == 1 {
                            V::Bit(bits[0])
                        } else {
                            V::Cube(bits)
                        };
                        assert_eq!(
                            bit(run(&cert, &pred.definition, vec![input])),
                            expected,
                            "{id} {symbol} {value:?}"
                        );
                        observations += 1;
                    }
                }
            }
        }
        let newly = p
            .predicates()
            .iter()
            .filter(|p| !old.predicates().contains(p))
            .map(|p| p.symbol.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(newly, demanded);
        assert!(p
            .unresolved_boundary_vc_symbols()
            .iter()
            .any(|s| s.contains(".AcceptInput.")));
        let row = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
        eprintln!(
            "Boundary rules {id}:{} terms,{} declarations;{} new predicates",
            cert.term_table.len(),
            cert.declarations.len(),
            newly.len()
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &output {
            fs::create_dir_all(out).unwrap();
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(base.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(row);
    }
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if let Some(out) = &output {
        fs::write(out.join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(base.join("certificates.json")).unwrap(), bytes);
    }
    eprintln!(
        "Boundary rules:15 original contexts,{observations} actual core predicate observations"
    );
}
