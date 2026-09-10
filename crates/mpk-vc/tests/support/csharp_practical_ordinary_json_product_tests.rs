//! Actual-source compound parsing and unchanged child/syntax definitions.
use super::*;

#[path = "csharp_practical_ordinary_json_builtin_tests.rs"]
mod builtin_tests;
#[path = "csharp_practical_ordinary_json_enum_tests.rs"]
mod enum_tests;
#[path = "csharp_practical_ordinary_json_ordered_tests.rs"]
mod ordered_tests;
#[path = "csharp_practical_ordinary_json_semantic_product_tests.rs"]
mod semantic_tests;
#[path = "csharp_practical_ordinary_json_sequence_tests.rs"]
mod sequence_tests;
#[path = "csharp_practical_ordinary_json_product_source_tests.rs"]
mod source_tests;
#[path = "csharp_practical_ordinary_json_sum_tests.rs"]
mod sum_tests;
#[path = "csharp_practical_ordinary_json_transition_tests.rs"]
mod transition_tests;

#[path = "csharp_practical_ordinary_json_cell_count_tests.rs"]
mod cell_count_tests;

#[test]
fn csharp_03_t06_w09_json_products_sources_and_dependency_preservation() {
    let bundle = b();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-products");
    let out = std::env::var_os("MPK_W09_JSON_PRODUCTS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let selected = std::env::var("MPK_W09_JSON_PRODUCTS_SELECT").ok();
    let mut rows = vec![];
    let mut contexts = 0;
    let mut defined_products = 0;
    for (id, row, facts) in super::value_tests::sources() {
        if selected.as_ref().is_some_and(|wanted| wanted != &id) {
            continue;
        }
        contexts += 1;
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
        let p = generate_csharp_practical_ordinary_json_products(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_json_products(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if !p.sums().is_empty() {
            assert_eq!(
                p.sums().len(),
                2,
                "this source owns Option<i32> and Option<string>"
            );
            assert!(p
                .sums()
                .iter()
                .all(|sum| sum.template_id == "mpk.csharp.semantic.option.v1"));
            for pointer in [
                "/sums/0/template_id",
                "/sums/0/arms/1/payload_type_id",
                "/sums/0/arms/1/payload_parse_definition",
                "/sums/0/arms/1/tag_literal/match_definition",
            ] {
                let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                *forged.pointer_mut(pointer).unwrap() = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_json_products(
                        &serde_json::to_vec(&forged).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err(),
                    "mutated {pointer}"
                );
            }
        }
        let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
        let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
        assert_eq!(p.primitives(), values.definitions());
        for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            super::super::super::structural_equivalence_tests::same_definition_closure(
                &old, &cert, &names,
            )
            .unwrap();
        }
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let defined = p
            .products()
            .iter()
            .map(|d| d.carrier.type_id.as_str())
            .chain(p.primitives().iter().map(|d| d.carrier.type_id.as_str()))
            .chain(p.sequences().iter().map(|d| d.carrier.type_id.as_str()))
            .chain(p.sums().iter().map(|d| d.carrier.type_id.as_str()))
            .chain(
                p.collections()
                    .iter()
                    .map(|d| d.sequence.carrier.type_id.as_str()),
            )
            .chain(p.enums().iter().map(|d| d.carrier.type_id.as_str()))
            .chain(p.vocabulary().iter().map(|d| d.carrier.type_id.as_str()))
            .collect::<BTreeSet<_>>();
        let deferred = layouts
            .carriers()
            .iter()
            .filter(|c| !defined.contains(c.type_id.as_str()))
            .map(|c| c.type_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.deferred_type_ids()
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            deferred
        );
        for product in p.products() {
            let source = facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == product.carrier.type_id)
                .unwrap();
            assert_eq!(
                product.member_names,
                source["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m["name"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            );
            let OrdinaryShape::Product { fields } = &product.carrier.shape else {
                panic!("source product layout");
            };
            assert_eq!(
                product.member_ids,
                fields.iter().map(|f| f.id.clone()).collect::<Vec<_>>()
            );
            assert_eq!(product.packet_depth, product.carrier.depth.max(7) + 1);
            defined_products += 1;
        }
        if syntax.literals().is_empty() {
            assert!(p.products().is_empty());
            continue;
        }
        assert_eq!(p.products().len(), usize::from(id.starts_with("calendar-")) + usize::from(id == "document-decb5507362a7672c711a5befd63efc1038c2c0a86ee20137a1c4c472e0d1684") + usize::from(id == "document-7a157feb27ed416752b5dde8aa2bf373b9a2c82755d2caa89e955057176ee57b"));
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        forged["deferred_type_ids"] = json!(["forged"]);
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_products(&p.canonical_bytes(), &bad, vir)
                .is_err()
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &out {
            fs::write(out.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!(
            "JSON products {id}: {} source products, {} terms, {} declarations",
            p.products().len(),
            cert.term_table.len(),
            cert.declarations.len()
        );
        rows.push(json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    if selected.is_some() {
        assert_eq!(contexts, 1, "selected source must exist");
        assert_eq!(rows.len(), 1, "selected source must have JSON grammar");
    } else {
        assert_eq!(contexts, 72);
        assert_eq!(rows.len(), 7);
        assert_eq!(defined_products, 6);
    }
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if let Some(out) = &out {
        fs::write(out.join("certificates.json"), bytes).unwrap();
    } else {
        if let Some(id) = &selected {
            let all: Value =
                serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
            let expected = all
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == *id)
                .unwrap();
            assert_eq!(&rows[0], expected);
        } else {
            assert_eq!(fs::read(root.join("certificates.json")).unwrap(), bytes);
        }
    }
}

#[test]
fn csharp_03_t06_w09_json_products_actual_core_parsing() {
    let root = std::env::var_os("MPK_W09_JSON_PRODUCTS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-products")
        });
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    let mut cases = 0;
    for row in rows
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["id"].as_str().unwrap().starts_with("calendar-"))
    {
        let id = row["id"].as_str().unwrap();
        let hex = fs::read_to_string(root.join(format!("{id}.hex"))).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let d = &row["program"]["products"][0];
        let names = d["member_names"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        let members = names
            .iter()
            .map(|name| match *name {
                "Date" => "\"Date\":\"0001-01-03\"",
                "Time" => "\"Time\":\"00:00:00.0000003\"",
                "Amount" => "\"Amount\":\"1.25\"",
                _ => panic!("unexpected source member"),
            })
            .collect::<Vec<_>>();
        let canonical = format!("{{{}}}", members.join(","));
        let value_depth = d["carrier"]["depth"].as_u64().unwrap() as u32;
        let packet_depth = d["packet_depth"].as_u64().unwrap() as u32;
        let roles = if names.len() <= 1 {
            0
        } else {
            32 - (names.len() as u32 - 1).leading_zeros()
        };
        let stride = value_depth - roles;
        let mut value_ones = BTreeSet::new();
        for (field, name) in names.iter().enumerate() {
            let (depth, ones) = match *name {
                "Date" => (5, BTreeSet::from([1])),    // Day number two.
                "Time" => (6, BTreeSet::from([0, 1])), // Three ticks.
                "Amount" => {
                    // Decimal fields: 2 role bits, then leading child padding.
                    // Scale two is at 1 + (1 << 6); coefficient is 125.
                    let mut bits = BTreeSet::from([65]);
                    bits.extend(
                        (0..96)
                            .filter(|i| 125_u128 & (1_u128 << i) != 0)
                            .map(|i| 2 | (i << 2)),
                    );
                    (9, bits)
                }
                _ => unreachable!(),
            };
            for index in ones {
                value_ones.insert(field | (index << (roles + stride - depth)));
            }
        }
        let mut check = |text: &str,
                         start: u32,
                         ending: u8,
                         depth: u32,
                         good: bool,
                         eof: bool,
                         end: u32| {
            let doc = document(
                text.len() as u32,
                &text.bytes().enumerate().collect::<Vec<_>>(),
            );
            let result = run(
                &cert,
                d["parse_definition"].as_str().unwrap(),
                vec![doc, word(start), word_tag(ending), word(depth)],
            );
            let mut expected = vec![false; 1 << packet_depth];
            if good {
                expected[0] = true;
                expected[2] = eof;
                for i in 0..32 {
                    expected[(2 + i) << 1] = end & (1 << i) != 0;
                    expected[(34 + i) << 1] = (names.len() as u32 + 1) & (1 << i) != 0;
                }
                for index in &value_ones {
                    expected[1 | (index << 1)] = true;
                }
            }
            for (i, wanted) in expected.into_iter().enumerate() {
                assert_eq!(
                    leaf(&cert, result.clone(), packet_depth as usize, i),
                    wanted,
                    "{id}, case {cases}, bit {i}, {text}"
                );
            }
            eprintln!("JSON product core case {cases}: {id}, start {start}, depth {depth}, accepted {good}");
            cases += 1;
        };
        check(&canonical, 0, 0, 0, true, true, canonical.len() as u32);
        if id == "calendar-date-time" {
            let prefix = format!("x{canonical},");
            check(&prefix, 1, 1, 0, true, false, canonical.len() as u32 + 1);
            check(
                &format!(
                    "{{{}}}",
                    members.iter().rev().copied().collect::<Vec<_>>().join(",")
                ),
                0,
                0,
                0,
                false,
                false,
                0,
            );
            check(&format!("{{{}}}", members[0]), 0, 0, 0, false, false, 0);
            check(
                &format!("{{{},{},{}}}", members[0], members[0], members[1]),
                0,
                0,
                0,
                false,
                false,
                0,
            );
            check(&canonical, 0, 0, 31, true, true, canonical.len() as u32);
            check(&canonical, 0, 0, 32, false, false, 0);
        }
    }
    assert_eq!(cases, 10);
}

#[path = "csharp_practical_ordinary_json_boundary_field_tests.rs"]
mod boundary_field_tests;

#[path = "csharp_practical_ordinary_json_envelope_tests.rs"]
mod envelope_tests;

#[path = "csharp_practical_ordinary_json_typed_depth_tests.rs"]
mod typed_depth_tests;

#[path = "csharp_practical_ordinary_json_depth_compound_tests.rs"]
mod depth_compound_tests;

#[path = "csharp_practical_ordinary_json_semantic_root_tests.rs"]
mod semantic_root_tests;

#[path = "csharp_practical_ordinary_json_typed_node_tests.rs"]
mod typed_node_tests;
