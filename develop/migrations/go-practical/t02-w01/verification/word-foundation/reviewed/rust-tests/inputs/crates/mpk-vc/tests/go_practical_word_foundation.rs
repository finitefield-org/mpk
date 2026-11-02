use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_range_foundation, generate_go_practical_word_foundation,
    import_go_practical_word_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};

#[path = "support/go_word_eval.rs"]
mod word_eval;

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_WORD_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", r.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{ext}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    if !accepted {
        let report: serde_json::Value = serde_json::from_str(&r.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}

fn global(c: &Certificate, name: &str) -> u32 {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name);
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    let function = constant(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments: args,
        },
    )
}
fn wc(c: &mut Certificate, width: usize, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("Mpk.Go.Practical.Word.W{width}.{name}"), args)
}
fn word_type(c: &mut Certificate, width: usize) -> u32 {
    let boolean = constant(c, "Std.Bool");
    let mut ty = boolean;
    for _ in 0..width.trailing_zeros() {
        ty = term(
            c,
            TermNode::Pi {
                ty: boolean,
                body: ty,
            },
        );
    }
    ty
}
fn declare(c: &mut Certificate, name: &str, kind: DeclarationKind) {
    let id = c.name_table.len() as u32;
    c.name_table.push(name.into());
    c.declarations.push(Declaration { name: id, kind });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let previous = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c
            .name_table
            .binary_search(&previous[d.name as usize])
            .unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let LevelNode::Param(name) = l {
            *name = c
                .name_table
                .binary_search(&previous[*name as usize])
                .unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}

#[test]
fn actual_full_width_word_circuits_and_universal_basic_laws_check_without_axioms() {
    let f = generate_go_practical_word_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.source_manifest.is_none()
            && c.proof_node_table.is_empty()
            && c.theory_certificates.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for width in [8, 16, 32, 64] {
        for member in [
            "Carrier",
            "Add",
            "Sub",
            "Neg",
            "Mul",
            "SignedDiv",
            "UnsignedDiv",
            "SignedRem",
            "UnsignedRem",
            "ShiftLeft",
            "ShiftRight",
            "SignedShiftRight",
            "DivideDefined",
            "SignedCountDefined",
            "AddZeroAt",
            "AndOnesAt",
            "XorZeroAt",
            "NotNotAt",
            "SignedTo64",
            "UnsignedTo8",
        ] {
            assert!(c.declarations.iter().any(|d| c.name_table[d.name as usize]
                == format!("Mpk.Go.Practical.Word.W{width}.{member}")));
        }
    }
    emit("word-foundation", f.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        f.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_word_import_is_deterministic_and_preserves_complete_range_bytes() {
    let f = generate_go_practical_word_foundation().unwrap();
    assert_eq!(f, generate_go_practical_word_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_word_foundation(f.certificate_bytes()).unwrap()
    );
    let mut changed = f.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_word_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let old = generate_go_practical_range_foundation().unwrap();
    let r = mpk_kernel::verify_certificate_bytes_json_output(old.certificate_bytes());
    assert!(
        r.accepted
            && r.json
                .contains("ddb538b929f4ef0e04d6ac74f119d561e2df132cdba409aa61b0259b99179a70")
    );
    emit("retained-range-basis", old.certificate_bytes(), true);
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.false");
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.Extra",
        DeclarationKind::Def {
            ty,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    );
    let bytes = encode(c);
    emit("honest-different-word-component", &bytes, true);
    assert_eq!(
        import_go_practical_word_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn actual_certificate_word_terms_agree_with_all_frozen_finite_integer_observations() {
    let source = include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let queries: Vec<word_eval::Query> = serde_json::from_slice(source).unwrap();
    assert_eq!(queries.len(), 1640);
    let f = generate_go_practical_word_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    let mut model = vec![];
    let mut oracle = vec![];
    let mut total_steps = 0;
    let mut maximum_steps = 0;
    for q in &queries {
        assert!(ids.insert(q.id.clone()));
        let (actual, steps) = word_eval::observe(&c, q);
        let expected = word_eval::oracle(q);
        assert_eq!(actual, expected, "query {q:?}");
        total_steps += steps;
        maximum_steps = maximum_steps.max(steps);
        model.push(actual);
        oracle.push(expected);
    }
    let counts = |name| model.iter().filter(|r| r.kind == name).count();
    assert_eq!(
        (
            counts("word"),
            counts("boolean"),
            counts("runtime_safety_failure")
        ),
        (1312, 288, 40)
    );
    if let Some(root) = std::env::var_os("MPK_GO_WORD_MODEL_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        let metrics = serde_json::json!({"queries":1640,"word_results":1312,"Boolean_results":288,"runtime_domain_failures":40,"total_AST_steps":total_steps,"maximum_query_AST_steps":maximum_steps,"query_sha256":format!("{:x}",Sha256::digest(source)),"primitive_values":"Bool constructors/recursors and original external input-word bit assignments; never arithmetic operations","source_or_universal_numeric_proof_claimed":false});
        for (name, raw) in [
            ("model.json", serde_json::to_vec(&model).unwrap()),
            ("oracle.json", serde_json::to_vec(&oracle).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            ("certificate.mpcert", f.certificate_bytes().to_vec()),
        ] {
            let mut raw = raw;
            if name != "certificate.mpcert" {
                raw.push(b'\n');
            }
            let mut out = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(name))
                .unwrap();
            out.write_all(&raw).unwrap();
        }
    }
    println!("queries=1640 word=1312 Boolean=288 safety=40 AST_steps={total_steps} max_query_steps={maximum_steps}");
}

#[test]
fn false_carry_add_identity_and_divisor_guard_claims_reject_as_core_type_mismatches() {
    let original = baseline();
    for width in [8, 16, 32, 64] {
        for claim in ["carry", "add-identity", "divisor-guard"] {
            let mut c = original.clone();
            let boolean = constant(&mut c, "Std.Bool");
            let no = constant(&mut c, "Std.Bool.false");
            let yes = constant(&mut c, "Std.Bool.true");
            if claim == "add-identity" {
                let i = global(&c, &format!("Mpk.Go.Practical.Word.W{width}.AddZeroAt")) as usize;
                let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
                    panic!();
                };
                let mut cursor = ty;
                let mut binders = vec![];
                while let TermNode::Pi { ty, body } = c.term_table[cursor as usize] {
                    binders.push(ty);
                    cursor = body;
                }
                let TermNode::App {
                    function,
                    mut arguments,
                } = c.term_table[cursor as usize].clone()
                else {
                    panic!();
                };
                assert_eq!(arguments.len(), 3);
                arguments[2] = no;
                let mut ty = term(
                    &mut c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                );
                for domain in binders.into_iter().rev() {
                    ty = term(
                        &mut c,
                        TermNode::Pi {
                            ty: domain,
                            body: ty,
                        },
                    );
                }
                c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
            } else {
                let zero = wc(&mut c, width, "Zero", vec![]);
                let (left, right) = if claim == "carry" {
                    let x = wc(&mut c, width, "Ones", vec![]);
                    let y = wc(&mut c, width, "One", vec![]);
                    (
                        wc(&mut c, width, &format!("Carry{width}"), vec![x, y, no]),
                        no,
                    )
                } else {
                    (wc(&mut c, width, "DivideDefined", vec![zero]), yes)
                };
                let ty = call(&mut c, "Std.Eq", vec![boolean, left, right]);
                let proof = call(&mut c, "Std.Eq.refl", vec![boolean, right]);
                declare(
                    &mut c,
                    &format!("Mpk.Go.Practical.Test.False{width}{claim}"),
                    DeclarationKind::Theorem { ty, proof },
                );
            }
            let bytes = encode(c);
            emit(&format!("{claim}-{width}"), &bytes, false);
            assert_eq!(
                import_go_practical_word_foundation(&bytes),
                Err(GoOrdinaryError::Linkage)
            );
        }
    }
}

#[test]
fn wrong_typed_shift_counts_and_typed_wrong_adder_definition_reject() {
    let original = baseline();
    for width in [8, 16, 32, 64] {
        let mut c = original.clone();
        let ty = word_type(&mut c, width);
        let zero = wc(&mut c, width, "Zero", vec![]);
        let no = constant(&mut c, "Std.Bool.false");
        let value = wc(&mut c, width, "ShiftLeft", vec![zero, no]);
        declare(
            &mut c,
            &format!("Mpk.Go.Practical.Test.BoolCount{width}"),
            DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            },
        );
        emit(&format!("Boolean-count-{width}"), &encode(c), false);
    }
    let mut c = original;
    let boolean = constant(&mut c, "Std.Bool");
    let a = term(&mut c, TermNode::Var(2));
    let mut value = a;
    for _ in 0..3 {
        value = term(
            &mut c,
            TermNode::Lam {
                ty: boolean,
                body: value,
            },
        );
    }
    let index = global(&c, "Mpk.Go.Practical.Bool.Sum") as usize;
    if let DeclarationKind::Def { value: v, .. } = &mut c.declarations[index].kind {
        *v = value;
    } else {
        panic!();
    }
    emit("wrong-adder-definition", &encode(c), false);
}
