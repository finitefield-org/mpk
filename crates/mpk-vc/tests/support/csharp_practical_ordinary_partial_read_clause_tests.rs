//! Implicit W03 definedness for indexing and active sum payloads.
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
const BOOL: &str = "mpk.csharp.value.bool.v1";
const I32: &str = "mpk.csharp.value.i32.v1";
fn expression(tag: &str, ty: &str, fields: Vec<(&str, J)>) -> J {
    J::Object(
        [
            ("tag".into(), J::string(tag)),
            ("type_id".into(), J::string(ty)),
        ]
        .into_iter()
        .chain(fields.into_iter().map(|(key, value)| (key.into(), value)))
        .collect(),
    )
}
fn clauses() -> Vec<(String, J, bool)> {
    let literal = total_clause_tests::literal;
    let source = literal_clause_tests::cases()
        .into_iter()
        .find(|(n, _, _)| n == "product-one")
        .unwrap()
        .2
        .type_id()
        .to_owned();
    let equal = |value, target| {
        expression(
            "structural_equal",
            BOOL,
            vec![("left", value), ("right", target)],
        )
    };
    let index = |sequence, i: i32| {
        expression(
            "sequence_index",
            &source,
            vec![
                ("sequence", sequence),
                (
                    "index",
                    expression("literal", I32, vec![("value", J::string(i.to_string()))]),
                ),
            ],
        )
    };
    let mut result = [-1, 0, 1, 2, 4095, 4096, i32::MIN, i32::MAX]
        .into_iter()
        .map(|i| {
            (
                format!("index {i}"),
                equal(
                    index(literal("sequence"), i),
                    literal(if i == 1 {
                        "product-negative"
                    } else {
                        "product-one"
                    }),
                ),
                (0..2).contains(&i),
            )
        })
        .collect::<Vec<_>>();
    let J::Object(mut empty) = literal("sequence") else {
        panic!()
    };
    empty.iter_mut().find(|(k, _)| k == "value").unwrap().1 = J::Array(vec![]);
    result.push((
        "empty index".into(),
        equal(index(J::Object(empty), 0), literal("product-one")),
        false,
    ));
    for arm in ["some", "none"] {
        let value = literal(arm);
        let payload = expression(
            "tagged_payload",
            &source,
            vec![("value", value.clone()), ("arm", J::string("some"))],
        );
        let predicate = equal(payload, literal("product-one"));
        result.push((format!("payload {arm}"), predicate.clone(), arm == "some"));
        let guard = expression(
            "tagged_is",
            BOOL,
            vec![("value", value), ("arm", J::string("some"))],
        );
        result.push((
            format!("guarded payload {arm}"),
            expression(
                "conditional",
                BOOL,
                vec![
                    ("condition", guard),
                    ("when_true", predicate),
                    (
                        "when_false",
                        expression("literal", BOOL, vec![("value", J::Bool(true))]),
                    ),
                ],
            ),
            true,
        ));
    }
    result
}
fn partial_requests() -> Value {
    total_clause_tests::requests_with_expressions(
        clauses().into_iter().map(|(_, e, _)| e).collect(),
        "partial-read-clauses",
    )
}
#[test]
fn csharp_03_t06_w09_partial_read_requests() {
    let requests = partial_requests();
    if let Some(path) = std::env::var_os("MPK_W09_PARTIAL_READ_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/partial-read-clauses/requests.json"),
            requests
        );
    }
}
fn core_name(name: &str) -> String {
    format!(
        "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn word(value: u32) -> V {
    sparse_cube(5, (0..32).filter(|i| value & (1 << i) != 0).collect())
}
fn sequence(depth: u32, length: u32) -> V {
    assert_eq!(depth, 18);
    let mut ones = (0..32)
        .filter(|i| length & (1 << i) != 0)
        .map(|i| i << (depth - 5))
        .collect::<BTreeSet<_>>();
    for (index, value) in [(0, 1u32), (1, 0xffff_fffe), (4095, 42)] {
        for i in 0..32 {
            if value & (1 << i) != 0 {
                ones.insert(1 | (index << 1) | (i << 13));
            }
        }
    }
    sparse_cube(depth, ones)
}
#[test]
fn csharp_03_t06_w09_partial_read_original_source() {
    let bundle = b();
    let request = partial_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_PARTIAL_READ_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/partial-read-clauses/responses.json")
    };
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_source_clauses(vir).unwrap();
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let depths = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c.depth))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(p.definitions().len(), clauses().len());
    for ((label, expr, defined), d) in clauses().iter().zip(p.definitions()) {
        let mut hash = Sha256::new();
        hash.update(b"MPK-CSHARP-CONTRACT-EXPRESSION-1.0\0");
        hash.update(a::canonical_practical_json_bytes(expr).unwrap());
        assert_eq!(d.expression_sha256, format!("{:x}", hash.finalize()));
        let input = sparse_cube(depths[d.source_type_id.as_str()], BTreeSet::new());
        assert_eq!(
            bit(run(
                &c,
                d.definedness_definition.as_ref().unwrap(),
                vec![input.clone()]
            )),
            *defined,
            "{label} definedness"
        );
        if *defined {
            assert!(
                bit(run(&c, &d.definition, vec![input])),
                "{label} normal value"
            );
        }
    }
    let mut recipes = BTreeMap::new();
    for expression in vir.contract_expressions() {
        for d in expression.definitions() {
            if matches!(d.tag.as_str(), "sequence_index" | "tagged_payload") {
                recipes.insert(d.name.clone(), d.clone());
            }
        }
    }
    assert_eq!(recipes.len(), 2);
    let mut observations = 0;
    for (name, d) in recipes {
        let function = core_name(&name);
        if d.tag == "sequence_index" {
            let guard = core_name(&format!("Mpk.CSharp.Data.ContractDefined.{name}"));
            let indices = [
                0,
                1,
                2,
                4094,
                4095,
                4096,
                4097,
                65536,
                0x8000_0000,
                u32::MAX,
            ];
            for length in [0, 1, 2, 4095, 4096, u32::MAX] {
                for i in indices {
                    let value = sequence(depths[d.argument_types[0].as_str()], length);
                    let defined = i < length && i < 4096;
                    assert_eq!(
                        bit(run(&c, &guard, vec![value.clone(), word(i)])),
                        defined,
                        "length{length} index{i}"
                    );
                    let expected = if defined {
                        match i {
                            0 => 1,
                            1 => 0xffff_fffe,
                            4095 => 42,
                            _ => 0,
                        }
                    } else {
                        0
                    };
                    // Outside the guard this checks storage masking only, not
                    // admission of that result as a normal contract value.
                    assert_eq!(
                        count(&c, run(&c, &function, vec![value, word(i)])),
                        expected
                    );
                    observations += 33;
                }
            }
        } else {
            assert_eq!(depths[d.argument_types[0].as_str()], 6);
            for tag in [0, 1, 2, 0x1000_0001, u32::MAX] {
                for payload in [0, 42, 0x8000_0000, u32::MAX] {
                    let mut ones = (0..32)
                        .filter(|i| tag & (1 << i) != 0)
                        .map(|i| i << 1)
                        .collect::<BTreeSet<_>>();
                    ones.extend(
                        (0..32)
                            .filter(|i| payload & (1 << i) != 0)
                            .map(|i| 1 | (i << 1)),
                    );
                    assert_eq!(
                        count(&c, run(&c, &function, vec![sparse_cube(6, ones)])),
                        if tag == 1 { payload } else { 0 }
                    );
                    observations += 32;
                }
            }
        }
    }
    // Both cache calls and standalone assembly must retain the entire original
    // sequence operation/dependency closure, including checked-read masking.
    let sequences = generate_csharp_practical_ordinary_sequences(vir).unwrap();
    let sequence_cert =
        mpk_cert::decode_canonical_certificate(sequences.certificate_bytes()).unwrap();
    let names = sequence_cert
        .declarations
        .iter()
        .map(|d| sequence_cert.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(&sequence_cert, &c, &names).unwrap();
    assert!(generate_csharp_practical_ordinary_public_domains(vir).is_err());
    assert!(generate_csharp_practical_ordinary_public_defaults(vir).is_err());
    assert!(generate_csharp_practical_ordinary_structural_public(vir).is_err());
    assert_eq!(
        import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let mut mutated: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    mutated["definitions"][0]
        .as_object_mut()
        .unwrap()
        .remove("definedness_definition");
    assert!(import_csharp_practical_ordinary_source_clauses(
        &serde_json::to_vec(&mutated).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let outputs = [
        (
            "partial-reads.hex",
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into_bytes(),
        ),
        ("program.json", p.canonical_bytes()),
        (
            "requests.json",
            serde_json::to_vec_pretty(&request).unwrap(),
        ),
    ];
    let out = std::env::var_os("MPK_W09_PARTIAL_READ_OUT").map(std::path::PathBuf::from);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/partial-read-clauses");
    for (name, bytes) in outputs {
        if let Some(out) = &out {
            fs::create_dir_all(out).unwrap();
            fs::write(out.join(name), bytes).unwrap();
        } else {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
        }
    }
    eprintln!("partial reads: {} clauses,{observations} raw guard/storage observations,{} terms,{} declarations",clauses().len(),c.term_table.len(),c.declarations.len());
}
