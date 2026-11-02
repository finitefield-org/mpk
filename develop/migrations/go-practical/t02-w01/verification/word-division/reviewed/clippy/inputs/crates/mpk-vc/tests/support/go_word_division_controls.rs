use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_division_foundation, import_go_practical_word_division_foundation,
    GoOrdinaryError,
};

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_division_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
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
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for declaration in &mut c.declarations {
        declaration.name = c
            .name_table
            .binary_search(&old[declaration.name as usize])
            .unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn reject(name: &str, c: Certificate) {
    let bytes = encode(c);
    assert_eq!(
        import_go_practical_word_division_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    emit(name, &bytes, false);
}

pub(super) fn honest_different_component() -> Vec<u8> {
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Test.ExtraDivision".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    });
    encode(c)
}

fn replace_domain(c: &mut Certificate, id: u32, index: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("theorem binder");
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, index - 1, domain);
        term(c, TermNode::Pi { ty, body })
    }
}
fn conclusion(c: &mut Certificate, id: u32, boolean: bool) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = conclusion(c, body, boolean);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            arguments[2] = constant(
                c,
                if boolean {
                    "Std.Bool.false"
                } else {
                    "Std.Nat.zero"
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
        _ => panic!("ordinary equality"),
    }
}
fn wrong_body(c: &mut Certificate, id: u32, overflow: bool) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = wrong_body(c, body, overflow);
            term(c, TermNode::Lam { ty, body })
        }
        TermNode::App {
            function,
            arguments,
        } if overflow => {
            assert_eq!(arguments.len(), 2);
            let TermNode::Const { global, .. } = c.term_table[function as usize] else {
                panic!("Or");
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                "Mpk.Go.Practical.Bool.Or"
            );
            arguments[1]
        }
        _ => {
            assert!(!overflow);
            constant(c, "Std.Nat.zero")
        }
    }
}
fn unrelated(c: &mut Certificate) -> u32 {
    let ty = constant(c, "Std.Bool");
    let t = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![ty, t, t])
}

#[test]
fn false_unbounded_gaps_bounds_horner_and_restoring_stage_laws_reject() {
    for (label, name, index) in [
        ("FalseGap", "StrictDecomposition", None),
        ("IgnoredDoubleBound", "DoubleBitBound", Some(3)),
        ("IgnoredRemainderBound", "RestoringConservation", Some(8)),
        ("IgnoredModulusBound", "RestoringBound", Some(9)),
        ("WrongHornerCombine", "HornerCombine", None),
        ("WrongStageCombine", "StageCombine", None),
    ] {
        let mut c = baseline();
        let member = global(&c, &format!("Mpk.Go.Practical.Word.Division.{name}"));
        let DeclarationKind::Theorem { ty, proof } = c.declarations[member].kind else {
            panic!("ordinary law");
        };
        let ty = if let Some(index) = index {
            let d = unrelated(&mut c);
            replace_domain(&mut c, ty, index, d)
        } else {
            conclusion(&mut c, ty, false)
        };
        c.declarations[member].kind = DeclarationKind::Theorem { ty, proof };
        reject(label, c);
    }
}

#[test]
fn false_actual_decisions_prefixes_remainder_bounds_and_fake_guards_reject() {
    for width in [8, 16, 32, 64] {
        for (label, suffix) in [
            ("MissingOverflow", "Decision"),
            ("WrongFirstPrefix", "Prefix1"),
            ("FalseRemainderBound", "UnsignedRemainderBound"),
            ("FakeDivisorGuard", "UnsignedDivisionConservation"),
        ] {
            let mut c = baseline();
            let member = global(
                &c,
                &format!("Mpk.Go.Practical.Word.Division.W{width}.{suffix}"),
            );
            c.declarations[member].kind = match c.declarations[member].kind.clone() {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => {
                    let value = wrong_body(&mut c, value, label == "MissingOverflow");
                    DeclarationKind::Def {
                        ty,
                        value,
                        reducibility,
                    }
                }
                DeclarationKind::Theorem { ty, proof } => {
                    let ty = if label == "FakeDivisorGuard" {
                        let d = unrelated(&mut c);
                        replace_domain(&mut c, ty, 2, d)
                    } else {
                        conclusion(&mut c, ty, true)
                    };
                    DeclarationKind::Theorem { ty, proof }
                }
                _ => panic!("ordinary member"),
            };
            reject(&format!("{label}-{width}"), c);
        }
    }
}
