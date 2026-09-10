use super::*;
use core_eval::{apply, bit, run, sparse_cube, V};

fn requests() -> Value {
    requests_with_conditional(false)
}
fn requests_with_conditional(conditional: bool) -> Value {
    requests_with_recipe(if conditional { "conditional" } else { "base" })
}
fn requests_with_recipe(recipe: &str) -> Value {
    use a::PracticalJsonValue as J;
    use mpk_vc::csharp_practical_source_artifacts as a;
    let bundle = b();
    let type_id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Data","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let root = csharp_practical_declaration_id(&json!({"kind":"method","namespace":"Data","owner":type_id("Entry"),"name":"Run","parameter_type_ids":[type_id("Envelope")],"result_type_id":"mpk.csharp.value.i32.v1"})).unwrap();
    let code = b"namespace Data;public readonly struct Value{public readonly int Amount;public Value(int amount){Amount=amount;}}public readonly struct Envelope{public readonly Value Direct;public readonly Value[] Items;public readonly Value? Optional;}public static class Entry{public static int Run(Envelope value){return new Value(value.Direct.Amount).Amount;}}\n";
    let originals = read("construction-vc/requests.json");
    let original = originals
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "positive_constructor")
        .unwrap();
    let contract = original["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "type_contract")
        .or_else(|| {
            original["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["path"] == "contracts/data.json")
        })
        .unwrap();
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::TypeContract,
        contract["utf8"].as_str().unwrap().as_bytes(),
    )
    .unwrap();
    let (_, plain_captures) = support::context(&bundle, &root, code);
    let source_hash = plain_captures.entries()[0].raw_sha256().to_owned();
    let (context, captures) = support::context_with_sidecar(&bundle, &root, code, |context| {
        let J::Object(mut fields) = template.clone() else {
            panic!()
        };
        fields.retain(|(k, _)| k != "contract_sha256");
        for (key, value) in &mut fields {
            *value = match key.as_str() {
                "invariants" if recipe == "conditional" || recipe == "structural" => {
                    let J::Array(clauses) = value.clone() else {
                        panic!()
                    };
                    let base = clauses[0].clone();
                    let eq = |number: &str| {
                        let J::Object(mut fields) = base.clone() else {
                            panic!()
                        };
                        for (key, value) in &mut fields {
                            if key == "operation_id" {
                                *value = J::string("integer.i32.equal.checked");
                            }
                            if key == "right" {
                                let J::Object(fields) = value else { panic!() };
                                fields.iter_mut().find(|(k, _)| k == "value").unwrap().1 =
                                    J::string(number);
                            }
                        }
                        J::Object(fields)
                    };
                    if recipe == "structural" {
                        let J::Object(base_fields) = &base else {
                            panic!()
                        };
                        let field = base_fields
                            .iter()
                            .find(|(k, _)| k == "left")
                            .unwrap()
                            .1
                            .clone();
                        let zero = base_fields
                            .iter()
                            .find(|(k, _)| k == "right")
                            .unwrap()
                            .1
                            .clone();
                        let J::Object(field_fields) = &field else {
                            panic!()
                        };
                        let receiver = field_fields
                            .iter()
                            .find(|(k, _)| k == "receiver")
                            .unwrap()
                            .1
                            .clone();
                        let J::Object(one_eq) = eq("1") else { panic!() };
                        let one = one_eq.iter().find(|(k, _)| k == "right").unwrap().1.clone();
                        let relation = |tag: &str, ty: &str, left: J, right: J| {
                            J::Object(vec![
                                ("tag".into(), J::string(tag)),
                                ("type_id".into(), J::string(ty)),
                                ("left".into(), left),
                                ("right".into(), right),
                            ])
                        };
                        let equal = relation(
                            "structural_equal",
                            "mpk.csharp.value.bool.v1",
                            field.clone(),
                            one,
                        );
                        let compare =
                            relation("structural_compare", "mpk.csharp.value.i32.v1", field, zero);
                        let J::Object(mut negative) = eq("-1") else {
                            panic!()
                        };
                        negative.iter_mut().find(|(k, _)| k == "left").unwrap().1 = compare;
                        let boolean = |v: bool| {
                            J::Object(vec![
                                ("tag".into(), J::string("literal")),
                                ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
                                ("value".into(), J::Bool(v)),
                            ])
                        };
                        let conditional = |condition: J, yes: J, no: J| {
                            J::Object(vec![
                                ("tag".into(), J::string("conditional")),
                                ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
                                ("condition".into(), condition),
                                ("when_true".into(), yes),
                                ("when_false".into(), no),
                            ])
                        };
                        J::Array(vec![conditional(
                            relation(
                                "structural_equal",
                                "mpk.csharp.value.bool.v1",
                                receiver.clone(),
                                receiver,
                            ),
                            conditional(equal, boolean(true), J::Object(negative)),
                            boolean(false),
                        )])
                    } else {
                        J::Array(vec![J::Object(vec![
                            ("tag".into(), J::string("conditional")),
                            ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
                            ("condition".into(), base.clone()),
                            ("when_true".into(), eq("1")),
                            ("when_false".into(), eq("-1")),
                        ])])
                    }
                }
                "semantic_context" => context.semantic_context().clone(),
                "compilation_id" => J::string(context.compilation_id()),
                "source_content_sha256" => J::string(source_hash.as_str()),
                _ => value.clone(),
            };
        }
        let hash = mpk_vc::hash_domain_separated_raw(
            a::TYPE_CONTRACT_HASH_DOMAIN,
            &a::canonical_practical_json_bytes(&J::Object(fields.clone())).unwrap(),
        )
        .unwrap()
        .to_hex();
        fields.push(("contract_sha256".into(), J::string(hash)));
        a::canonical_practical_json_bytes(&J::Object(fields)).unwrap()
    });
    json!([{"id":"nested-public-clauses","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}
#[test]
fn csharp_03_t06_w09_public_domain_original_source_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_PUBLIC_DOMAIN_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/public-domain-sources/requests.json"),
            requests()
        );
    }
}
fn count(c: &mpk_cert::encode::Certificate, v: V) -> u32 {
    (0..32).fold(0, |n, i| {
        let mut b = v.clone();
        for k in 0..5 {
            b = apply(c, b, V::Bit(i & (1 << k) != 0));
        }
        n | ((bit(b) as u32) << i)
    })
}
#[test]
fn csharp_03_t06_w09_public_domains_construction_sources() {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let original = read("construction-vc/goldens.json");
    let out = std::env::var_os("MPK_W09_PUBLIC_DOMAINS_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    for row in original.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == row["id"])
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == row["id"])
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_public_domains(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_public_domains(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        let clauses = generate_csharp_practical_ordinary_source_clauses(emitted.vir()).unwrap();
        assert_eq!(p.source_clauses(), clauses.definitions());
        let old = mpk_cert::decode_canonical_certificate(clauses.certificate_bytes()).unwrap();
        let names = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&old, &c, &names).unwrap();
        let source_type = &row["construction"]["types"][0];
        let d = p
            .definitions()
            .iter()
            .find(|d| d.carrier.type_id == source_type["type_id"].as_str().unwrap())
            .unwrap();
        if id != "enum_zero" {
            let members = source_type["members"].as_array().unwrap();
            let index = members
                .iter()
                .position(|m| m["type_id"] == "mpk.csharp.value.i32.v1")
                .unwrap();
            for n in [-1i32, 0, 1] {
                let bits = (0..32)
                    .filter(|i| (n as u32) & (1 << i) != 0)
                    .map(|i| index | (i << (d.carrier.depth - 5)))
                    .collect();
                let value = sparse_cube(d.carrier.depth, bits);
                let expected = if n > 0 || (n == 0 && id == "positive_default") {
                    1 + members.len() as u32
                } else {
                    65_537
                };
                assert_eq!(
                    count(&c, run(&c, &d.count_definition, vec![value.clone()])),
                    expected,
                    "{id}:{n}"
                );
                if n == 0 {
                    assert_eq!(
                        bit(run(&c, &d.valid_definition, vec![value])),
                        expected <= 65_536
                    );
                }
            }
        }
        let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        bad["source_clauses"] = json!("forged");
        assert!(import_csharp_practical_ordinary_public_domains(
            &serde_json::to_vec(&bad).unwrap(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        let metadata = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":c.term_table.len(),"declarations":c.declarations.len()});
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &out {
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/public-domains");
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(metadata);
    }
    assert_eq!(rows.len(), 7);
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/public-domains/certificates.json"),
            json!(rows)
        );
    }
}

#[test]
fn csharp_03_t06_w09_public_domain_preserves_representation_bytes() {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/domains");
    for id in ["positive_constructor", "enum_zero"] {
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
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let structural =
            generate_csharp_practical_ordinary_structural_foundations(emitted.vir()).unwrap();
        let old = root
            .parent()
            .unwrap()
            .join("structural-foundations")
            .join(format!("construction-vc-{id}.hex"));
        let structural_hex = structural
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        assert_eq!(
            fs::read_to_string(old).unwrap(),
            structural_hex,
            "{id}: structural predecessor"
        );
        let p = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        assert_eq!(
            fs::read_to_string(root.join(format!("construction-vc-{id}.hex"))).unwrap(),
            hex,
            "{id}"
        );
    }
}

#[test]
fn csharp_03_t06_w09_public_domain_nested_source_clauses() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_PUBLIC_DOMAIN_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/public-domain-sources/responses.json")
    };
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    assert!(responses[0].get("reject").is_none(), "{responses}");
    let facts = &responses[0]["facts"];
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_public_domains(emitted.vir()).unwrap();
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let types: BTreeMap<_, _> = p
        .definitions()
        .iter()
        .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
        .collect();
    let envelope_id = csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Data","owner":"","name":"Envelope","parameter_type_ids":[],"result_type_id":""})).unwrap();
    let template =
        relation_tests::sample(&envelope_id, 0, &types, facts, emitted.closure().closed());
    let MonomorphicValue::Product { fields, .. } = &template else {
        panic!()
    };
    let get = |name: &str| {
        fields
            .iter()
            .find(|f| f.name == name)
            .unwrap()
            .value
            .as_ref()
    };
    let positive = |n: i32| {
        let mut value = get("Direct").clone();
        let MonomorphicValue::Product { fields, .. } = &mut value else {
            panic!()
        };
        let MonomorphicValue::Signed { value: amount, .. } = fields[0].value.as_mut() else {
            panic!()
        };
        *amount = n.to_string();
        value
    };
    let make = |direct: i32, items: &[i32], optional: Option<i32>| {
        let mut value = template.clone();
        let MonomorphicValue::Product { fields, .. } = &mut value else {
            panic!()
        };
        for field in fields {
            match field.name.as_str() {
                "Direct" => *field.value = positive(direct),
                "Items" => {
                    let MonomorphicValue::Sequence { elements, .. } = field.value.as_mut() else {
                        panic!()
                    };
                    *elements = items.iter().map(|n| positive(*n)).collect();
                }
                "Optional" => {
                    let MonomorphicValue::Option { arm, value, .. } = field.value.as_mut() else {
                        panic!()
                    };
                    *arm = if optional.is_some() {
                        OptionArm::Some
                    } else {
                        OptionArm::None
                    };
                    *value = optional.map(|n| Box::new(positive(n)));
                }
                _ => panic!(),
            }
        }
        value
    };
    // Preserve a None tag but inject a nonzero inactive payload through the
    // independent storage encoder. Inactive source clauses are not evaluated,
    // but representation padding must still be zero.
    let mut inactive = make(1, &[], None);
    let MonomorphicValue::Product { fields, .. } = &mut inactive else {
        panic!()
    };
    let optional = fields.iter_mut().find(|f| f.name == "Optional").unwrap();
    let MonomorphicValue::Option { value, .. } = optional.value.as_mut() else {
        panic!()
    };
    *value = Some(Box::new(positive(1)));
    let definition = p
        .definitions()
        .iter()
        .find(|d| d.carrier.type_id == envelope_id)
        .unwrap();
    for (label, value, expected) in [
        ("empty and none", make(1, &[], None), 5),
        ("direct clause", make(0, &[], None), 65_537),
        ("array active valid", make(1, &[1], None), 7),
        ("array active clause", make(1, &[0], None), 65_537),
        ("array later clause", make(1, &[1, -1], None), 65_537),
        ("some active valid", make(1, &[], Some(1)), 7),
        ("some active clause", make(1, &[], Some(0)), 65_537),
        ("none nonzero inactive payload", inactive, 65_537),
    ] {
        let (depth, ones) = projection_tests::sparse_storage(&value, &types);
        assert_eq!(
            count(
                &c,
                run(
                    &c,
                    &definition.count_definition,
                    vec![sparse_cube(depth, ones)]
                )
            ),
            expected,
            "{label}"
        );
    }
}

#[test]
fn csharp_03_t06_w09_public_defaults_source_conditions() {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let goldens = read("construction-vc/goldens.json");
    let mut sources = vec![];
    for id in [
        "positive_default",
        "broken_default",
        "positive_constructor",
        "enum_zero",
    ] {
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone();
        let source_id = goldens
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()["construction"]["types"][0]["type_id"]
            .as_str()
            .unwrap()
            .to_owned();
        sources.push((id.to_owned(), request, response, Some(source_id)));
    }
    sources.push((
        "nested".into(),
        read("ordinary-foundation/public-domain-sources/requests.json")[0].clone(),
        read("ordinary-foundation/public-domain-sources/responses.json")[0].clone(),
        None,
    ));
    let source_rows = read("data-phase/data-stage-replay.json");
    let recursive = source_rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "f0219603fe7565fabe57876c5135941b287b1f66906ef2c463f7ce72c88cda19")
        .unwrap();
    sources.push((
        "declared-recursive".into(),
        recursive.clone(),
        recursive["outcome"].clone(),
        None,
    ));
    let out = std::env::var_os("MPK_W09_PUBLIC_DEFAULTS_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut manifest = vec![];
    for (id, request, response, source_id) in sources {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_public_defaults(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_public_defaults(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        assert!(import_csharp_practical_ordinary_public_domains(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        let plain = generate_csharp_practical_ordinary_public_domains(emitted.vir()).unwrap();
        assert!(plain.public_defaults().is_none());
        assert!(import_csharp_practical_ordinary_public_defaults(
            &plain.canonical_bytes(),
            plain.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        let defaults = generate_csharp_practical_ordinary_defaults(emitted.vir()).unwrap();
        for bytes in [plain.certificate_bytes(), defaults.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&old, &c, &names).unwrap();
        }
        let rows = p.public_defaults().unwrap();
        for row in rows {
            assert_eq!(
                &row.default,
                defaults
                    .definitions()
                    .iter()
                    .find(|d| d.carrier.type_id == row.default.carrier.type_id)
                    .unwrap()
            );
        }
        if let Some(source_id) = source_id {
            let row = rows
                .iter()
                .find(|r| r.default.carrier.type_id == source_id)
                .unwrap();
            assert!(row.default.structural_candidate.is_some());
            let declared = response["facts"]["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == source_id)
                .unwrap()["public_default"]
                .as_bool()
                .unwrap();
            assert_eq!(row.default.public_candidate_admitted, declared);
            let expected_domain = matches!(id.as_str(), "positive_default" | "enum_zero");
            let domain = p
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == source_id)
                .unwrap();
            let zero = run(
                &c,
                &row.default
                    .structural_candidate
                    .as_ref()
                    .unwrap()
                    .definition,
                vec![],
            );
            assert_eq!(
                bit(run(&c, &domain.valid_definition, vec![zero])),
                expected_domain,
                "{id}: domain"
            );
            assert_eq!(
                bit(run(&c, &row.valid_definition, vec![])),
                declared && expected_domain,
                "{id}: admission"
            );
        } else if id == "nested" {
            // Nullable None has no active Value child, even though Value(0)
            // violates its public invariant. Arrays have no CLR public default.
            let mut options = 0;
            let mut sequences = 0;
            for row in rows {
                if let Some(entry) = emitted
                    .closure()
                    .closed()
                    .entries()
                    .iter()
                    .find(|e| e["instance_id"] == row.default.carrier.type_id)
                {
                    match entry["template_id"].as_str().unwrap() {
                        "mpk.csharp.semantic.option.v1" => {
                            assert!(bit(run(&c, &row.valid_definition, vec![])));
                            assert!(row
                                .default
                                .structural_candidate
                                .as_ref()
                                .unwrap()
                                .source_requirements
                                .is_empty());
                            options += 1;
                        }
                        "mpk.csharp.semantic.bounded_sequence.v1" => {
                            assert!(!bit(run(&c, &row.valid_definition, vec![])));
                            assert!(row.default.structural_candidate.is_none());
                            sequences += 1;
                        }
                        _ => {}
                    }
                }
            }
            assert_eq!((options, sequences), (1, 1));
        } else {
            let source_rows = rows
                .iter()
                .filter(|r| r.default.carrier.type_id.starts_with("mpk.csharp.source."))
                .collect::<Vec<_>>();
            assert_eq!(source_rows.len(), 17);
            for row in source_rows {
                assert!(row.default.public_candidate_admitted);
                assert!(bit(run(&c, &row.valid_definition, vec![])));
            }
        }
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        forged["public_defaults"] = json!([]);
        assert!(import_csharp_practical_ordinary_public_defaults(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        let row = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":c.term_table.len(),"declarations":c.declarations.len()});
        if let Some(out) = &out {
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/public-defaults");
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        manifest.push(row);
    }
    assert_eq!(manifest.len(), 6);
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/public-defaults/certificates.json"),
            json!(manifest)
        );
    }
}

#[test]
fn csharp_03_t06_w09_structural_public_composes_existing_definitions() {
    let bundle = b();
    let mut sources = vec![];
    for (family, id) in [
        ("construction-vc", "positive_constructor"),
        ("binding-vc", "ordered_map"),
    ] {
        let req = read(&format!("{family}/requests.json"));
        let res = read(&format!("{family}/responses.json"));
        sources.push((
            format!("{family}-{id}"),
            req.as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap()
                .clone(),
            res.as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap()["facts"]
                .clone(),
        ));
    }
    sources.push((
        "nested".into(),
        read("ordinary-foundation/public-domain-sources/requests.json")[0].clone(),
        read("ordinary-foundation/public-domain-sources/responses.json")[0]["facts"].clone(),
    ));
    let out = std::env::var_os("MPK_W09_STRUCTURAL_PUBLIC_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    for (id, request, facts) in sources {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_structural_public(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let base =
            generate_csharp_practical_ordinary_structural_foundations(emitted.vir()).unwrap();
        let public = generate_csharp_practical_ordinary_public_defaults(emitted.vir()).unwrap();
        assert_eq!(p.source_clauses().unwrap(), public.source_clauses());
        assert_eq!(p.public_domains().unwrap(), public.definitions());
        assert_eq!(p.public_defaults(), public.public_defaults());
        assert_eq!(p.domains(), base.domains());
        assert_eq!(p.defaults(), base.defaults());
        assert_eq!(p.collections(), base.collections());
        assert_eq!(p.source_observations(), base.source_observations());
        assert_eq!(p.deferred_instances(), base.deferred_instances());
        for bytes in [base.certificate_bytes(), public.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&old, &c, &names)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        }
        assert_eq!(
            import_csharp_practical_ordinary_structural_public(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        assert!(import_csharp_practical_ordinary_structural_foundations(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        assert!(import_csharp_practical_ordinary_structural_public(
            &base.canonical_bytes(),
            base.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        for key in ["source_clauses", "public_domains", "public_defaults"] {
            let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            forged[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_structural_public(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &out {
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-public");
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":c.term_table.len(),"declarations":c.declarations.len()}));
    }
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/structural-public/certificates.json"),
            json!(rows)
        );
    }
}

#[test]
fn csharp_03_t06_w09_conditional_clause_original_source() {
    let bundle = b();
    let request = requests_with_conditional(true);
    // Contract inputs participate in the captured compilation context. Use the
    // independently captured response for this exact source/sidecar request.
    let response: Value = if let Some(path) = std::env::var_os("MPK_W09_CONDITIONAL_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/conditional-clauses/responses.json")
    };
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_source_clauses(emitted.vir()).unwrap();
    assert_eq!(p.definitions().len(), 1);
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let integrated = generate_csharp_practical_ordinary_structural_public(emitted.vir()).unwrap();
    let integrated_cert =
        mpk_cert::decode_canonical_certificate(integrated.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&integrated_cert).unwrap();
    assert_eq!(integrated.source_clauses().unwrap(), p.definitions());
    let names = c
        .declarations
        .iter()
        .map(|d| c.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(&c, &integrated_cert, &names).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_structural_public(
            &integrated.canonical_bytes(),
            integrated.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        integrated
    );
    let types = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
    let depth = types
        .carriers()
        .iter()
        .find(|t| t.type_id == p.definitions()[0].source_type_id)
        .unwrap()
        .depth;
    assert_eq!(depth, 5);
    for n in [i32::MIN, -2, -1, 0, 1, 2, i32::MAX] {
        let ones = (0..32).filter(|i| (n as u32) & (1 << i) != 0).collect();
        assert_eq!(
            bit(run(
                &c,
                &p.definitions()[0].definition,
                vec![sparse_cube(depth, ones)]
            )),
            n == -1 || n == 1,
            "{n}"
        );
    }
    assert_eq!(
        import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        p
    );
    let out = std::env::var_os("MPK_W09_CONDITIONAL_CLAUSES_OUT").map(std::path::PathBuf::from);
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    if let Some(out) = out {
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("conditional-source.hex"), hex).unwrap();
        fs::write(
            out.join("requests.json"),
            serde_json::to_vec_pretty(&request).unwrap(),
        )
        .unwrap();
        fs::write(out.join("program.json"), p.canonical_bytes()).unwrap();
        fs::write(
            out.join("integrated-program.json"),
            integrated.canonical_bytes(),
        )
        .unwrap();
        fs::write(
            out.join("conditional-integrated.hex"),
            integrated
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n",
        )
        .unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/conditional-clauses");
        assert_eq!(
            fs::read_to_string(root.join("conditional-source.hex")).unwrap(),
            hex
        );
        assert_eq!(
            fs::read(root.join("program.json")).unwrap(),
            p.canonical_bytes()
        );
        assert_eq!(
            read("ordinary-foundation/conditional-clauses/requests.json"),
            request
        );
        assert_eq!(
            fs::read(root.join("integrated-program.json")).unwrap(),
            integrated.canonical_bytes()
        );
        assert_eq!(
            fs::read_to_string(root.join("conditional-integrated.hex")).unwrap(),
            integrated
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n"
        );
    }
}

#[test]
fn csharp_03_t06_w09_conditional_requests() {
    let requests = requests_with_conditional(true);
    if let Some(path) = std::env::var_os("MPK_W09_CONDITIONAL_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/conditional-clauses/requests.json"),
            requests
        );
    }
}

#[test]
fn csharp_03_t06_w09_structural_clauses_original_source() {
    let bundle = b();
    let request = requests_with_recipe("structural");
    // Contract inputs participate in the captured compilation context. Use the
    // independently captured response for this exact source/sidecar request.
    let response: Value =
        if let Some(path) = std::env::var_os("MPK_W09_STRUCTURAL_CLAUSES_RESPONSES") {
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
        } else {
            read("ordinary-foundation/structural-clauses/responses.json")
        };
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_source_clauses(emitted.vir()).unwrap();
    assert_eq!(p.definitions().len(), 1);
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let integrated = generate_csharp_practical_ordinary_structural_public(emitted.vir()).unwrap();
    let integrated_cert =
        mpk_cert::decode_canonical_certificate(integrated.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&integrated_cert).unwrap();
    let base = generate_csharp_practical_ordinary_structural_foundations(emitted.vir()).unwrap();
    let base_cert = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
    let base_names = base_cert
        .declarations
        .iter()
        .map(|d| base_cert.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(
        &base_cert,
        &integrated_cert,
        &base_names,
    )
    .unwrap();
    assert_eq!(integrated.relations(), base.relations());
    assert_eq!(integrated.collections(), base.collections());
    assert_eq!(integrated.domains(), base.domains());

    // Exercise the standalone public-domain consumer with a nonempty relation
    // cache, and compare all its definitions with the integrated program.
    let public = generate_csharp_practical_ordinary_public_defaults(emitted.vir()).unwrap();
    let public_cert = mpk_cert::decode_canonical_certificate(public.certificate_bytes()).unwrap();
    let public_names = public_cert
        .declarations
        .iter()
        .map(|d| public_cert.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(
        &public_cert,
        &integrated_cert,
        &public_names,
    )
    .unwrap();
    assert_eq!(integrated.public_domains().unwrap(), public.definitions());
    assert_eq!(integrated.public_defaults(), public.public_defaults());

    assert_eq!(integrated.source_clauses().unwrap(), p.definitions());
    let names = c
        .declarations
        .iter()
        .map(|d| c.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(&c, &integrated_cert, &names).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_structural_public(
            &integrated.canonical_bytes(),
            integrated.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        integrated
    );
    let types = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
    let depth = types
        .carriers()
        .iter()
        .find(|t| t.type_id == p.definitions()[0].source_type_id)
        .unwrap()
        .depth;
    assert_eq!(depth, 5);
    for n in [i32::MIN, -2, -1, 0, 1, 2, i32::MAX] {
        let ones = (0..32).filter(|i| (n as u32) & (1 << i) != 0).collect();
        assert_eq!(
            bit(run(
                &c,
                &p.definitions()[0].definition,
                vec![sparse_cube(depth, ones)]
            )),
            n < 0 || n == 1,
            "{n}"
        );
    }
    assert_eq!(
        import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        p
    );
    let out = std::env::var_os("MPK_W09_STRUCTURAL_CLAUSES_OUT").map(std::path::PathBuf::from);
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    if let Some(out) = out {
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("structural-source.hex"), hex).unwrap();
        fs::write(
            out.join("requests.json"),
            serde_json::to_vec_pretty(&request).unwrap(),
        )
        .unwrap();
        fs::write(out.join("program.json"), p.canonical_bytes()).unwrap();
        fs::write(
            out.join("integrated-program.json"),
            integrated.canonical_bytes(),
        )
        .unwrap();
        fs::write(
            out.join("structural-integrated.hex"),
            integrated
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n",
        )
        .unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-clauses");
        assert_eq!(
            fs::read_to_string(root.join("structural-source.hex")).unwrap(),
            hex
        );
        assert_eq!(
            fs::read(root.join("program.json")).unwrap(),
            p.canonical_bytes()
        );
        assert_eq!(
            read("ordinary-foundation/structural-clauses/requests.json"),
            request
        );
        assert_eq!(
            fs::read(root.join("integrated-program.json")).unwrap(),
            integrated.canonical_bytes()
        );
        assert_eq!(
            fs::read_to_string(root.join("structural-integrated.hex")).unwrap(),
            integrated
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n"
        );
    }
}

#[test]
fn csharp_03_t06_w09_structural_clause_requests() {
    let requests = requests_with_recipe("structural");
    if let Some(path) = std::env::var_os("MPK_W09_STRUCTURAL_CLAUSES_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/structural-clauses/requests.json"),
            requests
        );
    }
}

#[path = "csharp_practical_ordinary_literal_clause_tests.rs"]
mod literal_clause_tests;

#[path = "csharp_practical_ordinary_total_clause_tests.rs"]
mod total_clause_tests;

#[path = "csharp_practical_ordinary_definedness_clause_tests.rs"]
mod definedness_clause_tests;

#[path = "csharp_practical_ordinary_partial_read_clause_tests.rs"]
mod partial_read_clause_tests;

#[path = "csharp_practical_ordinary_contract_expression_tests.rs"]
mod contract_expression_tests;
