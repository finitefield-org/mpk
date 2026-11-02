use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_division_prerequisites,
    import_go_practical_word_signed_division_prerequisites, GoOrdinaryError,
};

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_signed_division_prerequisites()
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
        import_go_practical_word_signed_division_prerequisites(&bytes),
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
        .push("Mpk.Go.Practical.Test.ExtraSignedDivisionPrerequisites".into());
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

pub(super) fn honest_different_signed_div_source() -> Vec<u8> {
    let mut c = baseline();
    let i = global(&c, "Mpk.Go.Practical.Word.W8.SignedDiv");
    let DeclarationKind::Def {
        ty,
        value,
        reducibility,
    } = c.declarations[i].kind.clone()
    else {
        panic!("source definition")
    };
    let value = constant_body(&mut c, value, "Mpk.Go.Practical.Word.W8.Zero");
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    encode(c)
}
#[test]
fn false_composed_zero_and_half_proof_premises_reject() {
    for (label, name, index, value) in [
        (
            "IgnoredScopedPositive",
            "Zero.PositiveProductZero",
            Some(2),
            "",
        ),
        (
            "FalseScopedHalf",
            "Bounds.HalfDouble6",
            None,
            "Std.Nat.zero",
        ),
    ] {
        let mut c = baseline();
        let i = global(
            &c,
            &format!("Mpk.Go.Practical.Word.SignedDivision.Prerequisite.{name}"),
        );
        let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
            panic!("ordinary law")
        };
        let ty = if let Some(n) = index {
            let domain = unrelated(&mut c);
            replace_domain(&mut c, ty, n, domain)
        } else {
            conclusion(&mut c, ty, value)
        };
        c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
        reject(label, c);
    }
}
#[test]
fn false_actual_unsigned_signed_magnitude_and_minimum_connections_reject() {
    for w in [8, 16, 32, 64] {
        for (label, path, index, value) in [
            (
                "FalseUnsignedConservation",
                "UnsignedDivisionConservation",
                None,
                "Std.Nat.zero",
            ),
            (
                "FakeUnsignedGuard",
                "UnsignedDivisionConservation",
                Some(2),
                "",
            ),
            (
                "FalseRemainderBound",
                "UnsignedRemainderBound",
                None,
                "Std.Bool.false",
            ),
            (
                "FalseHalfBound",
                "MagnitudeHalfBound",
                None,
                "Std.Bool.false",
            ),
            ("FalseScopedZero", "IsZeroNatural", None, "Std.Bool.true"),
            ("FakeScopedDivisor", "DefinedDivisorPositive", Some(1), ""),
            (
                "FalseMagnitudeSign",
                "MagnitudeSignIsMinimum",
                None,
                "Std.Bool.true",
            ),
        ] {
            let role = if label.contains("Unsigned") || label == "FalseRemainderBound" {
                "Unsigned"
            } else if label.contains("Scoped") {
                "Zero"
            } else {
                "Bounds"
            };
            let mut c = baseline();
            let i = global(
                &c,
                &format!("Mpk.Go.Practical.Word.SignedDivision.Prerequisite.{role}.W{w}.{path}"),
            );
            let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
                panic!("ordinary law")
            };
            let ty = if let Some(n) = index {
                let domain = unrelated(&mut c);
                replace_domain(&mut c, ty, n, domain)
            } else {
                conclusion(&mut c, ty, value)
            };
            c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
            reject(&format!("{label}-{w}"), c);
        }
    }
}
