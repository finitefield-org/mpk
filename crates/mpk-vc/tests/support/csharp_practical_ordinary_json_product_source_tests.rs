//! Reproducible nested/empty source products for ordinary JSON grammar.
use super::*;
use sha2::{Digest, Sha256};

pub(super) fn requests() -> Value {
    requests_for(&[
        ("nested", "namespace Boundary;public readonly struct Inner{public readonly int Number;public Inner(int number){Number=number;}}public readonly struct Payload{public readonly Inner Item;public Payload(Inner item){Item=item;}}public static class Entry{public static Payload Run(Payload p){return new Payload(new Inner(p.Item.Number));}}\n"),
        ("empty", "namespace Boundary;public readonly struct Payload{}public static class Entry{public static Payload Run(Payload p){return p;}}\n"),
    ])
}

pub(super) fn requests_for(sources: &[(&str, &str)]) -> Value {
    use PracticalJsonValue as J;
    let bundle = b();
    let template = read("boundary-output/source-requests.json")[0].clone();
    let root = template["roots"][0].as_str().unwrap();
    let mut requests = vec![];
    for &(id, source) in sources {
        let (context, captures) = support::context_with_sidecars(
            &bundle,
            root,
            source.as_bytes(),
            vec![
                "contracts/boundary.json".into(),
                "contracts/method.json".into(),
            ],
            |context| {
                [
                    (
                        "contracts/boundary.json",
                        PracticalArtifactKind::BoundaryContract,
                        "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
                    ),
                    (
                        "contracts/method.json",
                        PracticalArtifactKind::MethodContract,
                        "MPK-CSHARP-METHOD-CONTRACT-1.0",
                    ),
                ]
                .into_iter()
                .map(|(path, kind, domain)| {
                    let input = template["inputs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|v| v["path"] == path)
                        .unwrap();
                    let mut value = parse_canonical_practical_json(
                        kind,
                        input["utf8"].as_str().unwrap().as_bytes(),
                    )
                    .unwrap();
                    let J::Object(fields) = &mut value else {
                        panic!()
                    };
                    fields.retain(|(key, _)| key != "contract_sha256");
                    for (key, value) in fields.iter_mut() {
                        match key.as_str() {
                            "semantic_context" => *value = context.semantic_context().clone(),
                            "source_content_sha256" => {
                                *value =
                                    J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                            }
                            _ => {}
                        }
                    }
                    let bytes = canonical_practical_json_bytes(&value).unwrap();
                    let mut hash = Sha256::new();
                    hash.update(domain);
                    hash.update([0]);
                    hash.update(bytes);
                    let J::Object(fields) = &mut value else {
                        unreachable!()
                    };
                    fields.push((
                        "contract_sha256".into(),
                        J::string(format!("{:x}", hash.finalize())),
                    ));
                    canonical_practical_json_bytes(&value).unwrap()
                })
                .collect()
            },
        );
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),
            "inputs":captures.entries().iter().map(|entry|json!({"kind":if entry.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":entry.path(),"utf8":std::str::from_utf8(entry.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    json!(requests)
}
#[test]
fn csharp_03_t06_w09_json_product_source_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_PRODUCT_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/json-product-sources/requests.json")).unwrap(), bytes);
    }
}

fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_PRODUCT_NESTED_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-product-nested")
        })
}

