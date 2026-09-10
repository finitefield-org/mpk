//! Floating source expression aliases and checked-conversion definedness.
use super::fixed_codec_clause_tests::alias;
use super::*;
const OPS: [&str; 19] = [
    "plus",
    "negate",
    "abs",
    "is_nan",
    "is_infinity",
    "is_finite",
    "equal",
    "not_equal",
    "less",
    "less_equal",
    "greater",
    "greater_equal",
    "min",
    "max",
    "add",
    "subtract",
    "multiply",
    "divide",
    "remainder",
];
const CONVERSIONS: [(&str, &str, &str); 6] = [
    ("int32_to_single", "i32", "f32"),
    ("int64_to_double", "i64", "f64"),
    ("single_to_double", "f32", "f64"),
    ("double_to_single", "f64", "f32"),
    ("single_to_int32.checked", "f32", "i32"),
    ("double_to_int64.checked", "f64", "i64"),
];
fn literal(token: &str, value: J) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", value),
    ])
}
fn float(token: &str, value: f64) -> J {
    literal(
        token,
        J::string(if token == "f32" {
            format!("{:08x}", (value as f32).to_bits())
        } else {
            format!("{:016x}", value.to_bits())
        }),
    )
}
fn number(token: &str, value: &str) -> J {
    literal(token, J::string(value))
}
fn call(id: &str, result: &str, args: Vec<J>) -> J {
    let mut fields = vec![
        (
            "tag",
            J::string(if args.len() == 1 { "unary" } else { "binary" }),
        ),
        ("type_id", J::string(ty(result))),
        ("operation_id", J::string(id)),
    ];
    if args.len() == 1 {
        fields.push(("operand", args[0].clone()));
    } else {
        assert_eq!(args.len(), 2);
        fields.extend([("left", args[0].clone()), ("right", args[1].clone())]);
    }
    J::object(fields)
}
fn equal(left: J, right: J) -> J {
    J::object(vec![
        ("tag", J::string("structural_equal")),
        ("type_id", J::string(ty("bool"))),
        ("left", left),
        ("right", right),
    ])
}
fn floating_case(kind: &str, op: &str) -> J {
    let token = if kind == "single" { "f32" } else { "f64" };
    let unary = matches!(
        op,
        "plus" | "negate" | "abs" | "is_nan" | "is_infinity" | "is_finite"
    );
    let boolean = matches!(
        op,
        "is_nan"
            | "is_infinity"
            | "is_finite"
            | "equal"
            | "not_equal"
            | "less"
            | "less_equal"
            | "greater"
            | "greater_equal"
    );
    let (a, b) = match op {
        "plus" | "min" | "max" | "equal" | "less_equal" => (0.0, -0.0),
        "negate" => (0.0, 0.0),
        "abs" => (-3.0, 0.0),
        "is_nan" | "not_equal" => (f64::NAN, f64::NAN),
        "is_infinity" | "is_finite" | "greater" => (f64::INFINITY, 3.0),
        "less" => (f64::NAN, 3.0),
        "greater_equal" => (f64::NEG_INFINITY, 3.0),
        "add" => (f64::INFINITY, f64::NEG_INFINITY),
        "divide" => (1.0, 0.0),
        "remainder" => (3.0, 2.0),
        _ => (3.0, -2.0),
    };
    let args = if unary {
        vec![float(token, a)]
    } else {
        vec![float(token, a), float(token, b)]
    };
    let value = call(
        &format!("floating.{kind}.{op}"),
        if boolean { "bool" } else { token },
        args,
    );
    if op == "add" {
        return call(&format!("floating.{kind}.is_nan"), "bool", vec![value]);
    }
    let expected = if boolean {
        literal(
            "bool",
            J::Bool(matches!(
                op,
                "is_nan" | "is_infinity" | "equal" | "not_equal" | "less_equal" | "greater"
            )),
        )
    } else {
        float(
            token,
            match op {
                "plus" | "min" | "max" => 0.0,
                "negate" => -0.0,
                "abs" => 3.0,
                "subtract" => 5.0,
                "multiply" => -6.0,
                "divide" => f64::INFINITY,
                "remainder" => 1.0,
                _ => unreachable!(),
            },
        )
    };
    equal(value, expected)
}
fn conversion_case(suffix: &str, from: &str, to: &str, invalid: bool) -> J {
    let input = if invalid {
        float(from, f64::INFINITY)
    } else {
        match suffix {
            "int32_to_single" => number(from, "16777217"),
            "int64_to_double" => number(from, "9007199254740993"),
            "single_to_int32.checked" => float(from, 3.75),
            "double_to_int64.checked" => float(from, -3.75),
            _ => float(from, 1.25),
        }
    };
    let expected = match suffix {
        "int32_to_single" => float(to, 16777216.0),
        "int64_to_double" => float(to, 9007199254740992.0),
        "single_to_int32.checked" => number(to, "3"),
        "double_to_int64.checked" => number(to, "-3"),
        _ => float(to, 1.25),
    };
    equal(
        call(&format!("numeric.conversion.{suffix}"), to, vec![input]),
        expected,
    )
}
fn selected_ops(id: &str) -> Vec<&'static str> {
    OPS.into_iter()
        .filter(|op| match id {
            "single" => true,
            "double-basic" => !matches!(
                *op,
                "add" | "subtract" | "multiply" | "divide" | "remainder"
            ),
            "double-arithmetic" => matches!(*op, "add" | "subtract" | "multiply" | "remainder"),
            "double-divide" => *op == "divide",
            "conversions" => false,
            _ => panic!("unknown context"),
        })
        .collect()
}
fn requests() -> Value {
    let mut rows: Vec<(String, Vec<J>)> = [
        "single",
        "double-basic",
        "double-arithmetic",
        "double-divide",
    ]
    .into_iter()
    .map(|id| {
        (
            id.into(),
            selected_ops(id)
                .into_iter()
                .map(|op| floating_case(if id == "single" { "single" } else { "double" }, op))
                .collect(),
        )
    })
    .collect();
    let mut conversions = CONVERSIONS
        .into_iter()
        .map(|(s, f, t)| conversion_case(s, f, t, false))
        .collect::<Vec<_>>();
    for (s, f, t) in CONVERSIONS
        .into_iter()
        .filter(|(s, _, _)| s.ends_with(".checked"))
    {
        conversions.push(conversion_case(s, f, t, true));
    }
    rows.push(("conversions".into(), conversions));
    let requests = rows
        .into_iter()
        .flat_map(|(id, expressions)| {
            let source = native_source(&id);
            integer_codec_clause_tests::requests_for_source(vec![(id, expressions)], Some(&source))
                .as_array()
                .unwrap()
                .clone()
        })
        .collect::<Vec<_>>();
    json!(requests)
}
// The operation table is native-source-backed; each context contains only its
// relevant family so unrelated native operations do not consume its limits.
fn native_source(id: &str) -> String {
    let mut source = String::from("namespace Quantifiers;public static class Entry{public static int Run(int lower,int upper){");
    if id == "single" {
        source.push_str("float sx=(float)lower;float sy=(float)upper;");
    } else if id.starts_with("double-") {
        source.push_str("double dx=(double)(long)lower;double dy=(double)(long)upper;");
    } else {
        assert_eq!(id, "conversions");
        source.push_str("float sx=(float)lower;double dx=(double)(long)lower;");
    }

    for (prefix, token, math) in [
        ("s", "float", "System.MathF"),
        ("d", "double", "System.Math"),
    ] {
        if (id != "single" || prefix != "s") && (!id.starts_with("double-") || prefix != "d") {
            continue;
        }
        let x = format!("{prefix}x");
        let y = format!("{prefix}y");
        for (i, op) in OPS.into_iter().enumerate() {
            if !selected_ops(id).contains(&op)
                && !(op == "is_nan" && selected_ops(id).contains(&"add"))
            {
                continue;
            }
            let (result, expression) = match op {
                "plus" => (token, format!("+{x}")),
                "negate" => (token, format!("-{x}")),
                "abs" => (token, format!("{math}.Abs({x})")),
                "is_nan" => ("bool", format!("{token}.IsNaN({x})")),
                "is_infinity" => ("bool", format!("{token}.IsInfinity({x})")),
                "is_finite" => ("bool", format!("{token}.IsFinite({x})")),
                "min" => (token, format!("{math}.Min({x},{y})")),
                "max" => (token, format!("{math}.Max({x},{y})")),
                _ => {
                    let symbol = match op {
                        "equal" => "==",
                        "not_equal" => "!=",
                        "less" => "<",
                        "less_equal" => "<=",
                        "greater" => ">",
                        "greater_equal" => ">=",
                        "add" => "+",
                        "subtract" => "-",
                        "multiply" => "*",
                        "divide" => "/",
                        "remainder" => "%",
                        _ => unreachable!(),
                    };
                    (
                        if i < 12 { "bool" } else { token },
                        format!("{x}{symbol}{y}"),
                    )
                }
            };
            source.push_str(&format!("{result} {prefix}v{i}={expression};"));
        }
    }
    if id == "conversions" {
        source.push_str("double wider=(double)sx;float narrower=(float)dx;int ci=checked((int)sx);long cl=checked((long)dx);");
    }
    source.push_str("return lower;}}\n");
    source
}

