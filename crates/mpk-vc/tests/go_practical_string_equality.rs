use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_equality_foundation,
    generate_go_practical_string_normalization_foundation,
    import_go_practical_string_equality_foundation, GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_EQUALITY_OUTPUT") {
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringEqualityFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoStringEqualityFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_equality_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn generic_equality_preserves_all1143_original_identities() {
    let f = foundation();
    inspect("equality-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_equality_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_normalization_foundation().unwrap();
    let golden=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-normalization/epochs/typed-controls/selected/candidates/normalization-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 1143);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    assert_eq!(c.declarations.len(), 1215);
    println!("string equality:declarations={} terms={} bytes={} all1143 prior identities exact;13 definitions59 ordinary laws",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len());
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Equality.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("equality-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_equality_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_equality_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}
#[test]
fn universal_equality_laws_require_all_actual_conclusions_and_witnesses() {
    let base = baseline();
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with("Mpk.Go.Practical.String.Equality.")
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 59);
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
                name.strip_prefix("Mpk.Go.Practical.String.Equality.")
                    .unwrap()
            ),
            &encode(c),
            false,
        );
    }
    let mut conditions = vec![
        ("AndLeft".to_string(), 2),
        ("AndRight".into(), 2),
        ("BitReflect".into(), 2),
        ("AndComplete".into(), 2),
        ("AndComplete".into(), 3),
        ("BitComplete".into(), 2),
        ("TruthIff".into(), 2),
        ("TruthIff".into(), 3),
    ];
    for d in 0..=8 {
        conditions.push((format!("D{d}.Reflect"), 1));
        conditions.push((format!("D{d}.Complete"), 1));
    }
    for (name, index) in [
        ("ByteReflect", 2),
        ("ByteComplete", 2),
        ("ByteSameSym", 2),
        ("ByteSameTrans", 3),
        ("ByteSameTrans", 4),
        ("ByteFromEquality", 2),
        ("EquivalentLength", 2),
        ("EquivalentContents", 2),
        ("EquivalentSym", 2),
        ("EquivalentTrans", 3),
        ("EquivalentTrans", 4),
        ("FromStrongEquivalent", 2),
        ("EqualReflect", 2),
        ("EqualComplete", 2),
        ("DifferentLength", 2),
        ("EqualTrans", 3),
        ("EqualTrans", 4),
        ("EqualRespects", 4),
        ("EqualRespects", 5),
        ("NotEqualRespects", 4),
        ("NotEqualRespects", 5),
    ] {
        conditions.push((name.into(), index));
    }
    assert_eq!(conditions.len(), 47);
    for (name, position) in conditions {
        let mut c = base.clone();
        let full = format!("Mpk.Go.Practical.String.Equality.{name}");
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
    println!("equality controls:59 actual changed conclusions,47 erased real witnesses");
}