#[test]
fn csharp_03_t06_w09_json_product_nested_source_replay() {
    let bundle = b();
    let requests = read("ordinary-foundation/json-product-sources/requests.json");
    let responses = read("ordinary-foundation/json-product-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 2);
    assert_eq!(responses.as_array().unwrap().len(), 2);
    let out = pins();
    let write = std::env::var_os("MPK_W09_JSON_PRODUCT_NESTED_OUT").is_some();
    if write {
        fs::create_dir_all(&out).unwrap();
    }
    let mut rows = vec![];
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == request["id"])
            .unwrap();
        assert!(response.get("reject").is_none());
        let facts = &response["facts"];
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
        assert_eq!(p.products().len(), if id == "nested" { 2 } else { 1 });
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
        let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
        let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        assert_eq!(p.primitives(), values.definitions());
        for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            super::super::super::super::structural_equivalence_tests::same_definition_closure(
                &old, &cert, &names,
            )
            .unwrap();
        }
        for product in p.products() {
            let source = facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == product.carrier.type_id)
                .unwrap();
            let members = source["members"].as_array().unwrap();
            assert_eq!(
                product.member_names,
                members
                    .iter()
                    .map(|m| m["name"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            );
            // Captured facts have names/spans; stable IDs come from the
            // independently reconstructed carrier, not raw capture fields.
            let layout = layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == product.carrier.type_id)
                .unwrap();
            assert_eq!(&product.carrier, layout);
            let OrdinaryShape::Product { fields } = &layout.shape else {
                panic!()
            };
            assert_eq!(
                product.member_ids,
                fields.iter().map(|f| f.id.clone()).collect::<Vec<_>>()
            );
            assert!(!p.deferred_type_ids().contains(&product.carrier.type_id));
        }
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        forged["products"][0]["member_names"] = json!(["forged"]);
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if write {
            fs::write(out.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(out.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!(
            "JSON product nested source {id}: {} products, {} terms, {} declarations",
            p.products().len(),
            cert.term_table.len(),
            cert.declarations.len()
        );
        rows.push(
            json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),
            "terms":cert.term_table.len(),"declarations":cert.declarations.len()}),
        );
    }
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if write {
        fs::write(out.join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(out.join("certificates.json")).unwrap(), bytes);
    }
}

#[test]
fn csharp_03_t06_w09_json_product_nested_actual_core() {
    let root = pins();
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    let mut count = 0;
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let nested = id == "nested";
        let hex = fs::read_to_string(root.join(format!("{id}.hex"))).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let d = row["program"]["products"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| {
                if nested {
                    d["member_names"] == json!(["Item"])
                } else {
                    d["member_names"] == json!([])
                }
            })
            .unwrap();
        // A single field adds no role/padding bits at either nesting level.
        assert_eq!(d["carrier"]["depth"], if nested { 5 } else { 0 });
        let packet_depth = d["packet_depth"].as_u64().unwrap() as usize;
        let cases = if nested {
            vec![
                ("{\"Item\":{\"Number\":-2}}", 0, 0, 0, true, true, 22),
                ("{\"Item\":{\"Number\":-2}}", 0, 0, 30, true, true, 22),
                ("{\"Item\":{\"Number\":-2}}", 0, 0, 31, false, false, 0),
                ("x{\"Item\":{\"Number\":-2}},", 1, 1, 0, true, false, 23),
                (
                    "{\"Item\":{\"Number\":-2,\"Number\":1}}",
                    0,
                    0,
                    0,
                    false,
                    false,
                    0,
                ),
                ("{\"Item\":{\"Number\":-2}", 0, 0, 0, false, false, 0),
            ]
        } else {
            vec![
                ("{}", 0, 0, 0, true, true, 2),
                ("{}", 0, 0, 32, true, true, 2),
                ("{}", 0, 0, 33, false, false, 0),
                ("x{},", 1, 1, 0, true, false, 3),
                ("{ }", 0, 0, 0, false, false, 0),
                ("{\"x\":0}", 0, 0, 0, false, false, 0),
                ("{", 0, 0, 0, false, false, 0),
            ]
        };
        for (text, start, ending, depth, good, eof, end) in cases {
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
                    expected[(2 + i) << 1] = end & (1_u32 << i) != 0;
                    expected[(34 + i) << 1] = (if nested { 3_u32 } else { 1 }) & (1 << i) != 0;
                }
                if nested {
                    for i in 1..32 {
                        expected[1 | (i << 1)] = true;
                    } // Int32 -2.
                }
            }
            for (i, wanted) in expected.into_iter().enumerate() {
                assert_eq!(
                    leaf(&cert, result.clone(), packet_depth, i),
                    wanted,
                    "{id} text={text} depth={depth} bit={i}"
                );
            }
            count += 1;
            eprintln!("JSON nested core {count}: {id}, depth {depth}, accepted {good}");
        }
    }
    assert_eq!(count, 13);
}
