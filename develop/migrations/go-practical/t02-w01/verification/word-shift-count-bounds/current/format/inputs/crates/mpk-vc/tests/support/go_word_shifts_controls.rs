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
        .push("Mpk.Go.Practical.Test.ExtraWordShifts".into());
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

fn false_theorem(label: &str, name: &str, value: &str, domain: Option<usize>) {
    let mut c = baseline();
    let i = global(&c, &format!("Mpk.Go.Practical.Word.Shifts.{name}"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("ordinary shift theorem")
    };
    let ty = if let Some(index) = domain {
        let u = unrelated(&mut c);
        replace_domain(&mut c, ty, index, u)
    } else {
        conclusion(&mut c, ty, value)
    };
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    reject(label, c);
}
fn false_definition(label: &str, name: &str, value: &str) {
    let mut c = baseline();
    let i = global(&c, name);
    let DeclarationKind::Def {
        ty,
        value: body,
        reducibility,
    } = c.declarations[i].kind.clone()
    else {
        panic!("actual shift definition")
    };
    let value = constant_body(&mut c, body, value);
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    reject(label, c);
}
#[test]
fn actual_fixed_right_count_definitions_and_genuine_arithmetic_premises_reject_changes() {
    for (n, boolean) in [
        ("PowerMinusBalance", false),
        ("PowerMinusAdd", false),
        ("PowerPositive", true),
        ("PopConservation6", false),
        ("CubeValueBound6", true),
        ("RightCompose", false),
        ("CountSumZero", true),
        ("CountNonzeroPositive", true),
        ("WeightedBitZero", true),
        ("CountDeMorgan", true),
    ] {
        false_theorem(
            &format!("False{n}"),
            n,
            if boolean {
                if matches!(n, "PowerPositive" | "CubeValueBound6") {
                    "Std.Bool.false"
                } else {
                    "Std.Bool.true"
                }
            } else {
                "Std.Nat.zero"
            },
            None,
        );
    }
    for (n, index) in [
        ("RightCompose", 11),
        ("RightCompose", 12),
        ("WeightedBitZero", 2),
        ("CountPairZero", 4),
        ("CountPairZero", 5),
    ] {
        false_theorem(
            &format!("FakePremise-{n}-{index}"),
            n,
            "Std.Nat.zero",
            Some(index),
        );
    }
    for w in [8usize, 16, 32, 64] {
        for j in 0..w.trailing_zeros() {
            let i = 1 << j;
            for n in [
                "UnsignedRightConservation",
                "SignedRightConservation",
                "RemainderBound",
            ] {
                false_theorem(
                    &format!("False{n}-W{w}-{i}"),
                    &format!("W{w}.{n}{i}"),
                    if n == "RemainderBound" {
                        "Std.Bool.false"
                    } else {
                        "Std.Nat.zero"
                    },
                    None,
                );
            }
        }
        let i = w / 2;
        false_theorem(
            &format!("FalseSignedRightTop-W{w}"),
            &format!("W{w}.SignedRightTop{i}"),
            "Std.Bool.false",
            None,
        );
        false_theorem(
            &format!("FalseFilledRight-W{w}"),
            &format!("W{w}.FilledRightConservation{i}"),
            "Std.Nat.zero",
            None,
        );
        for n in ["Right", "SignedRight"] {
            false_definition(
                &format!("WrongActual{n}-W{w}"),
                &format!("Mpk.Go.Practical.Word.W{w}.{n}{i}"),
                "Std.Bool.false",
            );
        }
    }
    for d in 3..=6 {
        false_theorem(
            &format!("FalseCountBalance-{d}"),
            &format!("CountBalance{d}"),
            "Std.Nat.zero",
            None,
        );
        false_theorem(
            &format!("FalseCountHighZero-{d}"),
            &format!("CountHighZero{d}"),
            "Std.Bool.true",
            None,
        );
        false_definition(
            &format!("WrongCountHigh-{d}"),
            &format!("Mpk.Go.Practical.Word.Shifts.CountHigh{d}"),
            "Std.Bool.false",
        );
        false_definition(
            &format!("WrongCountHighValue-{d}"),
            &format!("Mpk.Go.Practical.Word.Shifts.CountHighValue{d}"),
            "Std.Nat.zero",
        );
    }
}
