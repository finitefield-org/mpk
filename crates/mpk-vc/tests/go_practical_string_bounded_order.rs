use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_bounded_order_foundation,
    import_go_practical_string_bounded_order_foundation, GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_BOUNDED_ORDER_OUTPUT") {
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringBoundedOrderFoundation {
    static VALUE: std::sync::OnceLock<
        mpk_vc::go_practical_ordinary::GoStringBoundedOrderFoundation,
    > = std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_bounded_order_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn bounded_order_preserves_all1706_prior_identities() {
    let f = foundation();
    inspect("bounded-order-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_bounded_order_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old =
        mpk_vc::go_practical_ordinary::generate_go_practical_string_order_foundation().unwrap();
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-order/epochs/semantic-controls/selected/candidates/order-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(path).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 1706);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    let laws = c
        .declarations
        .iter()
        .filter(|d| {
            c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Order.Bounded.")
        })
        .count();
    println!("bounded string order:declarations={} terms={} bytes={} all1706 prior identities and old full producer bytes exact;{} new ordinary laws",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len(),laws);
}

#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Order.Bounded.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("bounded-order-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_bounded_order_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_bounded_order_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn bounded_laws_bind_actual_conclusions_and_required_witnesses() {
    let base = baseline();
    let prefix = "Mpk.Go.Practical.String.Order.Bounded.";
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with(prefix) && matches!(d.kind, DeclarationKind::Theorem { .. }))
                .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 23);
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
            &format!("false-{}", name.strip_prefix(prefix).unwrap()),
            &encode(c),
            false,
        );
    }
    let witnesses = [
        ("LengthAddress", 1),
        ("LengthNotLess", 2),
        ("LengthNotLess", 3),
        ("LengthEqual", 2),
        ("LengthEqual", 3),
        ("LengthEqual", 4),
        ("SuccCancel", 2),
        ("RankByteEqual", 2),
        ("RankByteEqual", 11),
        ("RankByteEqual", 12),
        ("RanksReflect", 2),
        ("RanksComplete", 2),
        ("FromPoints", 2),
        ("FromPoints", 3),
        ("FromPoints", 4),
        ("FromRanks", 2),
        ("FromRanks", 3),
        ("FromRanks", 4),
        ("ToRanks", 2),
        ("EqualRanks", 2),
        ("EqualRanks", 3),
        ("TotalEquality", 2),
        ("TotalEquality", 3),
        ("EqualBounds", 2),
        ("EqualBounds", 3),
        ("Antisym", 2),
        ("Antisym", 3),
        ("Antisym", 4),
        ("Antisym", 5),
        ("EqualNoLess", 2),
        ("EqualNoGreater", 2),
        ("Trichotomy", 2),
        ("Trichotomy", 3),
        ("EqualLessEqual", 2),
        ("EqualGreaterEqual", 2),
        ("LessNotEqual", 2),
        ("GreaterNotEqual", 2),
    ];
    assert_eq!(witnesses.len(), 37);
    for (name, position) in witnesses {
        let mut c = base.clone();
        let full = format!("{prefix}{name}");
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
    println!("bounded order controls:23 changed conclusions;37 required typed witness domains");
}

