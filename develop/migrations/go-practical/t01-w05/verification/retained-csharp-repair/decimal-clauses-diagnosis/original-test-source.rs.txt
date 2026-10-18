//! Decimal source contract adapters, including range and arithmetic failures.
use super::fixed_codec_clause_tests::alias;
use super::*;
const MODES: [&str; 5] = [
    "ToEven",
    "AwayFromZero",
    "ToZero",
    "ToNegativeInfinity",
    "ToPositiveInfinity",
];
const INTS: [(&str, &str, &str); 9] = [
    ("sbyte", "i8", "sbyte"),
    ("byte", "u8", "byte"),
    ("int16", "i16", "short"),
    ("uint16", "u16", "ushort"),
    ("int32", "i32", "int"),
    ("uint32", "u32", "uint"),
    ("int64", "i64", "long"),
    ("uint64", "u64", "ulong"),
    ("char", "char", "char"),
];
const MAX: &str = "79228162514264337593543950335";
fn literal(token: &str, text: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", J::string(text)),
    ])
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
fn operations() -> Vec<String> {
    let mut ops = [
        "plus",
        "negate",
        "truncate",
        "floor",
        "ceiling",
        "equal",
        "not_equal",
        "less",
        "less_equal",
        "greater",
        "greater_equal",
        "add",
        "subtract",
        "multiply",
        "divide",
        "remainder",
    ]
    .into_iter()
    .map(|s| format!("decimal.{s}"))
    .collect::<Vec<_>>();
    for mode in MODES {
        for arity in [1, 2] {
            ops.push(format!("decimal.round.{mode}.{arity}"));
        }
    }
    for (name, _, _) in INTS {
        ops.push(format!("decimal.conversion.{name}_to_decimal"));
        ops.push(format!("decimal.conversion.decimal_to_{name}"));
    }
    ops
}
fn case(id: &str, invalid: bool) -> J {
    let op = id.strip_prefix("decimal.").unwrap();
    let (result, args, expected) = if let Some(suffix) = op.strip_prefix("conversion.") {
        let (from, to) = suffix.split_once("_to_").unwrap();
        let token = |name| {
            if name == "decimal" {
                "decimal"
            } else {
                INTS.iter().find(|(n, _, _)| *n == name).unwrap().1
            }
        };
        let from = token(from);
        let to = token(to);
        let input = if from == "decimal" {
            literal(from, if invalid { MAX } else { "2.5" })
        } else if from == "char" {
            J::object(vec![
                ("tag", J::string("literal")),
                ("type_id", J::string(ty(from))),
                ("value", J::Utf16String(vec![2])),
            ])
        } else {
            literal(from, "2")
        };
        let expected = if to == "char" {
            J::object(vec![
                ("tag", J::string("literal")),
                ("type_id", J::string(ty(to))),
                ("value", J::Utf16String(vec![2])),
            ])
        } else {
            literal(to, "2")
        };
        (to, vec![input], expected)
    } else if let Some(suffix) = op.strip_prefix("round.") {
        let (mode, arity) = suffix.split_once('.').unwrap();
        let mut args = vec![literal("decimal", "2.5")];
        if arity == "2" {
            args.push(literal("i32", if invalid { "29" } else { "0" }));
        }
        (
            "decimal",
            args,
            literal(
                "decimal",
                if matches!(mode, "AwayFromZero" | "ToPositiveInfinity") {
                    "3"
                } else {
                    "2"
                },
            ),
        )
    } else if matches!(op, "plus" | "negate" | "truncate" | "floor" | "ceiling") {
        (
            "decimal",
            vec![literal("decimal", "2.5")],
            literal(
                "decimal",
                match op {
                    "plus" => "2.5",
                    "negate" => "-2.5",
                    "ceiling" => "3",
                    _ => "2",
                },
            ),
        )
    } else {
        let boolean = matches!(
            op,
            "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal"
        );
        let args = if invalid {
            vec![
                literal("decimal", if op == "divide" { "1" } else { MAX }),
                literal(
                    "decimal",
                    if op == "divide" {
                        "0"
                    } else if op == "multiply" {
                        "2"
                    } else {
                        "1"
                    },
                ),
            ]
        } else {
            vec![literal("decimal", "2.5"), literal("decimal", "1")]
        };
        let expected = if boolean {
            J::object(vec![
                ("tag", J::string("literal")),
                ("type_id", J::string(ty("bool"))),
                (
                    "value",
                    J::Bool(matches!(op, "not_equal" | "greater" | "greater_equal")),
                ),
            ])
        } else {
            literal(
                "decimal",
                match op {
                    "add" => "3.5",
                    "subtract" => "1.5",
                    "multiply" | "divide" => "2.5",
                    "remainder" => "0.5",
                    _ => unreachable!(),
                },
            )
        };
        (if boolean { "bool" } else { "decimal" }, args, expected)
    };
    equal(call(id, result, args), expected)
}
fn group(id: &str) -> &str {
    let op = id.strip_prefix("decimal.").unwrap();
    if op.starts_with("conversion.") {
        "conversions"
    } else if op.starts_with("round.") {
        "round"
    } else if matches!(op, "plus" | "negate" | "truncate" | "floor" | "ceiling") {
        "unary"
    } else if matches!(op, "add" | "subtract" | "multiply" | "divide" | "remainder") {
        op
    } else {
        "comparisons"
    }
}
fn native_operation(id: &str) -> (String, String) {
    let op = id.strip_prefix("decimal.").unwrap();
    if let Some(suffix) = op.strip_prefix("conversion.") {
        let (from, to) = suffix.split_once("_to_").unwrap();
        let cs = |n| {
            if n == "decimal" {
                "decimal"
            } else {
                INTS.iter().find(|(name, _, _)| *name == n).unwrap().2
            }
        };
        let input = if from == "decimal" {
            "x".into()
        } else {
            format!("({})lower", cs(from))
        };
        return (cs(to).into(), format!("({})({input})", cs(to)));
    }
    if let Some(suffix) = op.strip_prefix("round.") {
        let (mode, arity) = suffix.split_once('.').unwrap();
        return (
            "decimal".into(),
            format!(
                "decimal.Round(x,{}System.MidpointRounding.{mode})",
                if arity == "2" { "0," } else { "" }
            ),
        );
    }
    let expression = match op {
        "plus" => "+x".into(),
        "negate" => "-x".into(),
        "truncate" => "decimal.Truncate(x)".into(),
        "floor" => "decimal.Floor(x)".into(),
        "ceiling" => "decimal.Ceiling(x)".into(),
        _ => format!(
            "x{}y",
            match op {
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
            }
        ),
    };
    (
        if group(id) == "comparisons" {
            "bool"
        } else {
            "decimal"
        }
        .into(),
        expression,
    )
}
fn requests() -> Value {
    let mut rows = BTreeMap::<String, (Vec<J>, String)>::new();
    for id in operations() {
        let row = rows.entry(group(&id).into()).or_insert_with(|| (vec![], String::from("namespace Quantifiers;public static class Entry{public static int Run(int lower,int upper){decimal x=(decimal)lower;decimal y=(decimal)upper;")));
        let (ty, expr) = native_operation(&id);
        row.1.push_str(&format!("{ty} v{}={expr};", row.0.len()));
        row.0.push(case(&id, false));
        if matches!(
            id.as_str(),
            "decimal.round.ToEven.2"
                | "decimal.conversion.decimal_to_int64"
                | "decimal.add"
                | "decimal.multiply"
                | "decimal.divide"
        ) {
            row.0.push(case(&id, true));
        }
    }
    json!(rows
        .into_iter()
        .flat_map(|(id, (cases, mut source))| {
            source.push_str("return lower;}}\n");
            integer_codec_clause_tests::requests_for_source(vec![(id, cases)], Some(&source))
                .as_array()
                .unwrap()
                .clone()
        })
        .collect::<Vec<_>>())
}
#[test]
fn csharp_03_t06_w09_decimal_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/decimal-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_DECIMAL_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_decimal_clauses_original_source() {
    verify_source(true);
}
#[test]
fn csharp_03_t06_w09_decimal_clauses_candidates() {
    verify_source(false);
}
fn verify_source(runtime: bool) {
    let runtime_contexts = std::env::var("MPK_W09_DECIMAL_CLAUSES_RUNTIME_CONTEXTS")
        .ok()
        .map(|s| s.split(',').map(str::to_owned).collect::<BTreeSet<_>>());
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_CLAUSE_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/decimal-clauses/responses.json")
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
        let standalone = generate_csharp_practical_ordinary_decimal(vir).unwrap();
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
        structural_equivalence_tests::same_decimal_definition_closure(
            &mpk_cert::decode_canonical_certificate(standalone.certificate_bytes()).unwrap(),
            &c,
            &roots,
        )
        .unwrap();
        // A shared helper may have a different name, but changed contents
        // must still reject, even below the stable public operation root.
        let mut broken = c.clone();
        let helper = broken.declarations.iter().position(|d| {
            let name = &broken.name_table[d.name as usize];
            name.starts_with("Mpk.CSharp.Ordinary.DecimalArithmetic.") && name.ends_with(".Result")
        });
        if let Some(index) = helper.filter(|_| id == "add") {
            let mpk_cert::encode::DeclarationKind::Def { value, .. } =
                &mut broken.declarations[index].kind
            else {
                panic!()
            };
            let old = *value;
            *value = broken.term_table.len() as u32;
            broken.term_table.push(mpk_cert::encode::TermNode::Var(999));
            assert!(
                structural_equivalence_tests::same_decimal_definition_closure(
                    &mpk_cert::decode_canonical_certificate(standalone.certificate_bytes())
                        .unwrap(),
                    &broken,
                    &roots,
                )
                .is_err(),
                "{id}: mutated helper {old} accepted"
            );
        }
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            attachments += 1;
            if !runtime || runtime_contexts.as_ref().is_some_and(|s| !s.contains(id)) {
                continue;
            }
            let e = vir
                .contract_expressions()
                .iter()
                .find(|e| e.attachment_sha256() == d.attachment_sha256)
                .unwrap();
            let encoded: Value = serde_json::from_slice(&e.canonical_bytes()).unwrap();
            let expr: Value =
                serde_json::from_str(encoded["canonical_expression"].as_str().unwrap()).unwrap();
            let op = &expr["left"];
            let operation = op["operation_id"].as_str().unwrap();
            let invalid = (operation == "decimal.round.ToEven.2" && op["right"]["value"] == "29")
                || (operation == "decimal.conversion.decimal_to_int64"
                    && op["operand"]["value"] == MAX)
                || (operation == "decimal.divide" && op["right"]["value"] == "0")
                || (matches!(operation, "decimal.add" | "decimal.multiply")
                    && op["left"]["value"] == MAX);
            if !invalid
                && !matches!(
                    operation,
                    "decimal.negate"
                        | "decimal.round.ToEven.2"
                        | "decimal.conversion.int64_to_decimal"
                        | "decimal.conversion.decimal_to_int64"
                        | "decimal.divide"
                )
            {
                continue;
            }
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
        eprintln!("decimal source context {id} passed");
    }
    assert_eq!(coverage, operations().into_iter().collect());
    assert_eq!(attachments, 49);
    let mut expected = (0, 0);
    if runtime {
        let counts = [
            ("add", 1, 1),
            ("comparisons", 0, 0),
            ("conversions", 3, 1),
            ("divide", 2, 1),
            ("multiply", 1, 1),
            ("remainder", 0, 0),
            ("round", 2, 1),
            ("subtract", 0, 0),
            ("unary", 1, 0),
        ];
        if let Some(ids) = &runtime_contexts {
            assert!(!ids.is_empty() && ids.iter().all(|id| counts.iter().any(|(s, _, _)| s == id)));
        }
        for (id, count, failures) in counts {
            if runtime_contexts.as_ref().is_none_or(|s| s.contains(id)) {
                expected.0 += count;
                expected.1 += failures;
            }
        }
    }
    assert_eq!((executed, rejected), expected);
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
    eprintln!("decimal clauses: 44 operation aliases/closures, 49 attachments, {executed} selected conditions, {rejected} failure definedness rejections");
}
