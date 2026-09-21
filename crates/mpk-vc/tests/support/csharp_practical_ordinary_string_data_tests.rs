//! W03 SSA relations reuse scalar bodies and retain all original use-point links.
use super::*;
fn requests() -> Value {
    let mut rows = read("ordinary-foundation/string-clauses/requests.json")
        .as_array()
        .unwrap()
        .clone();
    for (id, kind) in [("native-nonnull", "string"), ("native-nullable", "string?")] {
        let source=format!("#nullable enable\nusing System;namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{kind} x=\"a\\ud800\\0\";{kind} y=\"z\";char c='\\ud800';string a=x.Substring(lower,upper);string b=string.Concat(x,y,x);string d=string.Concat(x,y,x,y);string e=$\"{{x}}{{c}}{{y}}\";return a.Length;}}}}\n");
        rows.extend(
            integer_codec_clause_tests::requests_for_source(
                vec![(id.into(), vec![truth()])],
                Some(&source),
            )
            .as_array()
            .unwrap()
            .clone(),
        );
    }
    Value::Array(rows)
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_STRING_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/string-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_string_data_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_STRING_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/string-data/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_string_data_original_source() {
    verify(true);
}
#[test]
fn csharp_03_t06_w09_string_data_candidates() {
    verify(false);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_STRING_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/string-data/responses.json")
    };
    let runtime_prefix = std::env::var("MPK_W09_STRING_DATA_RUNTIME_PREFIX").ok();
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
        let program = generate_csharp_practical_ordinary_string_data(vir).unwrap();
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
            assert_ne!(format!("{:?}", d.family), "String");
        }
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_strings(vir).unwrap();
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
            if runtime
                && runtime_prefix
                    .as_ref()
                    .is_none_or(|prefix| id.starts_with(prefix))
                && std::env::var("MPK_W09_STRING_DATA_OPERATION")
                    .map_or(true, |selected| selected == *op)
                && observed.insert((op.clone(), d.source.signature.argument_type_ids.clone()))
            {
                let operation_started = std::time::Instant::now();
                eprintln!("string runtime start: {id} {op}");
                for (case_index, inputs) in cases(op, &d.source.signature.argument_type_ids)
                    .into_iter()
                    .enumerate()
                {
                    let case_started = std::time::Instant::now();
                    eprintln!("string case start: {id} {op} [{case_index}]");
                    let verdict = evaluate_string_operation(op, &inputs, false);
                    let expected_failure = verdict.as_ref().err().map(|e| match e {
                        StringError::NullReceiver => "exception.null_receiver",
                        StringError::NullArgument => "exception.null_argument",
                        StringError::IndexOutOfRange => "index_range",
                        StringError::ArgumentOutOfRange => "exception.range",
                        StringError::OutputBound => "obligation.output_bound",
                        other => panic!("invalid test input: {other:?}"),
                    });
                    let encoded_inputs = inputs
                        .iter()
                        .zip(&d.source.signature.argument_type_ids)
                        .map(|(v, t)| encode_bits(operand(v, t)))
                        .collect::<Vec<_>>();
                    let raw = d
                        .independent_failure_definitions
                        .iter()
                        .map(|name| bit(run(&cert, name, encoded_inputs.clone())))
                        .collect::<Vec<_>>();
                    assert_eq!(
                        raw.iter()
                            .position(|&b| b)
                            .map(|i| d.source.signature.ordered_checks[i].id.as_str()),
                        expected_failure,
                        "{op}: {inputs:?}"
                    );
                    if expected_failure.is_some() {
                        eprintln!("string data {op}: {expected_failure:?}, raw conditions {raw:?}");
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
                            if let Some(goal) = o.predicates.get(&format!("check.{i}.static_goal"))
                            {
                                assert_eq!(bit(run(&cert, goal, args.clone())), previous || !fail);
                            }
                            previous |= fail;
                        }
                        observations += 1;
                    }
                    eprintln!(
                        "string case completed: {id} {op} [{case_index}] in {:.3}s",
                        case_started.elapsed().as_secs_f64()
                    );
                }
                eprintln!(
                    "string runtime completed: {id} {op} in {:.3}s",
                    operation_started.elapsed().as_secs_f64()
                );
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
            import_csharp_practical_ordinary_string_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut corrupt = metadata.clone();
        corrupt["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_string_data(
            &serde_json::to_vec(&corrupt).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bytes = program.certificate_bytes().to_vec();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(import_csharp_practical_ordinary_string_data(
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
            "string data source {id}: {} definitions, {} use points, {} pending",
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
        "string data source: {operation_count} use points, {observations} result/guard cases"
    );
    assert!(operation_count >= 21);
    eprintln!("string data unique observed operations: {}", observed.len());
    assert_eq!(observations > 0, runtime);
}

fn physical_width(ty: &str) -> usize {
    match ty {
        "mpk.csharp.value.bool.v1" => 1,
        "mpk.csharp.value.char.v1" => 16,
        "mpk.csharp.value.i32.v1" => 32,
        "mpk.csharp.value.string.v1" => 1 << 19,
        _ => panic!("unexpected string result {ty}"),
    }
}
fn encode_bits(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn word(n: u32, width: usize) -> Vec<bool> {
    (0..width).map(|i| n & (1 << i) != 0).collect()
}
fn text(units: Option<&[u16]>, nullable: bool) -> Vec<bool> {
    let mut bits = vec![false; 1 << (19 + usize::from(nullable))];
    let Some(units) = units else {
        assert!(nullable);
        return bits;
    };
    let mut set = |i, b| {
        bits[if nullable { 1 + 2 * i } else { i }] = b;
    };
    for i in 0..32 {
        set(i << 14, units.len() & (1 << i) != 0);
    }
    for (i, &unit) in units.iter().enumerate() {
        for j in 0..16 {
            set(1 + 2 * i + (j << 15), unit & (1 << j) != 0);
        }
    }
    if nullable {
        bits[0] = true;
    }
    bits
}
fn operand(v: &StringOperand, ty: &str) -> Vec<bool> {
    match v {
        StringOperand::Text { utf16 } => text(utf16.as_deref(), ty != "mpk.csharp.value.string.v1"),
        StringOperand::Char { utf16 } => word(u32::from(*utf16), 16),
        StringOperand::Index { value } => word(*value as u32, 32),
    }
}
fn physical(v: &MonomorphicValue) -> Vec<bool> {
    match v {
        MonomorphicValue::Bool { value, .. } => vec![*value],
        MonomorphicValue::Signed { value, .. } => word(value.parse::<i32>().unwrap() as u32, 32),
        MonomorphicValue::Char { utf16, .. } => word(u32::from(*utf16), 16),
        MonomorphicValue::String { utf16, .. } => text(Some(utf16), false),
        _ => panic!("unexpected string result"),
    }
}
fn cases(op: &str, types: &[String]) -> Vec<Vec<StringOperand>> {
    let mut ordinary = types
        .iter()
        .map(|ty| match ty.as_str() {
            "mpk.csharp.value.i32.v1" => StringOperand::Index { value: 1 },
            "mpk.csharp.value.char.v1" => StringOperand::Char { utf16: 0xd800 },
            _ => StringOperand::Text {
                utf16: Some(vec![97, 0xd800, 0]),
            },
        })
        .collect::<Vec<_>>();
    if ordinary.len() == 2 && matches!(ordinary[1], StringOperand::Text { .. }) {
        ordinary[1] = StringOperand::Text {
            utf16: Some(vec![97, 0xd800]),
        };
    }
    let mut cases = vec![ordinary.clone()];
    let mut variant = ordinary.clone();
    for v in &mut variant {
        if let StringOperand::Text { utf16 } = v {
            *utf16 = Some(vec![]);
        }
    }
    cases.push(variant);
    for (i, ty) in types.iter().enumerate() {
        if ty.starts_with("mpk.csharp.instance.") {
            let mut absent = ordinary.clone();
            absent[i] = StringOperand::Text { utf16: None };
            cases.push(absent);
        }
    }
    if op == "string.index" || op == "string.substring.start_length" {
        ordinary[1] = StringOperand::Index { value: -1 };
        cases.push(ordinary.clone());
    }
    if op.starts_with("string.concat.") || op.starts_with("string.interpolation.") {
        // Exact capacity plus one unit: a static output-bound obligation,
        // not an exception, and no silently shortened string result.
        if let Some(first) = ordinary
            .iter()
            .position(|v| matches!(v, StringOperand::Text { .. }))
        {
            let others = ordinary
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != first)
                .map(|(_, v)| match v {
                    StringOperand::Text { utf16 } => utf16.as_ref().map_or(0, Vec::len),
                    StringOperand::Char { .. } => 1,
                    _ => 0,
                })
                .sum::<usize>();
            if others > 0 {
                ordinary[first] = StringOperand::Text {
                    utf16: Some(vec![0; 16384]),
                };
                cases.push(ordinary);
            }
        }
    }
    cases
}
