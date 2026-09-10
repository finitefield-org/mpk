//! Native calendar/temporal contract connections; application proofs are separate.
use super::fixed_codec_clause_tests::alias;
use super::*;
#[derive(Clone)]
struct Case {
    operation: String,
    result: &'static str,
    arguments: Vec<J>,
    expected: J,
    native: String,
    defined: bool,
    observe: bool,
}
fn literal(token: &str, value: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        (
            "value",
            if token == "bool" {
                J::Bool(value == "true")
            } else {
                J::string(value)
            },
        ),
    ])
}
impl Case {
    fn clause(&self) -> J {
        let mut fields = vec![
            (
                "tag",
                J::string(if self.arguments.len() == 1 {
                    "unary"
                } else {
                    "binary"
                }),
            ),
            ("type_id", J::string(ty(self.result))),
            ("operation_id", J::string(&self.operation)),
        ];
        if self.arguments.len() == 1 {
            fields.push(("operand", self.arguments[0].clone()));
        } else {
            assert_eq!(self.arguments.len(), 2);
            fields.extend([
                ("left", self.arguments[0].clone()),
                ("right", self.arguments[1].clone()),
            ]);
        }
        J::object(vec![
            ("tag", J::string("structural_equal")),
            ("type_id", J::string(ty("bool"))),
            ("left", J::object(fields)),
            ("right", self.expected.clone()),
        ])
    }
}
fn groups() -> BTreeMap<String, Vec<Case>> {
    let mut groups = BTreeMap::<String, Vec<Case>>::new();
    let mut add = |group: &str,
                   token: &str,
                   op: &str,
                   result,
                   arguments: Vec<J>,
                   expected: &str,
                   native: String,
                   defined,
                   observe| {
        groups.entry(group.into()).or_default().push(Case {
            operation: format!("{token}.{op}"),
            result,
            arguments,
            expected: literal(result, expected),
            native,
            defined,
            observe,
        });
    };
    for (token, x, y) in [
        ("date", "2024-01-31", "2024-02-29"),
        ("time", "01:02:03.0040000", "02:00:00.0000000"),
        ("duration", "-900610010000", "10000"),
        (
            "guid",
            "00000000000000000000000000000001",
            "00000000000000000000000000000002",
        ),
    ] {
        let group = if token == "date" {
            "date-comparisons"
        } else {
            token
        };
        for (op, operator, expected) in [
            ("compare", "", "-1"),
            ("equal", "==", "false"),
            ("not_equal", "!=", "true"),
            ("less", "<", "true"),
            ("less_equal", "<=", "true"),
            ("greater", ">", "false"),
            ("greater_equal", ">=", "false"),
        ] {
            if token == "guid" && !matches!(op, "compare" | "equal" | "not_equal") {
                continue;
            }
            add(
                group,
                token,
                op,
                if op == "compare" { "i32" } else { "bool" },
                vec![literal(token, x), literal(token, y)],
                expected,
                if op == "compare" {
                    "x.CompareTo(y)".into()
                } else {
                    format!("x{operator}y")
                },
                true,
                token == "guid" && op == "compare",
            );
        }
    }
    for (op, property, result, expected) in [
        ("year", "Year", "i32", "2024"),
        ("month", "Month", "i32", "1"),
        ("day", "Day", "i32", "31"),
        ("day_number", "DayNumber", "i32", "738915"),
        ("day_of_week", "DayOfWeek", "day_of_week", "3"),
    ] {
        add(
            "date-parts",
            "date",
            op,
            result,
            vec![literal("date", "2024-01-31")],
            expected,
            format!("x.{property}"),
            true,
            op == "year",
        );
    }
    for (op, method, expected) in [
        ("add_days", "AddDays", "2024-02-01"),
        ("add_months", "AddMonths", "2024-02-29"),
        ("add_years", "AddYears", "2025-01-31"),
    ] {
        let group = format!("date-{op}");
        add(
            &group,
            "date",
            op,
            "date",
            vec![literal("date", "2024-01-31"), literal("i32", "1")],
            expected,
            format!("x.{method}(1)"),
            true,
            true,
        );
        if op == "add_days" {
            add(
                &group,
                "date",
                op,
                "date",
                vec![literal("date", "9999-12-31"), literal("i32", "1")],
                "0001-01-01",
                format!("x.{method}(1)"),
                false,
                true,
            );
        }
    }
    for (token, input, output, native) in [
        (
            "time",
            "37230040000",
            "01:02:03.0040000",
            "new TimeOnly(37230040000L)",
        ),
        (
            "duration",
            "-900610010000",
            "-900610010000",
            "new TimeSpan(-900610010000L)",
        ),
    ] {
        add(
            token,
            token,
            "construct",
            token,
            vec![literal("i64", input)],
            output,
            native.into(),
            true,
            token == "time",
        );
    }
    add(
        "time",
        "time",
        "construct",
        "time",
        vec![literal("i64", "-1")],
        "00:00:00.0000000",
        "new TimeOnly(37230040000L)".into(),
        false,
        true,
    );
    for (op, property, result, expected) in [
        ("ticks", "Ticks", "i64", "37230040000"),
        ("hour", "Hour", "i32", "1"),
        ("minute", "Minute", "i32", "2"),
        ("second", "Second", "i32", "3"),
        ("millisecond", "Millisecond", "i32", "4"),
    ] {
        add(
            "time",
            "time",
            op,
            result,
            vec![literal("time", "01:02:03.0040000")],
            expected,
            format!("x.{property}"),
            true,
            op == "hour",
        );
    }
    add(
        "time",
        "time",
        "add_duration",
        "time",
        vec![
            literal("time", "01:02:03.0040000"),
            literal("duration", "-10000"),
        ],
        "01:02:03.0030000",
        "x.Add(new TimeSpan(-10000L))".into(),
        true,
        false,
    );
    add(
        "time",
        "time",
        "subtract",
        "duration",
        vec![
            literal("time", "01:02:03.0040000"),
            literal("time", "02:00:00.0000000"),
        ],
        "829230040000",
        "x-y".into(),
        true,
        true,
    );
    for (op, property, result, expected) in [
        ("ticks", "Ticks", "i64", "-900610010000"),
        ("days", "Days", "i32", "-1"),
        ("hours", "Hours", "i32", "-1"),
        ("minutes", "Minutes", "i32", "-1"),
        ("seconds", "Seconds", "i32", "-1"),
        ("milliseconds", "Milliseconds", "i32", "-1"),
    ] {
        add(
            "duration",
            "duration",
            op,
            result,
            vec![literal("duration", "-900610010000")],
            expected,
            format!("x.{property}"),
            true,
            op == "days",
        );
    }
    for (op, operator, expected) in [
        ("add", "+", "-900610000000"),
        ("subtract", "-", "-900610020000"),
        ("negate", "-", "900610010000"),
    ] {
        let mut args = vec![literal("duration", "-900610010000")];
        if op != "negate" {
            args.push(literal("duration", "10000"));
        }
        add(
            "duration",
            "duration",
            op,
            "duration",
            args,
            expected,
            if op == "negate" {
                "-x".into()
            } else {
                format!("x{operator}y")
            },
            true,
            op == "negate",
        );
    }
    add(
        "duration",
        "duration",
        "negate",
        "duration",
        vec![literal("duration", "-9223372036854775808")],
        "0",
        "-x".into(),
        false,
        true,
    );
    add(
        "duration",
        "duration",
        "add",
        "duration",
        vec![
            literal("duration", "9223372036854775807"),
            literal("duration", "1"),
        ],
        "0",
        "x+y".into(),
        false,
        true,
    );
    assert_eq!(groups.len(), 8);
    assert_eq!(
        groups
            .values()
            .flatten()
            .map(|c| &c.operation)
            .collect::<BTreeSet<_>>()
            .len(),
        50
    );
    assert_eq!(groups.values().map(Vec::len).sum::<usize>(), 54);
    groups
}
fn native_type(token: &str) -> &str {
    match token {
        "date" => "DateOnly",
        "time" => "TimeOnly",
        "duration" => "TimeSpan",
        "guid" => "Guid",
        "day_of_week" => "DayOfWeek",
        "i32" => "int",
        "i64" => "long",
        "bool" => "bool",
        _ => panic!(),
    }
}
fn requests() -> Value {
    json!(groups().into_iter().flat_map(|(id,cases)| {
        let token = cases[0].operation.split('.').next().unwrap();
        let locals = match token {
            "date"=>"DateOnly x=new DateOnly(2024,1,31);DateOnly y=new DateOnly(2024,2,29);",
            "time"=>"TimeOnly x=new TimeOnly(37230040000L);TimeOnly y=new TimeOnly(72000000000L);",
            "duration"=>"TimeSpan x=new TimeSpan(-900610010000L);TimeSpan y=new TimeSpan(10000L);",
            "guid"=>"Guid x=Guid.Empty;Guid y=Guid.Empty;", _=>panic!()
        };
        let mut source=format!("using System;namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{locals}");
        let mut emitted=BTreeSet::new();
        for (i,c) in cases.iter().enumerate() {
            if emitted.insert(&c.operation) { source.push_str(&format!("{} v{i}={};",native_type(c.result),c.native)); }
        }
        source.push_str("return lower;}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id,cases.iter().map(Case::clause).collect())],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
#[test]
fn csharp_03_t06_w09_calendar_clause_requests() {
    let value = requests();
    if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/calendar-clauses/requests.json"),
            value
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_CLAUSES_OUT") {
        fs::create_dir_all(&path).unwrap();
        fs::write(std::path::Path::new(&path).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/calendar-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_calendar_clauses_candidates() {
    verify_source(false);
}
#[test]
fn csharp_03_t06_w09_calendar_clauses_original_source() {
    verify_source(true);
}
fn verify_source(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let groups = groups();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_CALENDAR_CLAUSE_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/calendar-clauses/responses.json")
    };
    let mut coverage = BTreeSet::new();
    let mut attachments = 0;
    let mut observations = 0;
    let mut failures = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let cases = &groups[id];
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
        let program = generate_csharp_practical_ordinary_contract_expressions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        metadata(&program, &vc);
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let (definitions, standalone) = if id.starts_with("date-") || id == "guid" {
            let p = generate_csharp_practical_ordinary_calendar(vir).unwrap();
            (p.definitions().to_vec(), p.certificate_bytes().to_vec())
        } else {
            let p = generate_csharp_practical_ordinary_temporal(vir).unwrap();
            (p.definitions().to_vec(), p.certificate_bytes().to_vec())
        };
        let standalone = mpk_cert::decode_canonical_certificate(&standalone).unwrap();
        let mut roots = BTreeSet::new();
        for d in vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| matches!(d.tag.as_str(), "unary" | "binary"))
        {
            let params: Value = serde_json::from_str(&d.parameters).unwrap();
            let op = params["operation_id"].as_str().unwrap();
            coverage.insert(op.to_owned());
            let original = definitions
                .iter()
                .find(|s| s.operation.id == op)
                .unwrap_or_else(|| panic!("missing {op}"));
            assert_eq!(original.operation.argument_type_ids, d.argument_types);
            assert_eq!(original.operation.normal_result_type_id, d.result_type);
            assert_eq!(original.operation.ordered_checks, d.ordered_checks);
            let decl = cert
                .declarations
                .iter()
                .find(|d2| cert.name_table[d2.name as usize] == alias(&d.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = decl.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = cert.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                cert.name_table[cert.declarations[global as usize].name as usize],
                original.result_definition
            );
            roots.insert(original.result_definition.clone());
            roots.extend(original.ordered_failure_definitions.iter().cloned());
        }
        structural_equivalence_tests::same_definition_closure(&standalone, &cert, &roots).unwrap();
        let first = roots.iter().next().unwrap();
        let mut broken = cert.clone();
        let decl = broken
            .declarations
            .iter_mut()
            .find(|d| broken.name_table[d.name as usize] == *first)
            .unwrap();
        let mpk_cert::encode::DeclarationKind::Def { value, .. } = &mut decl.kind else {
            panic!()
        };
        *value = broken.term_table.len() as u32;
        broken.term_table.push(mpk_cert::encode::TermNode::Var(999));
        assert!(structural_equivalence_tests::same_definition_closure(
            &standalone,
            &broken,
            &roots
        )
        .is_err());
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for d in program.definitions() {
            attachments += 1;
            let e = vir
                .contract_expressions()
                .iter()
                .find(|e| e.attachment_sha256() == d.attachment_sha256)
                .unwrap();
            let encoded: Value = serde_json::from_slice(&e.canonical_bytes()).unwrap();
            let expr: Value =
                serde_json::from_str(encoded["canonical_expression"].as_str().unwrap()).unwrap();
            let case = cases
                .iter()
                .find(|c| {
                    serde_json::from_slice::<Value>(
                        &a::canonical_practical_json_bytes(&c.clause()).unwrap(),
                    )
                    .unwrap()
                        == expr
                })
                .unwrap();
            if runtime && case.observe {
                let args = d
                    .subjects
                    .iter()
                    .map(|(_, ty)| sparse_cube(types[ty], BTreeSet::new()))
                    .collect::<Vec<_>>();
                assert_eq!(
                    bit(run(&cert, &d.definedness_definition, args.clone())),
                    case.defined,
                    "{id}: {} definedness",
                    case.operation
                );
                if case.defined {
                    assert!(
                        bit(run(&cert, &d.value_definition, args)),
                        "{id}: {} value",
                        case.operation
                    );
                } else {
                    failures += 1;
                }
                observations += 1;
            }
        }
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut changed: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        changed["definitions"][0]["result_type"] = json!("changed");
        assert!(import_csharp_practical_ordinary_contract_expressions(
            &serde_json::to_vec(&changed).unwrap(),
            program.certificate_bytes(),
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
            "calendar/temporal source context {id} aliases/closures passed; runtime={runtime}"
        );
    }
    assert_eq!(
        coverage,
        groups
            .values()
            .flatten()
            .map(|c| c.operation.clone())
            .collect()
    );
    assert_eq!(attachments, 54);
    let expected = if runtime {
        groups.values().flatten().filter(|c| c.observe).count()
    } else {
        0
    };
    let errors = if runtime {
        groups
            .values()
            .flatten()
            .filter(|c| c.observe && !c.defined)
            .count()
    } else {
        0
    };
    assert_eq!((observations, failures), (expected, errors));
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
    eprintln!("calendar/temporal source: 50 aliases, 54 attachments, {observations} selected observations, {failures} definedness rejections");
}