#[path = "support/go_string_equality_eval.rs"]
mod equality_eval;

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
fn wrong_definitions_have_typed_prefixes_and_actual_semantic_counterexamples() {
    let mut modes = vec![
        "always-true".to_string(),
        "always-false".into(),
        "ignore-length".into(),
        "low8-length".into(),
        "length-only".into(),
        "compare-tail".into(),
        "not-equal-identity".into(),
    ];
    modes.extend((0..8).map(|i| format!("omit-bit{i}")));
    assert_eq!(modes.len(), 15);
    let mut prefixes = vec![];
    for mode in modes {
        let mut c = baseline();
        let definition = if mode.starts_with("omit-") {
            "ByteEqual"
        } else if mode == "not-equal-identity" {
            "NotEqual"
        } else {
            "Equal"
        };
        let full = format!("Mpk.Go.Practical.String.Equality.{definition}");
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
        let changed = replace_leaf(&mut c, value, 2, &mut |c, leaf| {
            if mode == "always-true" {
                return flag(c, true);
            }
            if mode == "always-false" {
                return flag(c, false);
            }
            if mode == "not-equal-identity" {
                let s = term(c, TermNode::Var(1));
                let t = term(c, TermNode::Var(0));
                return call(c, "Mpk.Go.Practical.String.Equality.Equal", vec![s, t]);
            }
            let TermNode::App {
                function,
                mut arguments,
            } = c.term_table[leaf as usize].clone()
            else {
                panic!("actual ordinary equality operation")
            };
            if let Some(index) = mode.strip_prefix("omit-bit") {
                let index = index.parse::<usize>().unwrap();
                assert_eq!(arguments.len(), 1);
                arguments[0] = replace_leaf(c, arguments[0], 3, &mut |c, old| {
                    let mut condition = flag(c, true);
                    for j in 0..3 {
                        let bit = term(c, TermNode::Var(j));
                        let target = flag(c, index & (1 << j) != 0);
                        let same = call(c, "Mpk.Go.Practical.Bool.Equal", vec![bit, target]);
                        condition = call(c, "Mpk.Go.Practical.Bool.And", vec![condition, same]);
                    }
                    let boolean = sym(c, "Std.Bool");
                    let motive = term(
                        c,
                        TermNode::Lam {
                            ty: boolean,
                            body: boolean,
                        },
                    );
                    let yes = flag(c, true);
                    call(c, "Std.Bool.cases", vec![motive, old, yes, condition])
                });
                return term(
                    c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                );
            }
            assert_eq!(arguments.len(), 2);
            match mode.as_str() {
                "ignore-length" => arguments[0],
                "length-only" => arguments[1],
                "low8-length" => {
                    let s = term(c, TermNode::Var(1));
                    let t = term(c, TermNode::Var(0));
                    let s = call(c, "Mpk.Go.Practical.String.Length", vec![s]);
                    let t = call(c, "Mpk.Go.Practical.String.Length", vec![t]);
                    let s = call(c, "Mpk.Go.Practical.Word.Numeric.W64.Digits8", vec![s]);
                    let t = call(c, "Mpk.Go.Practical.Word.Numeric.W64.Digits8", vec![t]);
                    arguments[1] = call(c, "Mpk.Go.Practical.Nat.Equal", vec![s, t]);
                    term(
                        c,
                        TermNode::App {
                            function,
                            arguments,
                        },
                    )
                }
                "compare-tail" => {
                    let TermNode::App {
                        function: fold,
                        arguments: mut body,
                    } = c.term_table[arguments[0] as usize].clone()
                    else {
                        panic!("actual prefix fold")
                    };
                    assert_eq!(body.len(), 1);
                    body[0] = replace_leaf(c, body[0], 8, &mut |c, guard| {
                        let TermNode::App { arguments, .. } = &c.term_table[guard as usize] else {
                            panic!("actual within-length byte choice")
                        };
                        assert_eq!(arguments.len(), 4);
                        arguments[2]
                    });
                    arguments[0] = term(
                        c,
                        TermNode::App {
                            function: fold,
                            arguments: body,
                        },
                    );
                    term(
                        c,
                        TermNode::App {
                            function,
                            arguments,
                        },
                    )
                }
                _ => unreachable!(),
            }
        });
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
        assert!(typed, "typed operation prefix:{mode}");
        prefixes.push(serde_json::json!({"case":mode,"actual_definition_prefix_typechecks":typed,"full_certificate_acceptance_claimed":false}));
        inspect_saved(&label, &raw, false);
        let (n, m, left, right, operator) = match mode.as_str() {
            "always-true" | "length-only" => (1, 1, 0, 1, "Equal"),
            "always-false" => (1, 1, 0, 0, "Equal"),
            "ignore-length" => (0, 1, 0, 0, "Equal"),
            "low8-length" => (0, 256, 0, 0, "Equal"),
            "compare-tail" => (0, 0, 0, 255, "Equal"),
            "not-equal-identity" => (1, 1, 0, 0, "NotEqual"),
            _ => (
                1,
                1,
                0,
                1u8 << mode
                    .strip_prefix("omit-bit")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
                "Equal",
            ),
        };
        let mut a = [0u8; 256];
        let mut b = [0u8; 256];
        a[..n].fill(left);
        b[..m].fill(right);
        if mode == "compare-tail" {
            b.fill(255);
        }
        let expected = if operator == "Equal" {
            a[..n] == b[..m]
        } else {
            a[..n] != b[..m]
        };
        let input = serde_json::json!({"lengths":[n,m],"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"operation":operator,"expected":expected,"source_or_nominal_authority":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut model = equality_eval::Interpreter::new(&c);
        let a = model.raw(n as u64, a);
        let b = model.raw(m as u64, b);
        let value = model.invoke(
            &format!("Mpk.Go.Practical.String.Equality.{operator}"),
            &[a, b],
        );
        let actual = model.boolean(value);
        let result = serde_json::json!({"actual":actual,"expected":expected,"actual_AST_steps":model.steps(),"actual_definition_counterexample":actual!=expected,"rejected_theorem_bodies_executed":false});
        retain(
            &label,
            "counterexample.json",
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
        );
        assert_ne!(actual, expected, "actual wrong operation:{mode}");
    }
    retain(
        "definition-controls",
        "prefix.json",
        format!("{}\n", serde_json::to_string_pretty(&prefixes).unwrap()).as_bytes(),
    );
}

#[test]
fn byte_equality_observations_cover_all65536_pairs() {
    let c = baseline();
    let mut total = 0;
    let mut steps = 0;
    for left in 0..=255u8 {
        let label = format!("byte-{left}");
        let input = serde_json::json!({"left_byte":left,"all_right_bytes":(0..=255u16).collect::<Vec<_>>(),"finite_observation_supplies_universal_proof":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut model = equality_eval::Interpreter::new(&c);
        let a = model.byte_input(left);
        let mut observed = vec![];
        let mut expected = vec![];
        for right in 0..=255u8 {
            let b = model.byte_input(right);
            let result = model.invoke("Mpk.Go.Practical.String.Equality.ByteEqual", &[a, b]);
            observed.push(model.boolean(result));
            expected.push(left == right);
            total += 1;
        }
        steps += model.steps();
        let result = serde_json::json!({"actual":observed,"expected":expected,"actual_AST_steps":model.steps(),"definition_callbacks":false,"theorem_bodies_executed":false});
        retain(
            &label,
            "observation.json",
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
        );
        assert_eq!(observed, expected, "all right bytes for{left}");
    }
    assert_eq!(total, 65536);
    println!("all65536 byte equality pairs execute actual Bool definitions;{steps} AST steps;universal proof comes from both cores");
}

#[test]
fn string_equality_observations_preserve_length_content_and_normalization() {
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
    let mut total_steps = 0;
    for (label, n, m, a, b) in cases {
        let label = format!("string-{label}");
        let expected = a[..n] == b[..m];
        let input = serde_json::json!({"lengths":[n,m],"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"expected":expected,"source_or_nominal_authority":false});
        retain(
            &label,
            "input.json",
            format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
        );
        let mut model = equality_eval::Interpreter::new(&c);
        let left = model.raw(n as u64, a);
        let right = model.raw(m as u64, b);
        let normalized_left = model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[left]);
        let normalized_right =
            model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[right]);
        let mut observed = vec![];
        for (left, right) in [(left, right), (normalized_left, normalized_right)] {
            for operation in ["Equal", "NotEqual"] {
                let result = model.invoke(
                    &format!("Mpk.Go.Practical.String.Equality.{operation}"),
                    &[left, right],
                );
                observed.push(model.boolean(result));
            }
        }
        let lengths = [
            model.length(left),
            model.length(right),
            model.length(normalized_left),
            model.length(normalized_right),
        ];
        let actual_a = model.bytes(left);
        let actual_b = model.bytes(right);
        let steps = model.steps();
        total_steps += steps;
        let result = serde_json::json!({"actual":observed,"expected":[expected,!expected,expected,!expected],"full_word_lengths":lengths,"all256_left_bytes":actual_a.to_vec(),"all256_right_bytes":actual_b.to_vec(),"actual_AST_steps":steps,"definition_callbacks":false,"theorem_bodies_executed":false});
        retain(
            &label,
            "observation.json",
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap()).as_bytes(),
        );
        assert_eq!(observed, [expected, !expected, expected, !expected]);
        assert_eq!(lengths, [n as u64, m as u64, n as u64, m as u64]);
        assert_eq!(actual_a, a);
        assert_eq!(actual_b, b);
    }
    println!("actual string Equal/NotEqual:75 whole raw/normalized cases,full64 lengths/all8 byte bits,dirty tails/empty/full/UTF8/NUL/binary boundaries;{total_steps} AST steps;no native Go source/program proof");
}
