//! W03 SSA relations reuse scalar bodies and retain all original use-point links.
use super::*;
fn requests() -> Value {
    // Reuse original captured C# programs, including constructors and each
    // unary/binary calendar contract's native operation, without recapture.
    read("ordinary-foundation/calendar-clauses/requests.json")
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/calendar-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_calendar_data_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/calendar-data/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_calendar_data_original_source() {
    verify(true);
}
#[test]
fn csharp_03_t06_w09_calendar_data_candidates() {
    verify(false);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/calendar-clauses/responses.json")
    };
    let mut operation_count = 0;
    let mut observations = 0;
    let mut observed = BTreeSet::new();
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
        let program = generate_csharp_practical_ordinary_calendar_data(vir).unwrap();
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
            assert!(
                format!("{:?}", d.family) != "CalendarTimeGuidMoney"
                    || d.signature.id.starts_with("money.")
                    || d.signature
                        .ordered_checks
                        .iter()
                        .any(|c| c.tag != RequiredCheckTag::Exception)
            );
        }
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_calendar(vir).unwrap();
        let old = mpk_cert::decode_canonical_certificate(legacy.certificate_bytes()).unwrap();
        let temporal = generate_csharp_practical_ordinary_temporal(vir).unwrap();
        let temporal_old =
            mpk_cert::decode_canonical_certificate(temporal.certificate_bytes()).unwrap();
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
                    .chain(temporal.definitions())
                    .find(|s| s.operation.id == d.source.signature.id)
                    .unwrap()
            );
            assert_eq!(
                d.independent_failure_definitions.len(),
                d.source.failure_names.len()
            );
            roots.insert(d.scalar.result_definition.clone());
            roots.insert(d.scalar.success_definition.clone());
            roots.extend(d.scalar.ordered_failure_definitions.iter().cloned());
        }
        for previous in [&old, &temporal_old] {
            let names = previous
                .declarations
                .iter()
                .map(|d| previous.name_table[d.name as usize].clone())
                .collect::<BTreeSet<_>>();
            let selected = roots.intersection(&names).cloned().collect();
            structural_equivalence_tests::same_definition_closure(previous, &cert, &selected)
                .unwrap();
        }
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
            if runtime && observed.insert(op.clone()) {
                let recipe = BusinessOperation::new(
                    op,
                    &d.source.signature.argument_type_ids,
                    &d.source.signature.normal_result_type_id,
                )
                .unwrap();
                for inputs in cases(op, &d.source.signature.argument_type_ids) {
                    let verdict = recipe.evaluate(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &inputs,
                    );
                    let expected_failure = verdict
                        .as_ref()
                        .err()
                        .map(|e| e.exception_type().expect("calendar/time exception"));
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
                            "calendar data {op}: {expected_failure:?}, raw conditions {raw:?}"
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
                            let index = actual.len() - 1;
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
            import_csharp_practical_ordinary_calendar_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut corrupt = metadata.clone();
        corrupt["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_calendar_data(
            &serde_json::to_vec(&corrupt).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bytes = program.certificate_bytes().to_vec();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(import_csharp_practical_ordinary_calendar_data(
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
            "calendar data source {id}: {} definitions, {} use points, {} pending",
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
        "calendar data source: {operation_count} use points, {observations} result/guard cases"
    );
    assert!(operation_count >= 50);
    eprintln!(
        "calendar data unique observed operations: {}",
        observed.len()
    );
    assert_eq!(observations > 0, runtime);
}

fn physical_width(ty: &str) -> usize {
    match ty
        .strip_prefix("mpk.csharp.value.")
        .unwrap()
        .strip_suffix(".v1")
        .unwrap()
    {
        "bool" => 1,
        "i32" | "date" | "day_of_week" => 32,
        "i64" | "time" | "duration" | "instant" => 64,
        "guid" => 128,
        _ => panic!("unexpected calendar data type {ty}"),
    }
}
fn encode_bits(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn physical(value: &MonomorphicValue) -> Vec<bool> {
    let n = match value {
        MonomorphicValue::Bool { value, .. } => u128::from(*value),
        MonomorphicValue::Signed { value, .. } => value.parse::<i128>().unwrap() as u128,
        MonomorphicValue::Date { day_number, .. } => u128::from(*day_number),
        MonomorphicValue::Time { ticks, .. } | MonomorphicValue::Duration { ticks, .. } => {
            ticks.parse::<i128>().unwrap() as u128
        }
        MonomorphicValue::Instant { milliseconds, .. } => {
            milliseconds.parse::<i128>().unwrap() as u128
        }
        MonomorphicValue::Guid { n, .. } => u128::from_str_radix(n, 16).unwrap(),
        MonomorphicValue::Enum { carrier, .. } => carrier.parse::<i128>().unwrap() as u128,
        _ => panic!("unexpected calendar operand"),
    };
    (0..physical_width(value.type_id()))
        .map(|i| n & (1u128 << i) != 0)
        .collect()
}
fn value(id: &str, n: i128) -> MonomorphicValue {
    match id
        .strip_prefix("mpk.csharp.value.")
        .unwrap()
        .strip_suffix(".v1")
        .unwrap()
    {
        "bool" => MonomorphicValue::Bool {
            type_id: id.into(),
            value: n != 0,
        },
        "date" => MonomorphicValue::Date {
            type_id: id.into(),
            day_number: n.try_into().unwrap(),
        },
        "time" => MonomorphicValue::Time {
            type_id: id.into(),
            ticks: n.to_string(),
        },
        "duration" => MonomorphicValue::Duration {
            type_id: id.into(),
            ticks: n.to_string(),
        },
        "instant" => MonomorphicValue::Instant {
            type_id: id.into(),
            milliseconds: n.to_string(),
        },
        "guid" => MonomorphicValue::Guid {
            type_id: id.into(),
            n: format!("{:032x}", n as u128),
        },
        "day_of_week" => MonomorphicValue::Enum {
            type_id: id.into(),
            underlying: "i32".into(),
            carrier: n.to_string(),
        },
        "i32" | "i64" => MonomorphicValue::Signed {
            type_id: id.into(),
            value: n.to_string(),
        },
        _ => panic!("unexpected calendar operand type {id}"),
    }
}
fn cases(op: &str, types: &[String]) -> Vec<Vec<MonomorphicValue>> {
    let numbers: Vec<Vec<i128>> = match op {
        "date.construct" => vec![vec![2024, 2, 29], vec![1900, 2, 29]],
        "date.add_days" => vec![vec![738944, 1], vec![3652058, 1]],
        "date.add_months" => vec![vec![738915, 1], vec![0, -1]],
        "date.add_years" => vec![vec![738944, 1], vec![3652058, 1]],
        "time.construct" => vec![vec![1], vec![864000000000]],
        "duration.construct" => vec![vec![-1], vec![i64::MAX as i128]],
        "duration.add" => vec![vec![-1, 2], vec![i64::MAX as i128, 1]],
        "duration.subtract" => vec![vec![1, 2], vec![i64::MIN as i128, 1]],
        "duration.negate" => vec![vec![1], vec![i64::MIN as i128]],
        "time.add_duration" | "time.subtract_duration" => vec![vec![0, 1], vec![1, -2]],
        "time.subtract" => vec![vec![0, 1]],
        "guid.empty" => vec![vec![]],
        _ => {
            let a = types
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    if t.ends_with(".date.v1") {
                        738915 + i as i128
                    } else if t.ends_with(".duration.v1") {
                        -900610010000 + i as i128
                    } else {
                        1 + i as i128
                    }
                })
                .collect();
            vec![a]
        }
    };
    numbers
        .into_iter()
        .map(|ns| {
            assert_eq!(ns.len(), types.len(), "{op}");
            types.iter().zip(ns).map(|(t, n)| value(t, n)).collect()
        })
        .collect()
}
