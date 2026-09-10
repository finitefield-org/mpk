use super::*;
use core_eval::{apply, bit as observed_bit, run, V};
use mpk_cert::Certificate;

fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn check_value(certificate: &Certificate, value: V, expected: &[bool]) {
    assert!(expected.len().is_power_of_two());
    let depth = expected.len().trailing_zeros();
    for (address, expected) in expected.iter().enumerate() {
        let mut leaf = value.clone();
        for selector in 0..depth {
            leaf = apply(certificate, leaf, V::Bit(address & (1 << selector) != 0));
        }
        assert_eq!(observed_bit(leaf), *expected, "address {address}");
    }
}

#[test]
fn csharp_03_t06_w09_finite_operations_original_sources() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_FINITE_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/finite-operations");
    let mut sources = relation_tests::sources()
        .into_iter()
        .chain(domain_sources::exception_sources())
        .collect::<Vec<_>>();
    let rows = read("data-phase/data-stage-replay.json");
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "0fd1cf08493a395f3053454f01cf17fb0c4cc2687e7c290f1015215ca3e33b89")
        .unwrap();
    sources.push((
        "unit-void-source".into(),
        row.clone(),
        row["outcome"]["facts"].clone(),
    ));
    let requests = read("ordinary-foundation/finite-sources/requests.json");
    let responses = read("ordinary-foundation/finite-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    sources.push((
        "parse-error-contract".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    let mut metrics = vec![];
    let mut parse_contexts = 0;
    let mut unit_contexts = 0;
    let mut source_payloads = 0;
    let mut constructor_cases = 0;
    for (id, row, facts) in sources {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_ordinary_finite_operations(emitted.vir()).unwrap();
        if program.operations().is_empty() {
            continue;
        }
        let certificate =
            mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&certificate).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let find = |op: &str, specialization: Option<&str>| {
            program
                .operations()
                .iter()
                .find(|o| o.operation_id == op && o.specialization.as_deref() == specialization)
                .unwrap()
        };
        if types.contains_key(&ty("unit")) {
            unit_contexts += 1;
            let make = find("mpk.csharp.value.unit.v1.make", None);
            assert!(make.argument_type_ids.is_empty());
            assert!(!observed_bit(run(&certificate, &make.definition, vec![])));
            for op in ["equal", "compare"] {
                let operation = find(&format!("mpk.csharp.value.unit.v1.{op}"), None);
                let result = run(&certificate, &operation.definition, vec![V::Bit(false); 2]);
                if op == "equal" {
                    assert!(observed_bit(result));
                } else {
                    check_value(&certificate, result, &[false; 32]);
                }
            }
        }
        if types.contains_key(&ty("parse_error")) {
            parse_contexts += 1;
            let bits = |n: u32| (0..32).map(|i| n & (1 << i) != 0).collect::<Vec<_>>();
            for left in 0..5 {
                let tag = find("mpk.csharp.value.parse_error.v1.tag", None);
                assert_eq!(tag.result_type_id, ty("u32"));
                check_value(
                    &certificate,
                    run(&certificate, &tag.definition, vec![input(bits(left))]),
                    &bits(left),
                );
                for right in 0..5 {
                    let equal = find("mpk.csharp.value.parse_error.v1.equal", None);
                    assert_eq!(
                        observed_bit(run(
                            &certificate,
                            &equal.definition,
                            vec![input(bits(left)), input(bits(right))]
                        )),
                        left == right
                    );
                    let compare = find("mpk.csharp.value.parse_error.v1.compare", None);
                    let expected = if left < right {
                        u32::MAX
                    } else {
                        u32::from(left != right)
                    };
                    check_value(
                        &certificate,
                        run(
                            &certificate,
                            &compare.definition,
                            vec![input(bits(left)), input(bits(right))],
                        ),
                        &bits(expected),
                    );
                }
            }
        }
        let exception_id = ty("exception");
        if types.contains_key(&exception_id) {
            let universe = derive_closed_exception_universe(
                emitted.closure().roots(),
                emitted.closure().closed(),
                emitted.vir().source_exceptions(),
            )
            .unwrap();
            for arm in universe.arms() {
                let operation = find(
                    "mpk.csharp.value.exception.v1.construct",
                    Some(&arm.type_id),
                );
                for seed in 0..2 {
                    let payload = (arm.tag >= 9).then(|| {
                        relation_tests::sample(
                            &arm.type_id,
                            seed,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        )
                    });
                    let arguments = payload
                        .iter()
                        .map(|p| input(relation_tests::storage(p, &types)))
                        .collect();
                    let expected = MonomorphicValue::ClosedException {
                        type_id: exception_id.clone(),
                        tag: arm.tag,
                        source_type_id: (arm.tag >= 9).then(|| arm.type_id.clone()),
                        payload: payload.clone().map(Box::new),
                    };
                    validate_monomorphic_value(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &expected,
                    )
                    .unwrap();
                    let storage = relation_tests::storage(&expected, &types);
                    let value = run(&certificate, &operation.definition, arguments);
                    check_value(&certificate, value.clone(), &storage);
                    constructor_cases += 1;
                    for target in universe.arms() {
                        // Explicit CLR hierarchy cases, independent of the
                        // universe ancestry list used by the generator.
                        let inherited =
                            match target.type_id.as_str() {
                                "System.ArgumentException" => matches!(
                                    arm.type_id.as_str(),
                                    "System.ArgumentOutOfRangeException"
                                        | "System.ArgumentNullException"
                                ),
                                "System.InvalidOperationException" => arm.type_id
                                    == "System.Runtime.CompilerServices.SwitchExpressionException",
                                _ => false,
                            };
                        let predicate = find(
                            "mpk.csharp.value.exception.v1.is_type",
                            Some(&target.type_id),
                        );
                        assert_eq!(
                            observed_bit(run(
                                &certificate,
                                &predicate.definition,
                                vec![value.clone()]
                            )),
                            arm.type_id == target.type_id || inherited,
                            "{id} {} is {}",
                            arm.type_id,
                            target.type_id
                        );
                        let mut forged = storage.clone();
                        // Role and leading tag-padding selectors precede the
                        // five tag selectors, so preserve the full stride.
                        forged[31 << (types[&exception_id].depth - 5)] = true;
                        assert!(!observed_bit(run(
                            &certificate,
                            &predicate.definition,
                            vec![input(forged)]
                        )));
                    }
                    if let Some(MonomorphicValue::Product { fields, .. }) = &payload {
                        source_payloads += 1;
                        for (index, field) in fields.iter().enumerate() {
                            let get = find(
                                "mpk.csharp.value.exception.v1.payload",
                                Some(&arm.payload_member_ids[index]),
                            );
                            assert_eq!(get.active_tag_requirement, Some(arm.tag));
                            assert_eq!(get.result_type_id, arm.payload_type_ids[index]);
                            check_value(
                                &certificate,
                                run(&certificate, &get.definition, vec![value.clone()]),
                                &relation_tests::storage(&field.value, &types),
                            );
                        }
                    }
                }
            }
        }
        let metadata = program.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_finite_operations(
                &metadata,
                program.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            program
        );
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "operations",
            "certificate_sha256",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_finite_operations(
                &serde_json::to_vec(&changed).unwrap(),
                program.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        if let Some(index) = program
            .operations()
            .iter()
            .position(|o| o.active_tag_requirement.is_some())
        {
            let mut changed = data.clone();
            changed["operations"][index]["active_tag_requirement"] = Value::Null;
            assert!(import_csharp_practical_ordinary_finite_operations(
                &serde_json::to_vec(&changed).unwrap(),
                program.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = program.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_finite_operations(
            &metadata,
            &changed,
            emitted.vir()
        )
        .is_err());
        let file = format!("{id}.hex");
        let hex = program
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
        metrics.push(json!({"id":id,"file":file,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"metadata":data}));
    }
    assert_eq!(metrics.len(), 7);
    assert!(
        unit_contexts > 0 && parse_contexts == 1 && source_payloads >= 4 && constructor_cases >= 90
    );
    eprintln!("finite operations: {} contexts, {unit_contexts} unit contexts, {parse_contexts} parse-error contexts, {source_payloads} source payloads, {constructor_cases} constructor cases", metrics.len());
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
