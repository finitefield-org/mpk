//! W03 SSA relations reuse scalar bodies and retain all original use-point links.
use super::*;
fn requests() -> Value {
    json!([
        ("arithmetic", "int sum=lower+upper;int quotient=lower/upper;int rem=lower%upper;return sum+quotient+rem;"),
        ("boolean", "bool a=lower<upper;bool b=lower==upper;bool c=a^b;return c?lower:upper;"),
        ("pending-string", "string value=\"ab\";return lower+value.Length;")
    ].into_iter().flat_map(|(id,body)| {
        let source=format!("namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{body}}}}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id.into(),vec![truth()])],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_INTEGER_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_integer_data_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_INTEGER_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/integer-data/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_integer_data_original_source() {
    verify(true);
}
#[test]
fn csharp_03_t06_w09_integer_data_candidates() {
    verify(false);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_INTEGER_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/integer-data/responses.json")
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
        let program = generate_csharp_practical_ordinary_integer_data(vir).unwrap();
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
        assert_eq!(!expected_pending.is_empty(), id != "arithmetic");
        for d in data
            .definitions()
            .iter()
            .filter(|d| expected_pending.contains(&d.id))
        {
            let expected = if id == "boolean" {
                "Structural"
            } else {
                "String"
            };
            assert_eq!(format!("{:?}", d.family), expected);
        }
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_integers(vir).unwrap();
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
                for (left, right) in [(7i32, 3i32), (i32::MAX, 1), (i32::MIN, -1), (1, 0), (0, 0)] {
                    let (result, zero, overflow) = if op == "boolean.xor" {
                        ((left != 0) ^ (right != 0), false, false)
                    } else {
                        (false, false, false)
                    };
                    let (normal, zero, overflow) = if op == "boolean.xor" {
                        (i32::from(result), zero, overflow)
                    } else {
                        match op.split('.').nth(2).unwrap() {
                            "add" => {
                                let (n, e) = left.overflowing_add(right);
                                (n, false, e)
                            }
                            "divide" => (
                                left.checked_div(right).unwrap_or(0),
                                right == 0,
                                left == i32::MIN && right == -1,
                            ),
                            "remainder" => (
                                left.checked_rem(right).unwrap_or(0),
                                right == 0,
                                left == i32::MIN && right == -1,
                            ),
                            "equal" => (i32::from(left == right), false, false),
                            "less" => (i32::from(left < right), false, false),
                            other => panic!("unexpected native operation {other}: {op}"),
                        }
                    };
                    let failed = zero || overflow;
                    for wrong in [false, true] {
                        let value = if wrong { normal ^ 1 } else { normal };
                        let encoded = |n: i32, ty: &str| {
                            if ty.ends_with(".bool.v1") {
                                V::Bit(n != 0)
                            } else {
                                V::Cube((0..32).map(|i| (n as u32) & (1 << i) != 0).collect())
                            }
                        };
                        let args = o
                            .source
                            .subjects
                            .iter()
                            .enumerate()
                            .map(|(i, s)| encoded([left, right, value][i], &s.type_id))
                            .collect::<Vec<_>>();
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
                                "{op}"
                            );
                        }
                        let mut previous = false;
                        for (i, check) in o.source.checks.iter().enumerate() {
                            let fail = match check.check.id.as_str() {
                                "exception.overflow" => overflow,
                                "exception.division_by_zero" => zero,
                                _ => panic!("unexpected check"),
                            };
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
            import_csharp_practical_ordinary_integer_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut corrupt = metadata.clone();
        corrupt["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_integer_data(
            &serde_json::to_vec(&corrupt).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bytes = program.certificate_bytes().to_vec();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(import_csharp_practical_ordinary_integer_data(
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
            "integer data source {id}: {} definitions, {} use points, {} pending",
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
        "integer data source: {operation_count} use points, {observations} result/guard cases"
    );
    assert!(operation_count >= 7);
    assert_eq!(observations, if runtime { operation_count * 10 } else { 0 });
}
