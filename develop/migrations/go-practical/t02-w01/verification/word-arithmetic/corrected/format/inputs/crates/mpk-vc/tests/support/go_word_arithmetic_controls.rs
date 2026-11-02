use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_arithmetic_foundation,
    import_go_practical_word_arithmetic_foundation, GoOrdinaryError,
};

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_arithmetic_foundation()
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
        import_go_practical_word_arithmetic_foundation(&bytes),
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
        .push("Mpk.Go.Practical.Test.ExtraArithmetic".into());
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

fn replace_domain(c: &mut Certificate, id: u32, depth: usize, replacement: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("theorem binder");
    };
    if depth == 0 {
        term(
            c,
            TermNode::Pi {
                ty: replacement,
                body,
            },
        )
    } else {
        let body = replace_domain(c, body, depth - 1, replacement);
        term(c, TermNode::Pi { ty, body })
    }
}
fn drop_addend(c: &mut Certificate, id: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = drop_addend(c, body);
            term(c, TermNode::Lam { ty, body })
        }
        TermNode::App {
            function,
            arguments,
        } => {
            assert_eq!(arguments.len(), 2);
            let TermNode::Const { global, .. } = c.term_table[function as usize] else {
                panic!("Nat addition");
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                "Mpk.Go.Practical.Nat.Add"
            );
            arguments[0]
        }
        _ => panic!("computed discarded-value definition"),
    }
}
fn zero_body(c: &mut Certificate, id: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = zero_body(c, body);
            term(c, TermNode::Lam { ty, body })
        }
        _ => constant(c, "Std.Nat.zero"),
    }
}
fn drop_conclusion_contribution(c: &mut Certificate, id: u32, side: usize) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = drop_conclusion_contribution(c, body, side);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            let TermNode::App { arguments: sum, .. } =
                c.term_table[arguments[side] as usize].clone()
            else {
                panic!("value plus modulus contribution");
            };
            assert_eq!(sum.len(), 2);
            arguments[side] = sum[0];
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("ordinary Nat conclusion"),
    }
}

#[test]
fn wrong_unbounded_power_push_and_ignored_shift_premise_reject() {
    for mutation in ["PowerBase", "PushLeaf", "IgnoredShiftPremise"] {
        let mut c = baseline();
        let suffix = match mutation {
            "PowerBase" => "PowerTwo",
            "PushLeaf" => "Push0",
            _ => "ProductCombine",
        };
        let index = global(&c, &format!("Mpk.Go.Practical.Word.Numeric.{suffix}"));
        match c.declarations[index].kind.clone() {
            DeclarationKind::Def {
                ty, reducibility, ..
            } => {
                let value = if mutation == "PowerBase" {
                    let nt = constant(&mut c, "Std.Nat");
                    let body = constant(&mut c, "Std.Nat.zero");
                    term(&mut c, TermNode::Lam { ty: nt, body })
                } else {
                    let bt = constant(&mut c, "Std.Bool");
                    let body = term(&mut c, TermNode::Var(1));
                    let body = term(&mut c, TermNode::Lam { ty: bt, body });
                    term(&mut c, TermNode::Lam { ty: bt, body })
                };
                c.declarations[index].kind = DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                };
            }
            DeclarationKind::Theorem { ty, proof } => {
                let nt = constant(&mut c, "Std.Nat");
                let zero = constant(&mut c, "Std.Nat.zero");
                let unrelated = call(&mut c, "Std.Eq", vec![nt, zero, zero]);
                let ty = replace_domain(&mut c, ty, 13, unrelated);
                c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
            }
            _ => panic!("ordinary compiled member"),
        }
        reject(mutation, c);
    }
}

#[test]
fn false_shift_product_correspondence_and_discarded_value_claims_reject() {
    let focused = std::env::var("MPK_GO_WORD_ARITHMETIC_CONTROL_SELECTION").ok();
    assert!(focused.is_none() || focused.as_deref() == Some("zero-witness-and-final-shift"));
    for width in [8_usize, 16, 32, 64] {
        for mutation in [
            "DroppedShift",
            "DroppedProduct",
            "WrongLeft",
            "WrongMulDiscard",
            "WrongShiftDiscard",
        ] {
            if focused.is_some()
                && mutation != "WrongMulDiscard"
                && !(width == 64 && mutation == "WrongShiftDiscard")
            {
                continue;
            }
            let mut c = baseline();
            let name = match mutation {
                "DroppedShift" => format!(
                    "Mpk.Go.Practical.Word.Numeric.W{width}.ShiftConservation{}",
                    width - 1
                ),
                "DroppedProduct" => {
                    format!("Mpk.Go.Practical.Word.Numeric.W{width}.MulValueConservation")
                }
                "WrongLeft" => format!("Mpk.Go.Practical.Word.W{width}.Left1"),
                "WrongMulDiscard" => {
                    format!("Mpk.Go.Practical.Word.Numeric.W{width}.MulDiscard{width}")
                }
                _ => format!(
                    "Mpk.Go.Practical.Word.Numeric.W{width}.ShiftDiscard{}",
                    width - 1
                ),
            };
            let index = global(&c, &name);
            match c.declarations[index].kind.clone() {
                DeclarationKind::Theorem { ty, proof } => {
                    let ty = drop_conclusion_contribution(
                        &mut c,
                        ty,
                        if mutation == "DroppedShift" { 1 } else { 2 },
                    );
                    c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
                }
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => {
                    let value = if mutation == "WrongLeft" {
                        let TermNode::Lam { ty, .. } = c.term_table[value as usize] else {
                            panic!("word binder");
                        };
                        let x = term(&mut c, TermNode::Var(0));
                        let body = call(
                            &mut c,
                            &format!("Mpk.Go.Practical.Word.W{width}.Left0"),
                            vec![x],
                        );
                        term(&mut c, TermNode::Lam { ty, body })
                    } else if mutation == "WrongMulDiscard" {
                        zero_body(&mut c, value)
                    } else {
                        drop_addend(&mut c, value)
                    };
                    c.declarations[index].kind = DeclarationKind::Def {
                        ty,
                        value,
                        reducibility,
                    };
                }
                _ => panic!("ordinary member"),
            }
            reject(&format!("{mutation}-{width}"), c);
        }
    }
}
