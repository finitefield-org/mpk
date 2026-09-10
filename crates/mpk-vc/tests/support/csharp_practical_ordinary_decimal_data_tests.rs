//! W03 SSA relations reuse scalar bodies and retain all original use-point links.
use super::*;
fn requests() -> Value {
    json!([
        ("add", "decimal a=lower;decimal b=upper;decimal c=a+b;return (int)c;"),
        ("divide", "decimal a=lower;decimal b=upper;decimal c=a/b;return (int)c;"),
        ("round", "decimal a=lower;decimal c=decimal.Round(a,upper,System.MidpointRounding.ToEven);return c<a?lower:upper;")
    ].into_iter().flat_map(|(id,body)| {
        let source=format!("namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{body}}}}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id.into(),vec![truth()])],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_decimal_data_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/decimal-data/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_decimal_data_original_source() {
    verify(true);
}
#[test]
fn csharp_03_t06_w09_decimal_data_candidates() {
    verify(false);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/decimal-data/responses.json")
    };
    let mut operation_count = 0;
    let mut observations = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none(), "{id}: {response}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let program = generate_csharp_practical_ordinary_decimal_data(vir).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let metadata: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        assert_eq!(metadata["data_vc_sha256"], data.hash());
        let included = program
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        let expected_pending = data
            .definitions()
            .iter()
            .filter(|d| !included.contains(d.id.as_str()))
            .map(|d| d.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(program.pending_definition_ids(), expected_pending);
        for d in data
            .definitions()
            .iter()
            .filter(|d| expected_pending.contains(&d.id))
        {
            assert_ne!(format!("{:?}", d.family), "FloatingDecimal");
        }
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_decimal(vir).unwrap();
        let old = mpk_cert::decode_canonical_certificate(legacy.certificate_bytes()).unwrap();
        let mut roots = BTreeSet::new();
        for d in program.definitions() {
            assert_eq!(
                &d.source,
                data.definitions()
                    .iter()
                    .find(|o| o.id == d.source.id)
                    .unwrap()
            );
            assert_eq!(
                &d.scalar,
                legacy
                    .definitions()
                    .iter()
                    .find(|s| s.operation.id == d.source.signature.id)
                    .unwrap()
            );
            assert_eq!(
                d.independent_failure_definitions.len(),
                d.source.failure_names.len()
            );
            if d.source.signature.id == "decimal.divide" {
                assert_ne!(
                    d.independent_failure_definitions,
                    d.scalar.ordered_failure_definitions
                );
            }
            roots.insert(d.scalar.result_definition.clone());
            roots.insert(d.scalar.success_definition.clone());
            roots.extend(d.scalar.ordered_failure_definitions.iter().cloned());
        }
        structural_equivalence_tests::same_definition_closure(&old, &cert, &roots).unwrap();
        for o in program.operations() {
            assert_eq!(
                &o.source,
                data.operations()
                    .iter()
                    .find(|source| source.id == o.source.id)
                    .unwrap()
            );
            let d = program
                .definitions()
                .iter()
                .find(|d| d.source.id == o.source.definition_id)
                .unwrap();
            let op = &d.source.signature.id;
            if runtime {
                let recipe = NumericOperation::new(
                    op,
                    &d.source.signature.argument_type_ids,
                    &d.source.signature.normal_result_type_id,
                    None,
                )
                .unwrap();
                for inputs in cases(op) {
                    let verdict = recipe.evaluate(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &inputs,
                    );
                    let expected_failure = verdict
                        .as_ref()
                        .err()
                        .map(|e| e.exception_type().expect("numeric exception"));
                    let encoded_inputs = inputs
                        .iter()
                        .map(|v| encode_bits(physical(v)))
                        .collect::<Vec<_>>();
                    let raw = d
                        .independent_failure_definitions
                        .iter()
                        .map(|name| bit(run(&cert, name, encoded_inputs.clone())))
                        .collect::<Vec<_>>();
                    assert_eq!(
                        raw.iter()
                            .position(|&b| b)
                            .map(|i| d.source.signature.ordered_checks[i]
                                .failure_type_id
                                .as_deref()
                                .unwrap()),
                        expected_failure,
                        "{op}: {inputs:?}"
                    );
                    if expected_failure.is_some() {
                        eprintln!(
                            "decimal data {op}: {expected_failure:?}, raw conditions {raw:?}"
                        );
                    }
                    let result_type = &d.source.signature.normal_result_type_id;
                    let normal = verdict
                        .as_ref()
                        .map(physical)
                        .unwrap_or_else(|_| vec![false; physical_width(result_type)]);
                    for wrong in [false, true] {
                        let mut actual = normal.clone();
                        if wrong {
                            let index = if actual.len() == 512 {
                                382
                            } else {
                                actual.len() - 1
                            };
                            actual[index] = !actual[index];
                        }
                        let mut args = encoded_inputs.clone();
                        args.push(encode_bits(actual));
                        assert_eq!(args.len(), o.source.subjects.len());
                        let failed = expected_failure.is_some();
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_guard"], args.clone())),
                            !failed,
                            "{op}"
                        );
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                            failed || !wrong,
                            "{op}"
                        );
                        if !failed {
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_relation"], args.clone())),
                                !wrong,
                                "{op}: {inputs:?}"
                            );
                        }
                        let mut previous = false;
                        for (i, fail) in raw.iter().copied().enumerate() {
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.prefix")],
                                    args.clone()
                                )),
                                !previous
                            );
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.failed")],
                                    args.clone()
                                )),
                                fail
                            );
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.guard")],
                                    args.clone()
                                )),
                                !previous && fail
                            );
                            previous |= fail;
                        }
                        observations += 1;
                    }
                }
            }
            operation_count += 1;
        }
        assert_eq!(
            program.operations().len(),
            data.operations()
                .iter()
                .filter(|o| included.contains(o.definition_id.as_str()))
                .count()
        );
        assert_eq!(
            import_csharp_practical_ordinary_decimal_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut corrupt = metadata.clone();
        corrupt["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_decimal_data(
            &serde_json::to_vec(&corrupt).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bytes = program.certificate_bytes().to_vec();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(import_csharp_practical_ordinary_decimal_data(
            &program.canonical_bytes(),
            &bytes,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &program.canonical_bytes());
        output(
            &format!("{id}.hex"),
            program
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        eprintln!(
            "decimal data source {id}: {} definitions, {} use points, {} pending",
            program.definitions().len(),
            program.operations().len(),
            program.pending_definition_ids().len()
        );
    }
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
    eprintln!(
        "decimal data source: {operation_count} use points, {observations} result/guard cases"
    );
    assert!(operation_count >= 9);
    assert_eq!(observations > 0, runtime);
}

fn physical_width(ty: &str) -> usize {
    if ty.ends_with(".decimal.v1") {
        512
    } else if ty.ends_with(".bool.v1") {
        1
    } else {
        assert!(ty.ends_with(".i32.v1"));
        32
    }
}
fn encode_bits(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
// Independently encode the frozen product address mapping, including zero padding.
fn physical(value: &MonomorphicValue) -> Vec<bool> {
    match value {
        MonomorphicValue::DecimalBits {
            negative,
            coefficient,
            scale,
            ..
        } => {
            let mut bits = vec![false; 512];
            bits[0] = *negative;
            for i in 0..8 {
                bits[1 + 64 * i] = scale & (1 << i) != 0;
            }
            let n = coefficient.parse::<u128>().unwrap();
            for i in 0..96 {
                bits[2 + 4 * i] = n & (1u128 << i) != 0;
            }
            bits
        }
        MonomorphicValue::Signed { type_id, value } => {
            assert_eq!(physical_width(type_id), 32);
            let n = value.parse::<i32>().unwrap() as u32;
            (0..32).map(|i| n & (1 << i) != 0).collect()
        }
        MonomorphicValue::Bool { value, .. } => vec![*value],
        _ => panic!("unexpected numeric operand"),
    }
}
fn dec(negative: bool, coefficient: u128, scale: u8) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        negative,
        coefficient: coefficient.to_string(),
        scale,
    }
}
fn int(n: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: ty("i32"),
        value: n.to_string(),
    }
}
fn cases(op: &str) -> Vec<Vec<MonomorphicValue>> {
    let max = (1u128 << 96) - 1;
    match op {
        "decimal.conversion.int32_to_decimal" => [0, -7, i32::MIN, i32::MAX]
            .into_iter()
            .map(|v| vec![int(v)])
            .collect(),
        "decimal.conversion.decimal_to_int32" => [
            dec(false, 775, 2),
            dec(false, 2147483648, 0),
            dec(true, 2147483648, 0),
            dec(true, 0, 28),
        ]
        .into_iter()
        .map(|v| vec![v])
        .collect(),
        "decimal.add" => vec![
            vec![dec(false, 1, 0), dec(false, 2, 0)],
            vec![dec(false, max, 0), dec(false, 1, 0)],
            vec![dec(true, 0, 28), dec(false, 0, 0)],
        ],
        "decimal.divide" => vec![
            vec![dec(false, 1, 0), dec(false, 2, 0)],
            vec![dec(false, 1, 0), dec(false, 0, 0)],
            vec![dec(false, max, 0), dec(false, 1, 28)],
        ],
        "decimal.round.ToEven.2" => vec![
            vec![dec(false, 25, 1), int(0)],
            vec![dec(true, 0, 28), int(28)],
            vec![dec(false, 1, 0), int(29)],
            vec![dec(false, 1, 0), int(-1)],
        ],
        "decimal.less" => vec![
            vec![dec(false, 1, 0), dec(false, 10, 1)],
            vec![dec(true, 1, 0), dec(false, 1, 0)],
            vec![dec(false, 0, 0), dec(true, 0, 28)],
        ],
        other => panic!("unexpected native decimal operation {other}"),
    }
}
