use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_numeric_foundation, import_go_practical_word_numeric_foundation,
    GoOrdinaryError,
};

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_numeric_foundation()
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
        import_go_practical_word_numeric_foundation(&bytes),
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
        .push("Mpk.Go.Practical.Test.ExtraNumeric".into());
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

#[test]
fn wrong_bit_weight_and_value_definitions_cannot_satisfy_retained_laws() {
    for suffix in ["Bit", "Weight0", "Value0"] {
        let mut c = baseline();
        let index = global(&c, &format!("Mpk.Go.Practical.Word.Numeric.{suffix}"));
        let zero = constant(&mut c, "Std.Nat.zero");
        let value = if suffix == "Weight0" {
            call(&mut c, "Std.Nat.succ", vec![zero])
        } else {
            let ty = constant(&mut c, "Std.Bool");
            term(&mut c, TermNode::Lam { ty, body: zero })
        };
        let DeclarationKind::Def {
            ty, reducibility, ..
        } = c.declarations[index].kind.clone()
        else {
            panic!("ordinary definition");
        };
        c.declarations[index].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        reject(&format!("wrong-{suffix}"), c);
    }
}

fn false_proof(c: &mut Certificate, ty: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = false_proof(c, body);
            term(c, TermNode::Lam { ty, body })
        }
        _ => {
            let boolean = constant(c, "Std.Bool");
            let false_ = constant(c, "Std.Bool.false");
            call(c, "Std.Eq.refl", vec![boolean, false_])
        }
    }
}
fn change_last_domain(c: &mut Certificate, id: u32, replacement: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("dependent theorem binders");
    };
    if matches!(c.term_table[body as usize], TermNode::Pi { .. }) {
        let body = change_last_domain(c, body, replacement);
        term(c, TermNode::Pi { ty, body })
    } else {
        term(
            c,
            TermNode::Pi {
                ty: replacement,
                body,
            },
        )
    }
}
fn drop_final_carry(c: &mut Certificate, id: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = drop_final_carry(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            let TermNode::App { arguments: sum, .. } = c.term_table[arguments[2] as usize].clone()
            else {
                panic!("result plus final carry");
            };
            assert_eq!(sum.len(), 2);
            arguments[2] = sum[0];
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("ordinary equality conclusion"),
    }
}

#[test]
fn false_actual_bits_ignored_overflow_and_dropped_carry_claims_reject() {
    for width in [8_usize, 16, 32, 64] {
        for mutation in [
            "CarryStep0",
            "SumBit0",
            "SignedCrossSum",
            "SignedAddition",
            "DroppedCarry",
        ] {
            let mut c = baseline();
            let suffix = if mutation == "DroppedCarry" {
                format!("Block{}At0", width.trailing_zeros())
            } else {
                mutation.into()
            };
            let index = global(
                &c,
                &format!("Mpk.Go.Practical.Word.Numeric.W{width}.{suffix}"),
            );
            let DeclarationKind::Theorem { mut ty, mut proof } = c.declarations[index].kind.clone()
            else {
                panic!("ordinary theorem");
            };
            match mutation {
                "CarryStep0" | "SumBit0" => proof = false_proof(&mut c, ty),
                "SignedCrossSum" | "SignedAddition" => {
                    let boolean = constant(&mut c, "Std.Bool");
                    let true_ = constant(&mut c, "Std.Bool.true");
                    let unrelated = call(&mut c, "Std.Eq", vec![boolean, true_, true_]);
                    ty = change_last_domain(&mut c, ty, unrelated);
                }
                "DroppedCarry" => ty = drop_final_carry(&mut c, ty),
                _ => unreachable!(),
            }
            c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
            reject(&format!("{mutation}-{width}"), c);
        }
    }
}
