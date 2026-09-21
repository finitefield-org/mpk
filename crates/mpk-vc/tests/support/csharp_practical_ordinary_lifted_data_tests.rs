//! Original-source lifted SSA relations, including null propagation and guards.
use super::super::relation_tests::storage;
use super::*;

fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}

fn requests() -> Value {
    let cases = [
        ("bool", "bool? a=lower>0;bool? b=upper>0;bool? x=!a;bool? y=a&b;bool? z=a|b;bool e=a==b;bool n=a!=b;"),
        ("i32-checked", "int? a=lower;int? b=upper;int? p=+a;int? n=-a;int? x=a+b;int? y=a-b;int? z=a*b;int? q=a/b;int? r=a%b;bool e=a==b;bool ne=a!=b;bool l=a<b;bool le=a<=b;bool g=a>b;bool ge=a>=b;"),
        ("i32-unchecked", "int? a=lower;int? b=upper;unchecked{int? n=-a;int? x=a+b;int? y=a-b;int? z=a*b;int? q=a/b;int? r=a%b;}"),
        ("i64", "long? a=(long)lower;long? b=(long)upper;long? n=-a;long? x=a+b;long? q=a/b;bool e=a==b;bool l=a<b;"),
        ("f32", "float? a=(float)lower;float? b=(float)upper;bool e=a==b;bool ne=a!=b;bool l=a<b;bool le=a<=b;bool g=a>b;bool ge=a>=b;"),
        ("f64", "double? a=1.0;double? b=-1.0;bool e=a==b;bool ne=a!=b;bool l=a<b;bool le=a<=b;bool g=a>b;bool ge=a>=b;"),
        ("decimal", "decimal? a=(decimal)lower;decimal? b=(decimal)upper;bool e=a==b;bool ne=a!=b;bool l=a<b;bool le=a<=b;bool g=a>b;bool ge=a>=b;")
    ];
    Value::Array(cases.into_iter().flat_map(|(id, body)| {
        let source = format!("namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{body}return lower;}}}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id.into(),vec![truth()])],Some(&source)).as_array().unwrap().clone()
    }).collect())
}
fn output(name: &str, bytes: &[u8]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/lifted-data");
    if let Some(path) = std::env::var_os("MPK_W09_LIFTED_DATA_OUT") {
        fs::create_dir_all(&path).unwrap();
        fs::write(Path::new(&path).join(name), bytes).unwrap();
    } else {
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_lifted_data_requests() {
    let r = requests();
    if let Some(path) = std::env::var_os("MPK_W09_LIFTED_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&r).unwrap()).unwrap();
    } else {
        assert_eq!(read("ordinary-foundation/lifted-data/requests.json"), r);
    }
}
#[test]
fn csharp_03_t06_w09_lifted_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_lifted_data_original_source() {
    verify(true);
}
fn payloads(token: &str) -> Vec<MonomorphicValue> {
    let ty = format!("mpk.csharp.value.{token}.v1");
    let values = match token {
        "bool" => vec![json!(false), json!(true)],
        "i32" => [0i64, 1, -1, i32::MIN as i64, i32::MAX as i64]
            .into_iter()
            .map(|v| json!(v.to_string()))
            .collect(),
        "i64" => [0i64, 1, -1, i64::MIN, i64::MAX]
            .into_iter()
            .map(|v| json!(v.to_string()))
            .collect(),
        "f32" => ["00000000", "80000000", "3f800000", "7fc00001", "7f800000"]
            .into_iter()
            .map(|v| json!(v))
            .collect(),
        "f64" => [
            "0000000000000000",
            "8000000000000000",
            "3ff0000000000000",
            "7ff8000000000001",
            "7ff0000000000000",
        ]
        .into_iter()
        .map(|v| json!(v))
        .collect(),
        "decimal" => vec![
            json!({"negative":false,"coefficient":"0","scale":0}),
            json!({"negative":true,"coefficient":"0","scale":0}),
            json!({"negative":false,"coefficient":"10","scale":1}),
        ],
        _ => panic!("{token}"),
    };
    values
        .into_iter()
        .map(|value| {
            let kind = match token {
                "bool" => "bool",
                "i32" | "i64" => "signed",
                "f32" => "f32_bits",
                "f64" => "f64_bits",
                _ => "decimal_bits",
            };
            let v = match kind {
                "decimal_bits" => {
                    let mut v = value;
                    v["kind"] = json!(kind);
                    v["type_id"] = json!(ty);
                    v
                }
                "f32_bits" | "f64_bits" => json!({"kind":kind,"type_id":ty,"bits":value}),
                _ => json!({"kind":kind,"type_id":ty,"value":value}),
            };
            serde_json::from_value(v).unwrap()
        })
        .collect()
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_LIFTED_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/lifted-data/responses.json")
    };
    let mut observations = 0;
    let mut operations = BTreeSet::new();
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
        let program = generate_csharp_practical_ordinary_lifted_data(vir).unwrap();
        assert!(!program.definitions().is_empty(), "{id}");
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let included = program
            .definitions()
            .iter()
            .map(|d| d.source.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            included,
            data.definitions()
                .iter()
                .filter(|d| d.signature.id.starts_with("lifted."))
                .map(|d| d.id.clone())
                .collect()
        );
        assert_eq!(
            program.pending_definition_ids(),
            data.definitions()
                .iter()
                .filter(|d| !included.contains(&d.id))
                .map(|d| d.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            program.operations().len(),
            data.operations()
                .iter()
                .filter(|o| included.contains(&o.definition_id))
                .count()
        );
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let selected = std::env::var("MPK_W09_LIFTED_DATA_CONTEXT").map_or(true, |v| v == id);
        for o in program.operations() {
            assert_eq!(
                &o.source,
                data.operations()
                    .iter()
                    .find(|x| x.id == o.source.id)
                    .unwrap()
            );
            let d = program
                .definitions()
                .iter()
                .find(|d| d.source.id == o.source.definition_id)
                .unwrap();
            assert_eq!(
                &d.source,
                data.definitions()
                    .iter()
                    .find(|x| x.id == d.source.id)
                    .unwrap()
            );
            let selected_operation = std::env::var("MPK_W09_LIFTED_DATA_OPERATION")
                .map_or(true, |id| id == d.source.signature.id);
            if !runtime
                || !selected
                || !selected_operation
                || !operations.insert(d.source.signature.id.clone())
            {
                continue;
            }
            let started = std::time::Instant::now();
            eprintln!("lifted runtime start: {}", d.source.signature.id);
            let parts = d.source.signature.id.split('.').collect::<Vec<_>>();
            let model = OutcomeModel::new(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.source.signature.argument_type_ids[0],
            )
            .unwrap();
            let mut values = vec![model.construct("none", None).unwrap()];
            values.extend(
                payloads(parts[1])
                    .into_iter()
                    .map(|v| model.construct("some", Some(v)).unwrap()),
            );
            let arity = d.source.signature.argument_type_ids.len();
            for (left_index, left) in values.iter().enumerate() {
                for (right_index, right) in values
                    .iter()
                    .take(if arity == 1 { 1 } else { values.len() })
                    .enumerate()
                {
                    if std::env::var("MPK_W09_LIFTED_DATA_CASE")
                        .is_ok_and(|case| case != format!("{left_index},{right_index}"))
                    {
                        continue;
                    }
                    let case_started = std::time::Instant::now();
                    eprintln!(
                        "lifted case start: {} [{left_index},{right_index}]",
                        d.source.signature.id
                    );
                    let inputs = if arity == 1 {
                        vec![left.clone()]
                    } else {
                        vec![left.clone(), right.clone()]
                    };
                    let verdict = model.lift(parts[2], &inputs, parts[3] == "checked");
                    let failure = verdict.as_ref().err().map(|e| e.exception_type().unwrap());
                    let encoded = inputs
                        .iter()
                        .map(|v| input(storage(v, &types)))
                        .collect::<Vec<_>>();
                    let raw = d
                        .independent_failure_definitions
                        .iter()
                        .map(|name| bit(run(&cert, name, encoded.clone())))
                        .collect::<Vec<_>>();
                    assert_eq!(
                        raw.iter()
                            .position(|b| *b)
                            .map(|i| d.source.signature.ordered_checks[i]
                                .failure_type_id
                                .as_deref()
                                .unwrap()),
                        failure,
                        "{} {inputs:?}",
                        d.source.signature.id
                    );
                    let normal =
                        verdict
                            .as_ref()
                            .map(|v| storage(v, &types))
                            .unwrap_or_else(|_| {
                                vec![
                                    false;
                                    1 << types[&d.source.signature.normal_result_type_id].depth
                                ]
                            });
                    for wrong in [false, true] {
                        let mut actual = normal.clone();
                        if wrong {
                            let end = actual.len() - 1;
                            actual[end] = !actual[end];
                        }
                        let mut args = encoded.clone();
                        args.push(input(actual));
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_guard"], args.clone())),
                            failure.is_none()
                        );
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                            failure.is_some() || !wrong
                        );
                        if failure.is_none() {
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_relation"], args.clone())),
                                !wrong,
                                "{} {inputs:?}",
                                d.source.signature.id
                            );
                        }
                        let mut previous = false;
                        for (i, failed) in raw.iter().enumerate() {
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
                                *failed
                            );
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.guard")],
                                    args.clone()
                                )),
                                !previous && *failed
                            );
                            previous |= *failed;
                        }
                        observations += 1;
                    }
                    eprintln!(
                        "lifted case completed: {} [{left_index},{right_index}] in {:.3}s",
                        d.source.signature.id,
                        case_started.elapsed().as_secs_f64()
                    );
                }
            }
            eprintln!(
                "lifted runtime completed: {} in {:.3}s",
                d.source.signature.id,
                started.elapsed().as_secs_f64()
            );
        }
        assert_eq!(
            import_csharp_practical_ordinary_lifted_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut bad: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        bad["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_lifted_data(
            &serde_json::to_vec(&bad).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut corrupt = program.certificate_bytes().to_vec();
        let end = corrupt.len() - 1;
        corrupt[end] ^= 1;
        assert!(import_csharp_practical_ordinary_lifted_data(
            &program.canonical_bytes(),
            &corrupt,
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
            "lifted data {id}: {} definitions, {} SSA points, {observations} observations",
            program.definitions().len(),
            program.operations().len()
        );
    }
    if runtime {
        assert!(
            observations > 0,
            "runtime selector matched no lifted operation"
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
}