#[path = "support/go_string_equality_eval.rs"]
mod model;
fn call_flag(model: &mut model::Interpreter<'_>, name: &str, args: &[u32]) -> (u32, bool) {
    let id = model.invoke(name, args);
    let flag = model.boolean(id);
    (id, flag)
}
fn save_row(name: &str, value: &serde_json::Value) {
    retain(
        name,
        "json",
        format!("{}\n", serde_json::to_string_pretty(value).unwrap()).as_bytes(),
    );
}
#[test]
fn bounded_order_observations_match_complete_raw_and_normalized_inputs() {
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
    let mut steps = 0;
    for (label, n, m, a, b) in cases {
        let label = format!("bounded-{label}");
        let expected_eq = a[..n] == b[..m];
        let expected = [
            a[..n] < b[..m],
            a[..n] <= b[..m],
            a[..n] > b[..m],
            a[..n] >= b[..m],
            expected_eq,
            expected_eq,
            expected_eq,
            expected_eq,
            true,
            true,
            true,
        ];
        save_row(
            &format!("{label}.input"),
            &serde_json::json!({"lengths":[n,m],"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"expected_columns":expected,"source_or_nominal_authority":false}),
        );
        let mut observations = vec![];
        for normalized in [false, true] {
            let mut model = model::Interpreter::new(&c);
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
            let kind = if normalized { "normalized" } else { "raw" };
            let mut actual = vec![];
            let mut values = vec![];
            for op in ["Less", "LessEqual", "Greater", "GreaterEqual"] {
                let (id, flag) = call_flag(
                    &mut model,
                    &format!("Mpk.Go.Practical.String.Order.{op}"),
                    &[left, right],
                );
                values.push(id);
                actual.push(flag);
                save_row(
                    &format!("{label}-{kind}-{:02}.partial", actual.len()),
                    &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
                );
            }
            let (eq_id, eq) = call_flag(
                &mut model,
                "Mpk.Go.Practical.String.Equality.Equal",
                &[left, right],
            );
            actual.push(eq);
            save_row(
                &format!("{label}-{kind}-05.partial"),
                &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
            );
            let x = model.invoke("Mpk.Go.Practical.String.Order.Ranks", &[left]);
            let y = model.invoke("Mpk.Go.Practical.String.Order.Ranks", &[right]);
            let (_, rank_eq) = call_flag(
                &mut model,
                "Mpk.Go.Practical.String.Order.D8.Equal",
                &[x, y],
            );
            actual.push(rank_eq);
            save_row(
                &format!("{label}-{kind}-06.partial"),
                &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
            );
            let no_less = model.invoke("Mpk.Go.Practical.Bool.Not", &[values[0]]);
            let no_greater = model.invoke("Mpk.Go.Practical.Bool.Not", &[values[2]]);
            let (_, neither) = call_flag(
                &mut model,
                "Mpk.Go.Practical.Bool.And",
                &[no_less, no_greater],
            );
            actual.push(neither);
            save_row(
                &format!("{label}-{kind}-07.partial"),
                &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
            );
            let (_, both) = call_flag(
                &mut model,
                "Mpk.Go.Practical.Bool.And",
                &[values[1], values[3]],
            );
            actual.push(both);
            save_row(
                &format!("{label}-{kind}-08.partial"),
                &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
            );
            let last = model.invoke("Mpk.Go.Practical.Bool.Or", &[eq_id, values[2]]);
            let (_, three) = call_flag(&mut model, "Mpk.Go.Practical.Bool.Or", &[values[0], last]);
            actual.push(three);
            save_row(
                &format!("{label}-{kind}-09.partial"),
                &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
            );
            let cap = model.invoke("Mpk.Go.Practical.FiniteVector.D8.Capacity", &[]);
            for source in [left, right] {
                let size = model.invoke("Mpk.Go.Practical.String.Size", &[source]);
                let too_large = model.invoke("Mpk.Go.Practical.Nat.Less", &[cap, size]);
                let (_, bound) = call_flag(&mut model, "Mpk.Go.Practical.Bool.Not", &[too_large]);
                actual.push(bound);
                save_row(
                    &format!("{label}-{kind}-{:02}.partial", actual.len()),
                    &serde_json::json!({"available_results":actual,"actual_AST_steps":model.steps()}),
                );
            }
            save_row(
                &format!("{label}-{kind}.observation"),
                &serde_json::json!({"actual":actual,"expected":expected,"actual_AST_steps":model.steps(),"definition_callbacks":false,"theorem_bodies_executed":false,"complete_carrier_observed":true}),
            );
            assert_eq!(actual, expected, "{label} {kind}");
            observations.push(actual);
            steps += model.steps();
        }
        assert_eq!(observations[0], observations[1]);
    }
    println!("75 bounded full carriers,raw/normalized;actual rank/string equality,neither-strict,both-weak,trichotomy/full64 bounds:{steps} AST steps");
}

#[test]
fn unbounded_carrier_observations_show_why_both_upper_witnesses_are_required() {
    let c = baseline();
    let mut total = 0;
    for (n, m) in [
        (256u64, 257u64),
        (257, 256),
        (257, 258),
        (511, 512),
        (512, 511),
    ] {
        let label = format!("domain-{n}-{m}");
        let bytes = [0u8; 256];
        save_row(
            &format!("{label}.input"),
            &serde_json::json!({"lengths":[n,m],"all64_left_length_bits":(0..64).map(|i|n&(1<<i)!=0).collect::<Vec<_>>(),"all64_right_length_bits":(0..64).map(|i|m&(1<<i)!=0).collect::<Vec<_>>(),"all256_left_bytes":bytes.to_vec(),"all256_right_bytes":bytes.to_vec(),"source_or_nominal_authority":false,"admitted_source_counterexample":false}),
        );
        let mut model = model::Interpreter::new(&c);
        let left = model.raw(n, bytes);
        let right = model.raw(m, bytes);
        assert_eq!([model.length(left), model.length(right)], [n, m]);
        assert_eq!(model.bytes(left), bytes);
        assert_eq!(model.bytes(right), bytes);
        let x = model.invoke("Mpk.Go.Practical.String.Order.Ranks", &[left]);
        let y = model.invoke("Mpk.Go.Practical.String.Order.Ranks", &[right]);
        let (_, rank_eq) = call_flag(
            &mut model,
            "Mpk.Go.Practical.String.Order.D8.Equal",
            &[x, y],
        );
        save_row(
            &format!("{label}-01.partial"),
            &serde_json::json!({"rank_equal":rank_eq,"actual_AST_steps":model.steps()}),
        );
        let (_, equal) = call_flag(
            &mut model,
            "Mpk.Go.Practical.String.Equality.Equal",
            &[left, right],
        );
        save_row(
            &format!("{label}-02.partial"),
            &serde_json::json!({"rank_equal":rank_eq,"string_equal":equal,"actual_AST_steps":model.steps()}),
        );
        let cap = model.invoke("Mpk.Go.Practical.FiniteVector.D8.Capacity", &[]);
        let mut bounds = vec![];
        for source in [left, right] {
            let count = model.invoke("Mpk.Go.Practical.String.Size", &[source]);
            let too_large = model.invoke("Mpk.Go.Practical.Nat.Less", &[cap, count]);
            let (_, bound) = call_flag(&mut model, "Mpk.Go.Practical.Bool.Not", &[too_large]);
            bounds.push(bound);
            save_row(
                &format!("{label}-bound{}.partial", bounds.len()),
                &serde_json::json!({"rank_equal":rank_eq,"string_equal":equal,"available_upper_bounds":bounds,"actual_AST_steps":model.steps()}),
            );
        }
        let byte = model.byte_input(0);
        let value = model.invoke("Mpk.Go.Practical.Word.Numeric.Value3", &[byte]);
        let (_, nul_zero) = call_flag(&mut model, "Mpk.Go.Practical.Nat.IsZero", &[value]);
        let flag = model.invoke("Std.Bool.false", &[]);
        let mut address = vec![left];
        address.extend([flag; 8]);
        let rank = model.invoke("Mpk.Go.Practical.String.Order.Rank", &address);
        let (_, rank_zero) = call_flag(&mut model, "Mpk.Go.Practical.Nat.IsZero", &[rank]);
        save_row(
            &format!("{label}.observation"),
            &serde_json::json!({"rank_equal":rank_eq,"string_equal":equal,"upper_bounds":bounds,"expected_upper_bounds":[n<=256,m<=256],"NUL_byte_value_is_zero":nul_zero,"present_NUL_rank_is_zero":rank_zero,"actual_AST_steps":model.steps(),"actual_missing_bound_countermodel":rank_eq&&!equal,"admitted_source_counterexample":false,"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert!(rank_eq && !equal && nul_zero && !rank_zero);
        assert_eq!(bounds, [n <= 256, m <= 256]);
        total += model.steps();
    }
    println!("five actual full64 raw carrier countermodels require upper witnesses;none is an admitted Go source string:{total} AST steps");
}
