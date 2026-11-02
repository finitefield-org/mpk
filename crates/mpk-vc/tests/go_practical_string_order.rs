use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_equality_foundation,
    generate_go_practical_string_order_foundation, import_go_practical_string_order_foundation,
    GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_ORDER_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(format!("{name}.{extension}")))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    retain(name, "mpcert", bytes);
    inspect_saved(name, bytes, accepted);
}
fn inspect_saved(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    retain(name, "rust.json", format!("{}\n", result.json).as_bytes());
    if accepted && !result.accepted {
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lo, mut hi) = (1, c.declarations.len());
        while lo < hi {
            let end = (lo + hi) / 2;
            let mut prefix = c.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lo = end + 1;
            } else {
                hi = end;
            }
        }
        panic!(
            "{name}: first declaration type-check failure {}: {}",
            c.name_table[c.declarations[lo - 1].name as usize],
            result.json
        );
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if accepted {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(v["axiom_count"], 0);
        assert!(v["axiom_report"]["entries"].as_array().unwrap().is_empty());
    } else {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        let d: serde_json::Value =
            serde_json::from_str(v["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}

use mpk_cert::encode::{Declaration, TermNode};
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let at = c.term_table.len() as u32;
    c.term_table.push(node);
    at
}
fn sym(c: &mut Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = sym(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn flag(c: &mut Certificate, yes: bool) -> u32 {
    sym(
        c,
        if yes {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    )
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let mpk_cert::encode::LevelNode::Param(name) = l {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn bad_result(c: &mut Certificate, ty: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = bad_result(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            if let TermNode::Const { global, .. } = c.term_table[function as usize] {
                if c.name_table[c.declarations[global as usize].name as usize] == "Std.Eq"
                    && arguments.len() == 3
                {
                    if let TermNode::Const { global, .. } = c.term_table[arguments[0] as usize] {
                        if c.name_table[c.declarations[global as usize].name as usize] == "Std.Bool"
                        {
                            arguments[2] = call(c, "Mpk.Go.Practical.Bool.Not", vec![arguments[2]]);
                            return term(
                                c,
                                TermNode::App {
                                    function,
                                    arguments,
                                },
                            );
                        }
                    }
                }
            }
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
        _ => {
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
    }
}
fn erase(c: &mut Certificate, ty: u32, at: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty: old, body } = c.term_table[ty as usize].clone() else {
        panic!("missing actual witness")
    };
    if at == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = erase(c, body, at - 1, domain);
        term(c, TermNode::Pi { ty: old, body })
    }
}

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringOrderFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStringOrderFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_order_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn generic_order_preserves_all1215_original_identities() {
    let f = foundation();
    inspect("order-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_order_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_equality_foundation().unwrap();
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-equality/epochs/semantic-controls/selected/candidates/equality-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 1215);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    let old = mpk_vc::go_practical_ordinary::generate_go_practical_word_order_foundation().unwrap();
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/word-order/corrected/rust-tests/certificates/word-order-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    for (name, value) in dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap(),
    ) {
        if let Some(current) = all.get(&name) {
            assert_eq!(current, &value, "{name}");
        }
    }
    let (defs, laws) = c
        .declarations
        .iter()
        .filter(|d| c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Order."))
        .fold((0, 0), |(defs, laws), d| match d.kind {
            DeclarationKind::Def { .. } => (defs + 1, laws),
            DeclarationKind::Theorem { .. } => (defs, laws + 1),
            _ => panic!("ordinary declarations only"),
        });
    println!("string order:declarations={} terms={} bytes={} all1215 prior identities exact;{} definitions{} ordinary laws",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len(),defs,laws);
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Order.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("order-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_order_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_order_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[path = "support/go_string_equality_eval.rs"]
mod order_eval;

#[test]
fn order_laws_bind_actual_conclusions_and_typed_witnesses() {
    let base = baseline();
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with("Mpk.Go.Practical.String.Order.")
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 190);
    for name in names {
        let mut c = base.clone();
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind else {
            unreachable!()
        };
        let ty = bad_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(
            &format!(
                "false-{}",
                name.strip_prefix("Mpk.Go.Practical.String.Order.").unwrap()
            ),
            &encode(c),
            false,
        );
    }
    let mut conditions = vec![
        ("OrLeftTrue".to_string(), 2),
        ("OrRightTrue".into(), 2),
        ("OrFalseLeft".into(), 2),
        ("OrFalseLeft".into(), 3),
        ("NatAsym".into(), 2),
    ];
    for d in 0..=8 {
        for (name, index) in [
            ("EqualReflect", 2),
            ("EqualComplete", 2),
            ("EqualRespects", 4),
            ("EqualRespects", 5),
            ("LessRespects", 4),
            ("LessRespects", 5),
            ("LessReflect", 2),
            ("LessReflect", 4),
            ("LessComplete", 2 + d),
            ("LessComplete", 3 + d),
        ] {
            conditions.push((format!("D{d}.{name}"), index));
        }
    }
    for (name, index) in [
        ("RankInside", 9),
        ("RankOutside", 9),
        ("RankLessInside", 10),
        ("RankLessInside", 11),
        ("RankEndBefore", 10),
        ("RankEndBefore", 11),
        ("RankEndAfter", 10),
        ("RankEndAfter", 11),
        ("RankEndEqual", 10),
        ("RankEndEqual", 11),
        ("RankRespects", 2),
        ("LessReflect", 2),
        ("LessReflect", 4),
        ("GreaterReflect", 2),
        ("GreaterReflect", 4),
        ("LessComplete", 10),
        ("LessComplete", 11),
        ("GreaterComplete", 10),
        ("GreaterComplete", 11),
    ] {
        conditions.push((name.into(), index));
    }
    for name in ["Less", "Greater", "LessEqual", "GreaterEqual"] {
        conditions.push((format!("{name}Respects"), 4));
        conditions.push((format!("{name}Respects"), 5));
    }
    assert_eq!(conditions.len(), 122);
    for (name, position) in conditions {
        let mut c = base.clone();
        let full = format!("Mpk.Go.Practical.String.Order.{name}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == full)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind else {
            unreachable!()
        };
        let boolean = sym(&mut c, "Std.Bool");
        let ty = erase(&mut c, ty, position, boolean);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("premise-{name}-{position}"), &encode(c), false);
    }
    println!("order controls:190 changed actual conclusions;122 replaced typed witness domains;vacuous empty-prefix witnesses remain part of the typed contract");
}

fn replace_leaf(
    c: &mut Certificate,
    at: u32,
    depth: usize,
    change: &mut impl FnMut(&mut Certificate, u32) -> u32,
) -> u32 {
    if depth == 0 {
        return change(c, at);
    }
    let TermNode::Lam { ty, body } = c.term_table[at as usize].clone() else {
        panic!("complete ordinary input/address lambdas")
    };
    let body = replace_leaf(c, body, depth - 1, change);
    term(c, TermNode::Lam { ty, body })
}

#[test]
fn byte_order_observations_cover_all65536_pairs() {
    let c = baseline();
    let mut total = 0;
    let mut steps = 0;
    for left in 0u8..=255 {
        let label = format!("byte-{left:03}");
        let input = serde_json::json!({"left_byte":left,"all_right_bytes":(0u16..=255).collect::<Vec<_>>(),"source_or_nominal_authority":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut model = order_eval::Interpreter::new(&c);
        let a = model.byte_input(left);
        let mut actual = vec![];
        let mut expected = vec![];
        for right in 0u8..=255 {
            let b = model.byte_input(right);
            let mut row = vec![];
            for (op, operands) in [
                ("ByteLess", [a, b]),
                ("ByteLessEqual", [a, b]),
                ("ByteLess", [b, a]),
                ("ByteLessEqual", [b, a]),
            ] {
                let value = model.invoke(&format!("Mpk.Go.Practical.String.Order.{op}"), &operands);
                row.push(model.boolean(value));
            }
            actual.push(row);
            expected.push(vec![
                left < right,
                left <= right,
                left > right,
                left >= right,
            ]);
            total += 1;
        }
        steps += model.steps();
        let result = serde_json::json!({"actual":actual,"expected":expected,"actual_AST_steps":model.steps(),"definition_callbacks":false,"theorem_bodies_executed":false});
        retain(
            &label,
            "observation.json",
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
        );
        assert_eq!(actual, expected, "all right bytes for{left}");
    }
    assert_eq!(total, 65536);
    println!("all65536 unsigned byte pairs,four comparisons:{steps} actual AST steps");
}

fn observe(name: &str, model: &mut order_eval::Interpreter<'_>, a: u32, b: u32) -> bool {
    let value = model.invoke(&format!("Mpk.Go.Practical.String.Order.{name}"), &[a, b]);
    model.boolean(value)
}
#[test]
fn string_order_observations_preserve_full_lengths_content_and_normalization() {
    let c = baseline();
    let mut cases = vec![];
    for n in [
        0, 1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 129, 254, 255, 256,
    ] {
        let prefix = (0..n).map(|i| (i * 37 + 129) as u8).collect::<Vec<_>>();
        let mut a = [255u8; 256];
        a[..n].copy_from_slice(&prefix);
        let mut b = [23u8; 256];
        b[..n].copy_from_slice(&prefix);
        cases.push((format!("boundary-{n}-same-dirty-tail"), n, n, a, b));
        if n > 0 {
            let mut first = b;
            first[0] ^= 128;
            cases.push((format!("boundary-{n}-first-high-bit"), n, n, a, first));
            let mut last = b;
            last[n - 1] ^= 1;
            cases.push((format!("boundary-{n}-last-low-bit"), n, n, a, last));
            cases.push((format!("boundary-{n}-short-prefix"), n - 1, n, a, b));
        }
    }
    for (label, a, b) in [
        ("embedded-nul", b"a\0b".as_slice(), b"a\0b".as_slice()),
        (
            "invalid-utf8",
            b"\xff\x80\0".as_slice(),
            b"\xff\x81\0".as_slice(),
        ),
        ("multibyte-equal", "日本語".as_bytes(), "日本語".as_bytes()),
        ("unicode-distinct", "é".as_bytes(), "e\u{301}".as_bytes()),
        ("zero-tail-prefix", b"a".as_slice(), b"a\0".as_slice()),
        ("full-length-distinction", b"".as_slice(), &[0u8; 256]),
    ] {
        let mut left = [161u8; 256];
        left[..a.len()].copy_from_slice(a);
        let mut right = [91u8; 256];
        right[..b.len()].copy_from_slice(b);
        cases.push((label.into(), a.len(), b.len(), left, right));
    }
    assert_eq!(cases.len(), 75);
    let mut total = 0;
    for (label, n, m, a, b) in cases {
        let label = format!("string-{label}");
        let expected = vec![
            a[..n] < b[..m],
            a[..n] <= b[..m],
            a[..n] > b[..m],
            a[..n] >= b[..m],
            b[..m] < a[..n],
            b[..m] <= a[..n],
            b[..m] > a[..n],
            b[..m] >= a[..n],
        ];
        let input = serde_json::json!({"lengths":[n,m],"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"expected_both_directions":expected,"source_or_nominal_authority":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut actual = vec![];
        for normalized in [false, true] {
            let mut model = order_eval::Interpreter::new(&c);
            let left = model.raw(n as u64, a);
            let right = model.raw(m as u64, b);
            assert_eq!(
                [model.length(left), model.length(right)],
                [n as u64, m as u64]
            );
            assert_eq!(model.bytes(left), a);
            assert_eq!(model.bytes(right), b);
            let (left, right) = if normalized {
                (
                    model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[left]),
                    model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[right]),
                )
            } else {
                (left, right)
            };
            let mut row = vec![];
            for (x, y) in [(left, right), (right, left)] {
                for op in ["Less", "LessEqual", "Greater", "GreaterEqual"] {
                    row.push(observe(op, &mut model, x, y));
                    retain(&format!("{label}-{}-{:02}",if normalized{"normalized"}else{"raw"},row.len()),"partial.json",format!("{}\n",serde_json::json!({"available_results":row,"actual_AST_steps":model.steps()})).as_bytes());
                }
            }
            let result = serde_json::json!({"actual":row,"expected":expected,"actual_AST_steps":model.steps(),"complete_carrier_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false});
            retain(
                &format!("{label}-{}", if normalized { "normalized" } else { "raw" }),
                "observation.json",
                format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
            );
            assert_eq!(row, expected, "{label} normalized={normalized}");
            actual.push(row);
            total += model.steps();
        }
        assert_eq!(actual[0], actual[1]);
    }
    println!("75 whole string cases,four comparisons,both operand orders,raw and normalized:{total} actual AST steps;finite observation grants no source proof");
}
#[test]
fn wrong_order_definitions_have_typed_prefixes_and_real_counterexamples() {
    let mut modes = vec![
        "always-true".to_string(),
        "always-false".into(),
        "greater-identity".into(),
        "less-equal-strict".into(),
        "greater-equal-strict".into(),
        "no-sentinel".into(),
        "ignore-end".into(),
        "low8-length".into(),
        "high-first".into(),
    ];
    for d in 1..=8 {
        modes.push(format!("omit-high-{d}"));
        modes.push(format!("ignore-prior-{d}"));
    }
    assert_eq!(modes.len(), 25);
    let mut prefixes = vec![];
    for mode in modes {
        let mut c = baseline();
        let depth = mode.rsplit('-').next().unwrap().parse::<usize>().ok();
        let definition = if let Some(d) = depth {
            format!("D{d}.Less")
        } else if mode == "high-first" {
            "D1.Less".into()
        } else if ["no-sentinel", "ignore-end", "low8-length"].contains(&mode.as_str()) {
            "Rank".into()
        } else {
            match mode.as_str() {
                "greater-identity" => "Greater",
                "less-equal-strict" => "LessEqual",
                "greater-equal-strict" => "GreaterEqual",
                _ => "Less",
            }
            .into()
        };
        let full = format!("Mpk.Go.Practical.String.Order.{definition}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == full)
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind
        else {
            unreachable!()
        };
        let changed = replace_leaf(
            &mut c,
            value,
            if definition == "Rank" { 9 } else { 2 },
            &mut |c, leaf| {
                if mode == "always-true" {
                    return flag(c, true);
                }
                if mode == "always-false" {
                    return flag(c, false);
                }
                if [
                    "greater-identity",
                    "less-equal-strict",
                    "greater-equal-strict",
                ]
                .contains(&mode.as_str())
                {
                    let s = term(c, TermNode::Var(1));
                    let t = term(c, TermNode::Var(0));
                    return call(
                        c,
                        if mode == "greater-equal-strict" {
                            "Mpk.Go.Practical.String.Order.Greater"
                        } else {
                            "Mpk.Go.Practical.String.Order.Less"
                        },
                        vec![s, t],
                    );
                }
                let TermNode::App {
                    function,
                    mut arguments,
                } = c.term_table[leaf as usize].clone()
                else {
                    panic!("actual rank/comparison definition")
                };
                if definition == "Rank" {
                    assert_eq!(arguments.len(), 4);
                    if mode == "ignore-end" {
                        return arguments[2];
                    }
                    if mode == "no-sentinel" {
                        let TermNode::App {
                            arguments: ref old, ..
                        } = c.term_table[arguments[2] as usize]
                        else {
                            unreachable!()
                        };
                        arguments[2] = old[0];
                    } else {
                        let TermNode::App {
                            function: less,
                            arguments: ref pair,
                        } = c.term_table[arguments[3] as usize]
                        else {
                            unreachable!()
                        };
                        let mut pair = pair.clone();
                        let s = term(c, TermNode::Var(8));
                        let s = call(c, "Mpk.Go.Practical.String.Length", vec![s]);
                        pair[1] = call(c, "Mpk.Go.Practical.Word.Numeric.W64.Digits8", vec![s]);
                        arguments[3] = term(
                            c,
                            TermNode::App {
                                function: less,
                                arguments: pair,
                            },
                        );
                    }
                    return term(
                        c,
                        TermNode::App {
                            function,
                            arguments,
                        },
                    );
                }
                assert_eq!(arguments.len(), 2);
                let TermNode::App {
                    function: and,
                    arguments: ref right,
                } = c.term_table[arguments[1] as usize]
                else {
                    unreachable!()
                };
                let right = right.clone();
                if mode.starts_with("omit-high") {
                    return arguments[0];
                }
                if mode.starts_with("ignore-prior") {
                    arguments[1] = right[1];
                    return term(
                        c,
                        TermNode::App {
                            function,
                            arguments,
                        },
                    );
                }
                assert_eq!(mode, "high-first");
                let f = term(c, TermNode::Var(1));
                let g = term(c, TermNode::Var(0));
                let yes = flag(c, true);
                let f = term(
                    c,
                    TermNode::App {
                        function: f,
                        arguments: vec![yes],
                    },
                );
                let g = term(
                    c,
                    TermNode::App {
                        function: g,
                        arguments: vec![yes],
                    },
                );
                let high_equal = call(c, "Mpk.Go.Practical.String.Order.D0.Equal", vec![f, g]);
                let last = term(
                    c,
                    TermNode::App {
                        function: and,
                        arguments: vec![high_equal, arguments[0]],
                    },
                );
                term(
                    c,
                    TermNode::App {
                        function,
                        arguments: vec![right[1], last],
                    },
                )
            },
        );
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value: changed,
            reducibility,
        };
        let raw = encode(c.clone());
        let label = format!("operation-{mode}");
        retain(&label, "mpcert", &raw);
        let mut prefix = c.clone();
        prefix.declarations.truncate(at + 1);
        let typed = mpk_kernel::check_declarations(&prefix).is_ok();
        prefixes.push(serde_json::json!({"case":mode,"actual_definition_prefix_typechecks":typed,"full_certificate_acceptance_claimed":false}));
        assert!(typed, "typed prefix:{mode}");
        inspect_saved(&label, &raw, false);
        let (mut n, mut m) = (1usize, 1usize);
        let mut a = [0u8; 256];
        let mut b = [0u8; 256];
        let mut op = "Less";
        match mode.as_str() {
            "always-true" => {}
            "always-false" => b[0] = 1,
            "greater-identity" => {
                a[0] = 1;
                op = "Greater"
            }
            "less-equal-strict" => op = "LessEqual",
            "greater-equal-strict" => op = "GreaterEqual",
            "no-sentinel" | "ignore-end" => n = 0,
            "low8-length" => {
                n = 0;
                m = 256
            }
            "high-first" => {
                n = 2;
                m = 2;
                a[1] = 1;
                b[0] = 1
            }
            _ => {
                let split = 1 << (depth.unwrap() - 1);
                n = split + 1;
                m = n;
                if mode.starts_with("omit-high") {
                    b[split] = 1
                } else {
                    a[0] = 1;
                    b[split] = 1
                }
            }
        }
        let expected = match op {
            "Less" => a[..n] < b[..m],
            "Greater" => a[..n] > b[..m],
            "LessEqual" => a[..n] <= b[..m],
            "GreaterEqual" => a[..n] >= b[..m],
            _ => unreachable!(),
        };
        let input = serde_json::json!({"lengths":[n,m],"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"operation":op,"expected":expected,"canonical_padding":true,"source_or_nominal_authority":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut model = order_eval::Interpreter::new(&c);
        let left = model.raw(n as u64, a);
        let right = model.raw(m as u64, b);
        let actual = observe(op, &mut model, left, right);
        let result = serde_json::json!({"actual":actual,"expected":expected,"actual_definition_counterexample":actual!=expected,"actual_AST_steps":model.steps(),"rejected_theorem_bodies_executed":false});
        retain(
            &label,
            "counterexample.json",
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
        );
        assert_ne!(actual, expected, "real semantic counterexample:{mode}");
    }
    retain(
        "definition-controls",
        "prefix.json",
        format!("{}\n", serde_json::to_string_pretty(&prefixes).unwrap()).as_bytes(),
    );
}
