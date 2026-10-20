use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::generate_go_practical_word_shift_foundation;

fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_word_shift_foundation()
                    .unwrap()
                    .certificate_bytes(),
            )
            .unwrap()
        })
        .clone()
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
    emit(name, &bytes, false);
}

pub(super) fn honest_different_component() -> Vec<u8> {
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Test.ExtraFullCountShifts".into());
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
        panic!("binder");
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, index - 1, domain);
        term(c, TermNode::Pi { ty, body })
    }
}
fn unrelated(c: &mut Certificate) -> u32 {
    let bt = constant(c, "Std.Bool");
    let t = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![bt, t, t])
}
fn conclusion(c: &mut Certificate, id: u32, value: &str) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = conclusion(c, body, value);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            arguments[2] = if value == "nat-one" {
                let zero = constant(c, "Std.Nat.zero");
                call(c, "Std.Nat.succ", vec![zero])
            } else {
                constant(c, value)
            };
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("equality"),
    }
}
fn constant_body(c: &mut Certificate, id: u32, value: &str) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = constant_body(c, body, value);
            term(c, TermNode::Lam { ty, body })
        }
        _ => constant(c, value),
    }
}

fn change_theorem(label: &str, name: &str, result: &str, domain: Option<usize>) {
    let mut c = baseline();
    let i = global(&c, &format!("Mpk.Go.Practical.Word.Shifts.{name}"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("source theorem")
    };
    let ty = if let Some(index) = domain {
        let u = unrelated(&mut c);
        replace_domain(&mut c, ty, index, u)
    } else if result == "int-zero" {
        false_relation(&mut c, ty)
    } else {
        conclusion(&mut c, ty, result)
    };
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    reject(label, c);
}
fn false_relation(c: &mut Certificate, id: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = false_relation(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 2);
            let z = constant(c, "Std.Nat.zero");
            arguments[1] = call(c, "Mpk.Go.Practical.Int.FromNat", vec![z]);
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("balanced Int equivalence"),
    }
}
fn change_definition(label: &str, name: &str, value: &str) {
    let mut c = baseline();
    let i = global(&c, name);
    let DeclarationKind::Def {
        ty,
        value: body,
        reducibility,
    } = c.declarations[i].kind.clone()
    else {
        panic!("definition")
    };
    let value = constant_body(&mut c, body, value);
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    reject(label, c);
}
fn erase_count_top(c: &mut Certificate, id: u32, hits: &mut usize) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = erase_count_top(c, body, hits);
            term(c, TermNode::Lam { ty, body })
        }
        TermNode::App {
            function,
            arguments,
        } => {
            let is_count = matches!(c.term_table[function as usize], TermNode::Var(0));
            let is_top=arguments.len()==6&&arguments.iter().all(|i|matches!(c.term_table[*i as usize],TermNode::Const{global,..}if c.name_table[c.declarations[global as usize].name as usize]=="Std.Bool.true"));
            if is_count && is_top {
                *hits += 1;
                return constant(c, "Std.Bool.false");
            }
            let function = erase_count_top(c, function, hits);
            let arguments = arguments
                .into_iter()
                .map(|i| erase_count_top(c, i, hits))
                .collect();
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => id,
    }
}
#[test]
fn new_count_shift_premises_real_source_bits_and_signed_interpretations_reject_changes() {
    for (n, boolean) in [
        ("CountBitComplement", false),
        ("CountPrefixComplementStep", false),
        ("RightStrictGap", false),
        ("RightRemainderComposeBound", true),
        ("LeftCompose", false),
        ("RightSaturationCoefficient", false),
        ("RightSaturatedConservation", false),
        ("LeftSignedBalance", false),
        ("RightSignedBalance", false),
    ] {
        change_theorem(
            &format!("False-{n}"),
            n,
            if boolean {
                "Std.Bool.false"
            } else {
                "Std.Nat.zero"
            },
            None,
        );
    }
    for (n, i) in [
        ("CountPrefixComplementStep", 5),
        ("CountPrefixComplementStep", 6),
        ("CountComplementBound", 3),
        ("RightSaturationCoefficient", 9),
        ("RightSaturatedRemainderBound", 9),
        ("RightSaturatedRemainderBound", 11),
    ] {
        change_theorem(&format!("Fake-{n}-{i}"), n, "Std.Nat.zero", Some(i));
    }
    for d in 3..=6 {
        change_theorem(
            &format!("False-CountBelowWidth-{d}"),
            &format!("CountBelowWidth{d}"),
            "Std.Bool.true",
            None,
        );
        for n in ["CountLargePower", "CountSmallValue"] {
            change_theorem(
                &format!("Fake-{n}-{d}"),
                &format!("{n}{d}"),
                "Std.Nat.zero",
                Some(1),
            );
        }
        change_definition(
            &format!("Wrong-CountExcess-{d}"),
            &format!("Mpk.Go.Practical.Word.Shifts.CountExcess{d}"),
            "Std.Nat.zero",
        );
    }
    for w in [8usize, 16, 32, 64] {
        for (op, lemma, int) in [
            ("ShiftLeft", "UnsignedLeftConservation", false),
            ("ShiftLeft", "SignedLeftRefinement", true),
            ("ShiftRight", "UnsignedRightConservation", false),
            ("SignedShiftRight", "SignedRightFloorConservation", true),
            ("SignedShiftRight", "SignedRightRemainderBound", false),
            ("SignedShiftRight", "SignedRightTop", false),
        ] {
            let result = if int {
                "int-zero"
            } else if lemma.ends_with("Bound") || lemma.ends_with("Top") {
                "Std.Bool.false"
            } else {
                "Std.Nat.zero"
            };
            change_theorem(
                &format!("False-{lemma}-W{w}"),
                &format!("W{w}.{lemma}"),
                result,
                None,
            );
            let _ = op;
        }
        for op in ["ShiftLeft", "ShiftRight", "SignedShiftRight"] {
            change_definition(
                &format!("Wrong-{op}-W{w}"),
                &format!("Mpk.Go.Practical.Word.W{w}.{op}"),
                &format!("Mpk.Go.Practical.Word.W{w}.Zero"),
            );
            let mut c = baseline();
            let i = global(&c, &format!("Mpk.Go.Practical.Word.W{w}.{op}"));
            let DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } = c.declarations[i].kind.clone()
            else {
                panic!("source shift")
            };
            let mut hits = 0;
            let value = erase_count_top(&mut c, value, &mut hits);
            assert_eq!(hits, 1, "actual count bit63");
            c.declarations[i].kind = DeclarationKind::Def {
                ty,
                value,
                reducibility,
            };
            reject(&format!("Missing-CountBit63-{op}-W{w}"), c);
        }
    }
    change_definition(
        "Wrong-UnsignedLeftDiscard-W64",
        "Mpk.Go.Practical.Word.Shifts.W64.LeftDiscard",
        "Std.Nat.zero",
    );
    change_definition(
        "Wrong-SignedRightRemainder-W64",
        "Mpk.Go.Practical.Word.Shifts.W64.SignedRightRemainder",
        "Std.Nat.zero",
    );
    change_definition(
        "Wrong-CountLowComplement6",
        "Mpk.Go.Practical.Word.Shifts.CountLowComplement6",
        "Std.Nat.zero",
    );
    change_theorem("False-SelectTop64", "SelectTop64", "Std.Bool.true", None);
}
