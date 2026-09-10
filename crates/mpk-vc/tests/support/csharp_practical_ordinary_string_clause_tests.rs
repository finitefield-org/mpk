//! Native string contract adapters with context-selected nullable carriers.
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
fn literal(token: &str, value: J) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", value),
    ])
}
fn string(value: &[u16]) -> J {
    literal("string", J::Utf16String(value.to_vec()))
}
fn character(c: u16) -> J {
    literal("char", J::Utf16String(vec![c]))
}
fn number(n: i32) -> J {
    literal("i32", J::string(n.to_string()))
}
fn boolean(b: bool) -> J {
    literal("bool", J::Bool(b))
}
fn text(nullable: bool, value: Option<&[u16]>) -> J {
    if !nullable {
        return string(value.unwrap());
    }
    let id =
        csharp_practical_closed_instance_id(&b(), &instance("option", vec![primitive("string")]))
            .unwrap();
    J::object(vec![
        ("tag", J::string("tagged_make")),
        ("type_id", J::string(&id)),
        ("semantic_instance_id", J::string(id)),
        (
            "arm",
            J::string(if value.is_some() { "some" } else { "none" }),
        ),
        ("payload", value.map(string).unwrap_or(J::Null)),
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
    for nullable in [false, true] {
        let suffix = if nullable { "nullable" } else { "nonnull" };
        let mut add =
            |group: &str, op: &str, result, args, expected, native: &str, defined, observe| {
                groups
                    .entry(format!("{group}-{suffix}"))
                    .or_default()
                    .push(Case {
                        operation: format!("string.{op}"),
                        result,
                        arguments: args,
                        expected,
                        native: native.into(),
                        defined,
                        observe,
                    });
            };
        let x = [97u16, 0xd800];
        let y = [97u16, 0xd800, 0];
        add(
            "basic",
            "length",
            "i32",
            vec![text(nullable, Some(&x))],
            number(2),
            "x.Length",
            true,
            true,
        );
        add(
            "basic",
            "index",
            "char",
            vec![text(nullable, Some(&x)), number(1)],
            character(0xd800),
            "x[1]",
            true,
            true,
        );
        for index in [-1, 2] {
            add(
                "basic",
                "index",
                "char",
                vec![text(nullable, Some(&x)), number(index)],
                character(0),
                "x[1]",
                false,
                true,
            );
        }
        for value in [x.as_slice(), &[]] {
            add(
                "basic",
                "is_null_or_empty",
                "bool",
                vec![text(nullable, Some(value))],
                boolean(value.is_empty()),
                "string.IsNullOrEmpty(x)",
                true,
                true,
            );
        }
        if nullable {
            add(
                "basic",
                "length",
                "i32",
                vec![text(true, None)],
                number(0),
                "x.Length",
                false,
                true,
            );
            add(
                "basic",
                "index",
                "char",
                vec![text(true, None), number(0)],
                character(0),
                "x[1]",
                false,
                true,
            );
            add(
                "basic",
                "is_null_or_empty",
                "bool",
                vec![text(true, None)],
                boolean(true),
                "string.IsNullOrEmpty(x)",
                true,
                true,
            );
        }
        for (op, native, result, expected) in [
            ("equality.operator", "x==y", "bool", boolean(false)),
            ("inequality.operator", "x!=y", "bool", boolean(true)),
            (
                "equals.ordinal",
                "string.Equals(x,y,StringComparison.Ordinal)",
                "bool",
                boolean(false),
            ),
            (
                "equals.instance.ordinal",
                "x.Equals(y,StringComparison.Ordinal)",
                "bool",
                boolean(false),
            ),
            (
                "compare.ordinal",
                "string.Compare(x,y,StringComparison.Ordinal)",
                "i32",
                number(-1),
            ),
        ] {
            add(
                "ordinal",
                op,
                result,
                vec![text(nullable, Some(&x)), text(nullable, Some(&y))],
                expected,
                native,
                true,
                true,
            );
            if nullable && op == "equals.instance.ordinal" {
                add(
                    "ordinal",
                    op,
                    result,
                    vec![text(true, None), text(true, Some(&y))],
                    boolean(false),
                    native,
                    false,
                    true,
                );
            }
        }
        for (op, native, needle) in [
            (
                "contains.ordinal",
                "x.Contains(y,StringComparison.Ordinal)",
                vec![0xd800],
            ),
            (
                "starts_with.ordinal",
                "x.StartsWith(y,StringComparison.Ordinal)",
                vec![97],
            ),
            (
                "ends_with.ordinal",
                "x.EndsWith(y,StringComparison.Ordinal)",
                vec![0xd800],
            ),
        ] {
            add(
                "ordinal",
                op,
                "bool",
                vec![text(nullable, Some(&x)), text(nullable, Some(&needle))],
                boolean(true),
                native,
                true,
                true,
            );
            if nullable {
                for (left, right) in [
                    (None, Some(needle.as_slice())),
                    (Some(x.as_slice()), None),
                    (None, None),
                ] {
                    add(
                        "ordinal",
                        op,
                        "bool",
                        vec![text(true, left), text(true, right)],
                        boolean(false),
                        native,
                        false,
                        true,
                    );
                }
            }
        }
        if nullable {
            add(
                "ordinal",
                "equals.ordinal",
                "bool",
                vec![text(true, None), text(true, None)],
                boolean(true),
                "string.Equals(x,y,StringComparison.Ordinal)",
                true,
                true,
            );
            add(
                "ordinal",
                "compare.ordinal",
                "i32",
                vec![text(true, None), text(true, Some(&x))],
                number(-1),
                "string.Compare(x,y,StringComparison.Ordinal)",
                true,
                true,
            );
        }
        for (op, args, expected, native) in [
            (
                "concat.string2",
                vec![text(nullable, Some(&[97])), text(nullable, Some(&[98]))],
                vec![97, 98],
                "string.Concat(x,y)",
            ),
            (
                "concat.operator.string_string",
                vec![text(nullable, Some(&[97])), text(nullable, Some(&[98]))],
                vec![97, 98],
                "x+y",
            ),
            (
                "concat.operator.string_char",
                vec![text(nullable, Some(&[97])), character(0xd800)],
                vec![97, 0xd800],
                "x+c",
            ),
            (
                "concat.operator.char_string",
                vec![character(0xd800), text(nullable, Some(&[98]))],
                vec![0xd800, 98],
                "c+y",
            ),
        ] {
            add(
                "construction",
                op,
                "string",
                args,
                string(&expected),
                native,
                true,
                true,
            );
        }
        for (shape, native) in [
            ("s", "$\"{x}\""),
            ("c", "$\"{c}\""),
            ("ss", "$\"{x}{y}\""),
            ("sc", "$\"{x}{c}\""),
            ("cs", "$\"{c}{x}\""),
            ("cc", "$\"{c}{c}\""),
        ] {
            let mut args = vec![];
            let mut result = vec![];
            for kind in shape.chars() {
                if kind == 's' {
                    args.push(text(nullable, Some(&[97])));
                    result.push(97);
                } else {
                    args.push(character(0xd800));
                    result.push(0xd800);
                }
            }
            add(
                "construction",
                &format!("interpolation.restricted.{shape}"),
                "string",
                args,
                string(&result),
                native,
                true,
                true,
            );
        }
        let large = vec![97; 16384];
        add(
            "construction",
            "concat.string2",
            "string",
            vec![text(nullable, Some(&large)), text(nullable, Some(&[98]))],
            string(&[]),
            "string.Concat(x,y)",
            false,
            true,
        );
        if nullable {
            add(
                "construction",
                "concat.string2",
                "string",
                vec![text(true, None), text(true, Some(&[98]))],
                string(&[98]),
                "string.Concat(x,y)",
                true,
                true,
            );
            add(
                "construction",
                "concat.operator.string_char",
                "string",
                vec![text(true, None), character(0xd800)],
                string(&[0xd800]),
                "x+c",
                true,
                true,
            );
        }
    }
    assert_eq!(groups.len(), 6);
    assert_eq!(
        groups
            .values()
            .flatten()
            .map(|c| &c.operation)
            .collect::<BTreeSet<_>>()
            .len(),
        21
    );
    groups
}
fn native_type(token: &str) -> &str {
    match token {
        "string" => "string",
        "char" => "char",
        "bool" => "bool",
        "i32" => "int",
        _ => panic!(),
    }
}
fn requests() -> Value {
    json!(groups().into_iter().flat_map(|(id,cases)|{
        let nullable=id.ends_with("-nullable");
        // Declared nullable types select the carrier; known non-null initializers
        // keep this capture source warning-free. Null behavior is exercised by
        // the independently supplied contract expressions below.
        let string_type=if nullable {"string?"} else {"string"};
        let mut locals=format!("{string_type} x=\"a\\ud800\";");
        if !id.starts_with("basic-") {locals.push_str(&format!("{string_type} y=\"a\\ud800\\0\";"));}
        if id.starts_with("construction-") {locals.push_str("char c='\\ud800';");}
        let mut source=format!("#nullable enable\nusing System;namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{locals}");
        let mut seen=BTreeSet::new();for (i,c) in cases.iter().enumerate(){if seen.insert(&c.operation){source.push_str(&format!("{} v{i}={};",native_type(c.result),c.native));}}
        source.push_str("return lower;}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id,cases.iter().map(Case::clause).collect())],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
#[test]
fn csharp_03_t06_w09_string_clause_requests() {
    let value = requests();
    if let Some(path) = std::env::var_os("MPK_W09_STRING_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/string-clauses/requests.json"),
            value
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_STRING_CLAUSES_OUT") {
        fs::create_dir_all(&path).unwrap();
        fs::write(std::path::Path::new(&path).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/string-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_string_clauses_candidates() {
    verify_source(false);
}
#[test]
fn csharp_03_t06_w09_string_clauses_original_source() {
    verify_source(true);
}
fn verify_source(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let groups = groups();
    let runtime_prefix = std::env::var("MPK_W09_STRING_RUNTIME_PREFIX").ok();
    let selected = |id: &str| runtime_prefix.as_ref().is_none_or(|p| id.starts_with(p));
    assert!(
        groups.keys().any(|id| selected(id)),
        "no matching runtime context"
    );
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_STRING_CLAUSE_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/string-clauses/responses.json")
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
        let p = generate_csharp_practical_ordinary_strings(vir).unwrap();
        let (definitions, standalone) = (p.definitions().to_vec(), p.certificate_bytes().to_vec());
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
            let expression = encoded["canonical_expression"].as_str().unwrap().as_bytes();
            let case = cases
                .iter()
                .find(|c| a::canonical_practical_json_bytes(&c.clause()).unwrap() == expression)
                .unwrap();
            if runtime && selected(id) && case.observe {
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
        eprintln!("string source context {id} aliases/closures passed; runtime={runtime}");
    }
    assert_eq!(
        coverage,
        groups
            .values()
            .flatten()
            .map(|c| c.operation.clone())
            .collect()
    );
    assert_eq!(attachments, groups.values().map(Vec::len).sum::<usize>());
    let expected = if runtime {
        groups
            .iter()
            .filter(|(id, _)| selected(id))
            .flat_map(|(_, cases)| cases)
            .filter(|c| c.observe)
            .count()
    } else {
        0
    };
    let errors = if runtime {
        groups
            .iter()
            .filter(|(id, _)| selected(id))
            .flat_map(|(_, cases)| cases)
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
    eprintln!("string source: 21 operation kinds, {attachments} attachments, {observations} selected observations, {failures} definedness rejections");
}