#[test]
fn csharp_03_t06_w09_floating_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_FLOATING_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/floating-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_FLOATING_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/floating-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_floating_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_FLOATING_CLAUSE_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/floating-clauses/responses.json")
    };
    let mut coverage = BTreeSet::new();
    let mut executed = 0;
    let mut rejected = 0;
    let mut attachments = 0;
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
        let p = generate_csharp_practical_ordinary_contract_expressions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        metadata(&p, &vc);
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let standalone = generate_csharp_practical_ordinary_floating(vir).unwrap();
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
            let original = standalone
                .definitions()
                .iter()
                .find(|s| s.operation.id == op)
                .unwrap_or_else(|| panic!("missing {op}"));
            assert_eq!(original.operation.argument_type_ids, d.argument_types);
            assert_eq!(original.operation.normal_result_type_id, d.result_type);
            assert_eq!(original.operation.ordered_checks, d.ordered_checks);
            let decl = c
                .declarations
                .iter()
                .find(|v| c.name_table[v.name as usize] == alias(&d.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = decl.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                original.result_definition
            );
            roots.insert(original.result_definition.clone());
            roots.extend(original.ordered_failure_definitions.iter().cloned());
        }
        structural_equivalence_tests::same_definition_closure(
            &mpk_cert::decode_canonical_certificate(standalone.certificate_bytes()).unwrap(),
            &c,
            &roots,
        )
        .unwrap();
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            attachments += 1;
            let e = vir
                .contract_expressions()
                .iter()
                .find(|e| e.attachment_sha256() == d.attachment_sha256)
                .unwrap();
            let encoded: Value = serde_json::from_slice(&e.canonical_bytes()).unwrap();
            let expr: Value =
                serde_json::from_str(encoded["canonical_expression"].as_str().unwrap()).unwrap();
            let op = if expr["tag"] == "structural_equal" {
                &expr["left"]
            } else {
                &expr["operand"]
            };
            let operation = op["operation_id"].as_str().unwrap();
            if id != "conversions"
                && ![".equal", ".not_equal", ".divide", ".add", ".negate"]
                    .into_iter()
                    .any(|suffix| operation.ends_with(suffix))
            {
                continue;
            }
            let invalid = id == "conversions"
                && op["operand"]["value"]
                    .as_str()
                    .is_some_and(|s| matches!(s, "7f800000" | "7ff0000000000000"));
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| sparse_cube(types[ty], BTreeSet::new()))
                .collect::<Vec<_>>();
            assert_eq!(
                bit(run(&c, &d.definedness_definition, args.clone())),
                !invalid,
                "{id}: {operation} definedness"
            );
            if invalid {
                rejected += 1;
            } else {
                assert!(
                    bit(run(&c, &d.value_definition, args)),
                    "{id}: {operation} value"
                );
            }
            executed += 1;
        }
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        changed["definitions"][0]["result_type"] = json!("changed");
        assert!(import_csharp_practical_ordinary_contract_expressions(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        eprintln!("floating source context {id} passed");
    }
    let mut expected = ["single", "double"]
        .into_iter()
        .flat_map(|kind| {
            OPS.into_iter()
                .map(move |op| format!("floating.{kind}.{op}"))
        })
        .collect::<BTreeSet<_>>();
    expected.extend(
        CONVERSIONS
            .into_iter()
            .map(|(s, _, _)| format!("numeric.conversion.{s}")),
    );
    assert_eq!(coverage, expected);
    assert_eq!((attachments, executed, rejected), (46, 18, 2));
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
    eprintln!("floating clauses: 44 operation aliases/closures, 46 attachments, 18 selected conditions, 2 overflow definedness rejections");
}

#[test]
fn csharp_03_t06_w09_floating_clause_rejects_oversized_source() {
    let bundle = b();
    let requests = read("ordinary-foundation/floating-clauses/limit-rejections/requests.json");
    let responses = read("ordinary-foundation/floating-clauses/limit-rejections/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    assert!(matches!(
        generate_csharp_practical_ordinary_contract_expressions(emitted.vir()),
        Err(OrdinaryCarrierError::Limit)
    ));
}

#[test]
fn csharp_03_t06_w09_floating_clauses_pinned_bytes() {
    let bundle = b();
    let requests = requests();
    assert_eq!(
        read("ordinary-foundation/floating-clauses/requests.json"),
        requests
    );
    let responses = read("ordinary-foundation/floating-clauses/responses.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/floating-clauses");
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_contract_expressions(emitted.vir()).unwrap();
        assert_eq!(
            fs::read(fixture.join(format!("{id}.json"))).unwrap(),
            p.canonical_bytes()
        );
        assert_eq!(
            fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(requests.as_array().unwrap().len(), 5);
}
