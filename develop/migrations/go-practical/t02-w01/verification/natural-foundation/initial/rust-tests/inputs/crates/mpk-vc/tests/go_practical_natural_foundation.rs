use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_core_foundation, generate_go_practical_natural_foundation,
    import_go_practical_natural_foundation, GoOrdinaryError,
};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let checked = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    assert_eq!(checked.accepted, accepted, "{name}: {}", checked.json);
    if let Some(root) = std::env::var_os("MPK_GO_NATURAL_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, data) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", checked.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&data).unwrap();
        }
    }
}

fn global(c: &Certificate, name: &str) -> u32 {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let index = c.term_table.len() as u32;
    c.term_table.push(node);
    index
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
fn lam(c: &mut Certificate, ty: u32, body: u32) -> u32 {
    term(c, TermNode::Lam { ty, body })
}
fn pi(c: &mut Certificate, ty: u32, body: u32) -> u32 {
    term(c, TermNode::Pi { ty, body })
}
fn natural(c: &mut Certificate, value: u32) -> u32 {
    let mut result = constant(c, "Std.Nat.zero");
    for _ in 0..value {
        result = call(c, "Std.Nat.succ", vec![result]);
    }
    result
}
fn declare(c: &mut Certificate, name: &str, kind: DeclarationKind) {
    let id = c.name_table.len() as u32;
    c.name_table.push(name.into());
    c.declarations.push(Declaration { name: id, kind });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let previous = c.name_table.clone();
    c.name_table.sort();
    for declaration in &mut c.declarations {
        declaration.name = c
            .name_table
            .binary_search(&previous[declaration.name as usize])
            .unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
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
    let foundation = generate_go_practical_natural_foundation().unwrap();
    mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap()
}

#[test]
fn actual_ordinary_unbounded_natural_arithmetic_and_universal_laws_are_checked() {
    let foundation = generate_go_practical_natural_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.source_manifest.is_none()
            && c.proof_node_table.is_empty()
            && c.theory_certificates.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for name in [
        "Pred",
        "Add",
        "Mul",
        "Sub",
        "IsZero",
        "Equal",
        "Less",
        "LessEqual",
        "AddAssoc",
        "AddComm",
        "AddInterchange",
        "MulAssoc",
        "MulComm",
        "MulAddLeft",
        "MulAddRight",
        "AddCancel",
        "EqualReflect",
        "EqualComplete",
        "EqualTranslate",
        "LessTranslate",
        "SubTranslate",
        "DifferenceDecomposition",
        "EmptyLength",
    ] {
        global(&c, &format!("Mpk.Go.Practical.Nat.{name}"));
    }
    emit("natural-foundation", foundation.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        foundation.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_import_reconstructs_all_terms_and_retains_the_core_basis() {
    let a = generate_go_practical_natural_foundation().unwrap();
    assert_eq!(a, generate_go_practical_natural_foundation().unwrap());
    assert_eq!(
        a,
        import_go_practical_natural_foundation(a.certificate_bytes()).unwrap()
    );
    let mut appended = a.certificate_bytes().to_vec();
    appended.push(0);
    assert_eq!(
        import_go_practical_natural_foundation(&appended),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_natural_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let core = generate_go_practical_core_foundation().unwrap();
    let output = mpk_kernel::verify_certificate_bytes_json_output(core.certificate_bytes());
    assert!(output
        .json
        .contains("cf784b36a42929e22d7c2882f41acbcc45e631d4b1cef02af7f564373ec61444"));
    emit("retained-core-basis", core.certificate_bytes(), true);
    let mut c = baseline();
    let nat = constant(&mut c, "Std.Nat");
    let zero = natural(&mut c, 0);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.Extra",
        DeclarationKind::Def {
            ty: nat,
            value: zero,
            reducibility: DefinitionReducibility::Reducible,
        },
    );
    let honest_different = encode(c);
    emit("honest-different-component", &honest_different, true);
    assert_eq!(
        import_go_practical_natural_foundation(&honest_different),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn actual_computations_cover_zeros_unequal_operands_and_truncated_subtraction() {
    let mut c = baseline();
    let nat = constant(&mut c, "Std.Nat");
    let boolean = constant(&mut c, "Std.Bool");
    let cases = [
        ("Pred", vec![0], 0, false),
        ("Pred", vec![7], 6, false),
        ("Add", vec![0, 4], 4, false),
        ("Add", vec![6, 3], 9, false),
        ("Mul", vec![5, 0], 0, false),
        ("Mul", vec![4, 3], 12, false),
        ("Sub", vec![9, 4], 5, false),
        ("Sub", vec![4, 9], 0, false),
        ("Sub", vec![0, 0], 0, false),
        ("IsZero", vec![0], 1, true),
        ("IsZero", vec![8], 0, true),
        ("Equal", vec![7, 7], 1, true),
        ("Equal", vec![0, 3], 0, true),
        ("Equal", vec![3, 0], 0, true),
        ("Less", vec![4, 9], 1, true),
        ("Less", vec![9, 4], 0, true),
        ("Less", vec![5, 5], 0, true),
        ("LessEqual", vec![3, 3], 1, true),
        ("LessEqual", vec![3, 4], 1, true),
        ("LessEqual", vec![4, 3], 0, true),
    ];
    for (i, (op, args, expected, is_bool)) in cases.into_iter().enumerate() {
        let arguments = args.into_iter().map(|a| natural(&mut c, a)).collect();
        let left = call(&mut c, &format!("Mpk.Go.Practical.Nat.{op}"), arguments);
        let ty = if is_bool { boolean } else { nat };
        let right = if is_bool {
            constant(
                &mut c,
                if expected == 1 {
                    "Std.Bool.true"
                } else {
                    "Std.Bool.false"
                },
            )
        } else {
            natural(&mut c, expected)
        };
        let result = call(&mut c, "Std.Eq", vec![ty, left, right]);
        let proof = call(&mut c, "Std.Eq.refl", vec![ty, right]);
        declare(
            &mut c,
            &format!("Mpk.Go.Practical.Test.Computation{i:02}"),
            DeclarationKind::Theorem { ty: result, proof },
        );
    }
    emit("actual-computations", &encode(c), true);
}

#[test]
fn fully_reencoded_wrong_definitions_and_unrelated_proofs_fail_in_the_core() {
    let baseline = baseline();
    for mutation in [
        "addition",
        "multiplication",
        "subtraction",
        "equality",
        "order",
        "cancellation",
        "reflection",
        "difference-decomposition",
    ] {
        let mut c = baseline.clone();
        let nat = constant(&mut c, "Std.Nat");
        let boolean = constant(&mut c, "Std.Bool");
        let zero_var = term(&mut c, TermNode::Var(0));
        let one_var = term(&mut c, TermNode::Var(1));
        let name = match mutation {
            "addition" | "multiplication" | "subtraction" => {
                let value = lam(&mut c, nat, one_var);
                let value = lam(&mut c, nat, value);
                let name = match mutation {
                    "addition" => "Add",
                    "multiplication" => "Mul",
                    _ => "Sub",
                };
                let index = global(&c, &format!("Mpk.Go.Practical.Nat.{name}")) as usize;
                if let DeclarationKind::Def { value: v, .. } = &mut c.declarations[index].kind {
                    *v = value;
                } else {
                    panic!();
                }
                name
            }
            "equality" | "order" => {
                let value = constant(
                    &mut c,
                    if mutation == "equality" {
                        "Std.Bool.true"
                    } else {
                        "Std.Bool.false"
                    },
                );
                let value = lam(&mut c, nat, value);
                let value = lam(&mut c, nat, value);
                let name = if mutation == "equality" {
                    "Equal"
                } else {
                    "Less"
                };
                let index = global(&c, &format!("Mpk.Go.Practical.Nat.{name}")) as usize;
                if let DeclarationKind::Def { value: v, .. } = &mut c.declarations[index].kind {
                    *v = value;
                } else {
                    panic!();
                }
                name
            }
            _ => {
                let name = match mutation {
                    "cancellation" => "AddCancel",
                    "reflection" => "EqualReflect",
                    _ => "DifferenceDecomposition",
                };
                let index = global(&c, &format!("Mpk.Go.Practical.Nat.{name}")) as usize;
                let DeclarationKind::Theorem { ty, .. } = c.declarations[index].kind else {
                    panic!();
                };
                let mut cursor = ty;
                let mut binders = vec![];
                while let TermNode::Pi { ty, body } = c.term_table[cursor as usize] {
                    binders.push(ty);
                    cursor = body;
                }
                let proof = if mutation == "cancellation" {
                    zero_var
                } else {
                    let a = term(&mut c, TermNode::Var(2));
                    call(&mut c, "Std.Eq.refl", vec![nat, a])
                };
                let mut proof = proof;
                for ty in binders.into_iter().rev() {
                    proof = lam(&mut c, ty, proof);
                }
                c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
                name
            }
        };
        // No malformed encoding, stale hashes, new axiom, or resource failure
        // can stand in for an ordinary proof rejection.
        assert!(global(&c, &format!("Mpk.Go.Practical.Nat.{name}")) > 0);
        let _ = boolean;
        let bytes = encode(c);
        assert_eq!(
            import_go_practical_natural_foundation(&bytes),
            Err(GoOrdinaryError::Linkage)
        );
        emit(mutation, &bytes, false);
    }
}

#[test]
fn accepting_a_boolean_value_does_not_prove_its_truth() {
    let mut c = baseline();
    let boolean = constant(&mut c, "Std.Bool");
    let no = constant(&mut c, "Std.Bool.false");
    let yes = constant(&mut c, "Std.Bool.true");
    let a = natural(&mut c, 9);
    let b = natural(&mut c, 4);
    let value = call(&mut c, "Mpk.Go.Practical.Nat.Less", vec![a, b]);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.ClauseValue",
        DeclarationKind::Def {
            ty: boolean,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    );
    let original = encode(c.clone());
    emit("false-clause-value", &original, true);
    let value = constant(&mut c, "Mpk.Go.Practical.Test.ClauseValue");
    let ty = call(&mut c, "Std.Eq", vec![boolean, value, yes]);
    let proof = call(&mut c, "Std.Eq.refl", vec![boolean, no]);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.UnprovedClause",
        DeclarationKind::Theorem { ty, proof },
    );
    emit("false-clause-proof", &encode(c), false);
}
