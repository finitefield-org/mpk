use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_error_value_foundation,
    generate_go_practical_product_relation_foundation, generate_go_practical_word_foundation,
    import_go_practical_error_value_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[allow(dead_code)]
#[path = "support/go_word_eval.rs"]
mod word_eval;

const PREFIX: &str = "Mpk.Go.Practical.ErrorValue";

fn baseline() -> Certificate {
    static BASE: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
        mpk_cert::decode_canonical_certificate(
            generate_go_practical_error_value_foundation()
                .unwrap()
                .certificate_bytes(),
        )
        .unwrap()
    })
    .clone()
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_ERROR_VALUES_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap()
                .write_all(&raw)
                .unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
    if accepted {
        assert_eq!(report["axiom_count"], 0);
        assert!(report["axiom_report"]["entries"]
            .as_array()
            .unwrap()
            .is_empty());
    } else {
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name) as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = constant(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn ec(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("{PREFIX}.{name}"), args)
}
fn bit(c: &mut Certificate, b: bool) -> u32 {
    constant(c, if b { "Std.Bool.true" } else { "Std.Bool.false" })
}
fn literal(c: &mut Certificate, value: u64) -> u32 {
    fn tree(c: &mut Certificate, value: u64, start: usize, width: usize) -> u32 {
        if width == 1 {
            return bit(c, value & (1u64 << start) != 0);
        }
        let no = tree(c, value, start, width / 2);
        let yes = tree(c, value, start + width / 2, width / 2);
        let condition = term(c, TermNode::Var(width.trailing_zeros() - 1));
        call(c, "Std.Bool.rec", vec![no, yes, condition])
    }
    let boolean = constant(c, "Std.Bool");
    let mut value = tree(c, value, 0, 64);
    for _ in 0..6 {
        value = term(
            c,
            TermNode::Lam {
                ty: boolean,
                body: value,
            },
        );
    }
    value
}
fn equation(c: &mut Certificate, ty: u32, left: u32, right: u32, honest: bool, name: &str) {
    let proposition = call(c, "Std.Eq", vec![ty, left, right]);
    let proof = call(
        c,
        "Std.Eq.refl",
        vec![ty, if honest { right } else { left }],
    );
    let name = {
        let n = c.name_table.len() as u32;
        c.name_table.push(format!(
            "Control.ErrorValue.Case_{}",
            name.replace('-', "_")
        ));
        n
    };
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Theorem {
            ty: proposition,
            proof,
        },
    });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(n) = level {
            *n = c.name_table.binary_search(&old[*n as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131_072);
    assert!(c.imports.is_empty());
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}

#[test]
fn actual_full_width_error_carrier_retains_entire_word_and_product_libraries() {
    let bytes = generate_go_practical_error_value_foundation().unwrap();
    inspect("error-values", bytes.certificate_bytes(), true);
    let c = baseline();
    let names = dependencies::declarations(&c);
    for (prior, count, digest) in [
        (
            generate_go_practical_word_foundation()
                .unwrap()
                .certificate_bytes()
                .to_vec(),
            1892,
            "23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c",
        ),
        (
            generate_go_practical_product_relation_foundation()
                .unwrap()
                .certificate_bytes()
                .to_vec(),
            94,
            "584cbf2e2a70ce0d86f7703b754e9a22fd84dcd59ffd42e6df00e22fdb2a0d1d",
        ),
    ] {
        let old = mpk_cert::decode_canonical_certificate(&prior).unwrap();
        assert_eq!(old.declarations.len(), count);
        assert_eq!(format!("{:x}", Sha256::digest(&prior)), digest);
        for (name, identity) in dependencies::declarations(&old) {
            assert_eq!(names.get(&name), Some(&identity), "{name}");
        }
        println!(
            "retained declarations={} bytes={} sha256={:x}",
            old.declarations.len(),
            prior.len(),
            Sha256::digest(&prior)
        );
    }
    assert_eq!(
        import_go_practical_error_value_foundation(bytes.certificate_bytes()).unwrap(),
        bytes
    );
    println!(
        "error-values declarations={} terms={} bytes={} sha256={:x}",
        c.declarations.len(),
        c.term_table.len(),
        bytes.certificate_bytes().len(),
        Sha256::digest(bytes.certificate_bytes())
    );
}

fn boundary_certificate() -> Vec<u8> {
    let mut c = baseline();
    let boolean = constant(&mut c, "Std.Bool");
    let payload_type = call(
        &mut c,
        "Mpk.Go.Practical.Product.Type",
        vec![boolean, boolean],
    );
    // Both fields and their differing values must survive copying; no selected
    // payload field or inactive zero is discarded by this generic component.
    let f = bit(&mut c, false);
    let t = bit(&mut c, true);
    let payload = call(
        &mut c,
        "Mpk.Go.Practical.Product.Make",
        vec![boolean, boolean, t, f],
    );
    for tag_value in [0, 1, 2, 1u64 << 63, u64::MAX] {
        let tag = literal(&mut c, tag_value);
        let e = ec(&mut c, "Make", vec![payload_type, tag, payload]);
        let copy = ec(&mut c, "Copy", vec![payload_type, e]);
        for (label, v) in [("made", e), ("copied", copy)] {
            let result = ec(&mut c, "Tag", vec![payload_type, v]);
            let word_type = constant(&mut c, "Mpk.Go.Practical.Word.W64.Carrier");
            equation(
                &mut c,
                word_type,
                result,
                tag,
                true,
                &format!("{tag_value}-{label}-tag"),
            );
            let result = ec(&mut c, "Payload", vec![payload_type, v]);
            equation(
                &mut c,
                payload_type,
                result,
                payload,
                true,
                &format!("{tag_value}-{label}-payload"),
            );
        }
    }
    encode(c)
}

#[test]
fn actual_nil_zero_high_tag_and_complete_payload_boundary_claims() {
    let bytes = boundary_certificate();
    inspect("error-boundaries", &bytes, true);
    assert_eq!(
        import_go_practical_error_value_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = generate_go_practical_error_value_foundation()
        .unwrap()
        .certificate_bytes()
        .to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_error_value_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_error_value_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    observe_all_tag_bits_and_payload_fields();
}

fn observe_all_tag_bits_and_payload_fields() {
    use serde_json::json;
    let c = baseline();
    let tags: std::collections::BTreeSet<_> = [0, u64::MAX]
        .into_iter()
        .chain((0..64).map(|i| 1u64 << i))
        .collect();
    let mut rows = vec![];
    let mut total_steps = 0;
    for tag in tags {
        for left in [false, true] {
            for right in [false, true] {
                let mut model = word_eval::Interpreter::new(&c);
                let bool_type = model.invoke("Std.Bool", &[]);
                let left_bit = model.invoke(
                    if left {
                        "Std.Bool.true"
                    } else {
                        "Std.Bool.false"
                    },
                    &[],
                );
                let right_bit = model.invoke(
                    if right {
                        "Std.Bool.true"
                    } else {
                        "Std.Bool.false"
                    },
                    &[],
                );
                let payload_type =
                    model.invoke("Mpk.Go.Practical.Product.Type", &[bool_type, bool_type]);
                let payload = model.invoke(
                    "Mpk.Go.Practical.Product.Make",
                    &[bool_type, bool_type, left_bit, right_bit],
                );
                let tag_value = model.word(64, tag);
                let e = model.invoke(
                    &format!("{PREFIX}.Make"),
                    &[payload_type, tag_value, payload],
                );
                let copy = model.invoke(&format!("{PREFIX}.Copy"), &[payload_type, e]);
                for (kind, value) in [("made", e), ("copied", copy)] {
                    let nil_value =
                        model.invoke(&format!("{PREFIX}.IsNil"), &[payload_type, value]);
                    let nil = model.boolean(nil_value);
                    let non_nil_value =
                        model.invoke(&format!("{PREFIX}.IsNonNil"), &[payload_type, value]);
                    let non_nil = model.boolean(non_nil_value);
                    assert_eq!((nil, non_nil), (tag == 0, tag != 0));
                    let observed_tag =
                        model.invoke(&format!("{PREFIX}.Tag"), &[payload_type, value]);
                    assert_eq!(model.bits(64, observed_tag), tag);
                    let observed_payload =
                        model.invoke(&format!("{PREFIX}.Payload"), &[payload_type, value]);
                    let observed_left = model.invoke(
                        "Mpk.Go.Practical.Product.Left",
                        &[bool_type, bool_type, observed_payload],
                    );
                    let observed_right = model.invoke(
                        "Mpk.Go.Practical.Product.Right",
                        &[bool_type, bool_type, observed_payload],
                    );
                    assert_eq!(
                        (model.boolean(observed_left), model.boolean(observed_right)),
                        (left, right)
                    );
                    let mut comparisons = vec![];
                    for case in [0, 1, 2, 1u64 << 63, u64::MAX, tag]
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>()
                    {
                        let case_value = model.word(64, case);
                        let matched = model.invoke(
                            &format!("{PREFIX}.HasCase"),
                            &[payload_type, case_value, value],
                        );
                        let matched = model.boolean(matched);
                        assert_eq!(matched, case == tag);
                        comparisons
                            .push(json!({"case_hex": format!("{case:016x}"), "matched": matched}));
                    }
                    rows.push(
                        json!({"tag_hex": format!("{tag:016x}"), "payload": [left, right],
                        "value": kind, "nil": nil, "non_nil": non_nil, "cases": comparisons,
                        "entire_tag_and_both_payload_fields_retained": true}),
                    );
                }
                total_steps += model.steps();
            }
        }
    }
    assert_eq!(rows.len(), 528);
    if let Some(root) = std::env::var_os("MPK_GO_ERROR_VALUES_TEST_OUTPUT") {
        use std::io::Write;
        let raw = serde_json::to_vec_pretty(&json!({"classification": "Finite execution of actual certificate terms; no source or universal-proof substitute.",
            "rows": rows, "total_interpreter_steps": total_steps})).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(root).join("finite-observations.json"))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
    println!("finite actual-DAG observations=528 all64 single-bit tags/all4 payloads; steps={total_steps}");
}

fn changed_result(c: &mut Certificate, ty: u32, result: u32) -> u32 {
    if let TermNode::Pi { ty: domain, body } = c.term_table[ty as usize].clone() {
        let body = changed_result(c, body, result);
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        result
    }
}
fn changed_domain(c: &mut Certificate, ty: u32, index: usize, replacement: u32) -> u32 {
    let TermNode::Pi { ty: domain, body } = c.term_table[ty as usize].clone() else {
        panic!("binder")
    };
    if index == 0 {
        term(
            c,
            TermNode::Pi {
                ty: replacement,
                body,
            },
        )
    } else {
        let body = changed_domain(c, body, index - 1, replacement);
        term(c, TermNode::Pi { ty: domain, body })
    }
}
#[test]
fn every_new_law_and_actual_nonzero_premise_must_typecheck() {
    let original = baseline();
    for d in &original.declarations {
        let name = &original.name_table[d.name as usize];
        if !name.starts_with(PREFIX) {
            continue;
        }
        let DeclarationKind::Theorem { ty, proof } = d.kind else {
            continue;
        };
        let mut c = original.clone();
        let boolean = constant(&mut c, "Std.Bool");
        let no = bit(&mut c, false);
        let yes = bit(&mut c, true);
        let wrong = call(&mut c, "Std.Eq", vec![boolean, no, yes]);
        let ty = changed_result(&mut c, ty, wrong);
        let id = global(&c, name);
        c.declarations[id].kind = DeclarationKind::Theorem { ty, proof };
        inspect(
            &format!("false-{}", name.rsplit('.').next().unwrap()),
            &encode(c),
            false,
        );
    }
    let mut premises = vec![
        ("NonzeroMakeIsNotNil".to_owned(), 3),
        ("NonzeroMakeIsNonNil".to_owned(), 3),
        ("NilHasNoNonzeroCase".to_owned(), 3),
        ("AndFalseLeft".to_owned(), 2),
        ("AndFalseRight".to_owned(), 2),
    ];
    premises.extend((0..64).map(|i| (format!("TagDifferentAt{i}"), 2)));
    for (name, index) in premises {
        let mut c = original.clone();
        let id = global(&c, &format!("{PREFIX}.{name}"));
        let DeclarationKind::Theorem { ty, proof } = c.declarations[id].kind else {
            panic!()
        };
        let boolean = constant(&mut c, "Std.Bool");
        let yes = bit(&mut c, true);
        let trivial = call(&mut c, "Std.Eq", vec![boolean, yes, yes]);
        let ty = changed_domain(&mut c, ty, index, trivial);
        c.declarations[id].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("erased-{name}"), &encode(c), false);
    }
}

fn changed_body(
    c: &mut Certificate,
    value: u32,
    replace: &mut impl FnMut(&mut Certificate) -> u32,
) -> u32 {
    if let TermNode::Lam { ty, body } = c.term_table[value as usize].clone() {
        let body = changed_body(c, body, replace);
        term(c, TermNode::Lam { ty, body })
    } else {
        replace(c)
    }
}
fn corruption(label: &str, member: &str, mut replace: impl FnMut(&mut Certificate) -> u32) {
    let mut c = baseline();
    let id = global(&c, &format!("{PREFIX}.{member}"));
    let DeclarationKind::Def { ty, value, .. } = c.declarations[id].kind else {
        panic!()
    };
    let value = changed_body(&mut c, value, &mut replace);
    c.declarations[id].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility: DefinitionReducibility::Reducible,
    };
    inspect(label, &encode(c), false);
}
#[test]
fn complete_tags_payloads_and_nil_comparisons_cannot_be_replaced() {
    corruption("wrong-nil-tag", "Nil", |c| {
        let p = term(c, TermNode::Var(1));
        let zero = term(c, TermNode::Var(0));
        let tag = constant(c, "Mpk.Go.Practical.Word.W64.One");
        ec(c, "Make", vec![p, tag, zero])
    });
    corruption("dropped-copy-tag", "Copy", |c| {
        let p = term(c, TermNode::Var(1));
        let e = term(c, TermNode::Var(0));
        let payload = ec(c, "Payload", vec![p, e]);
        let zero = constant(c, "Mpk.Go.Practical.Word.W64.Zero");
        ec(c, "Make", vec![p, zero, payload])
    });
    corruption("low-bit-nil", "IsNil", |c| {
        let p = term(c, TermNode::Var(1));
        let e = term(c, TermNode::Var(0));
        let tag = ec(c, "Tag", vec![p, e]);
        let no = bit(c, false);
        let low = term(
            c,
            TermNode::App {
                function: tag,
                arguments: vec![no; 6],
            },
        );
        call(c, "Mpk.Go.Practical.Bool.Not", vec![low])
    });
    corruption("constant-case-match", "HasCase", |c| bit(c, true));
    corruption("dropped-payload", "Payload", |c| bit(c, false));

    for (name, tag_value, member, expected) in [
        ("false-concrete-zero-is-nil", 1, "IsNil", true),
        ("false-high-tag-is-nil", 1u64 << 63, "IsNil", true),
        ("false-nil-is-non-nil", 0, "IsNonNil", true),
    ] {
        let mut c = baseline();
        let boolean = constant(&mut c, "Std.Bool");
        let payload = bit(&mut c, false);
        let tag = literal(&mut c, tag_value);
        let e = ec(&mut c, "Make", vec![boolean, tag, payload]);
        let result = ec(&mut c, member, vec![boolean, e]);
        let expected = bit(&mut c, expected);
        equation(&mut c, boolean, result, expected, false, name);
        inspect(name, &encode(c), false);
    }
}
