use super::inspect;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_equality_foundation;

fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_word_signed_equality_foundation()
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
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
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
            arguments[2] = constant(c, value);
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
fn replace_domain(c: &mut Certificate, id: u32, index: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("binder")
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, index - 1, domain);
        term(c, TermNode::Pi { ty, body })
    }
}
fn claim(label: &str, name: &str, value: &str, premise: Option<usize>) {
    let mut c = baseline();
    let i = global(&c, &format!("Mpk.Go.Practical.Word.SignedEquality.{name}"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("theorem")
    };
    let ty = if let Some(index) = premise {
        let unrelated = unrelated_truth(&mut c);
        replace_domain(&mut c, ty, index, unrelated)
    } else {
        conclusion(&mut c, ty, value)
    };
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    inspect(label, &encode(c), false);
}
fn unrelated_truth(c: &mut Certificate) -> u32 {
    let bt = constant(c, "Std.Bool");
    let yes = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![bt, yes, yes])
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
fn source(width: usize, not_equal: bool) {
    let mut c = baseline();
    let suffix = if not_equal { "NotEqual" } else { "Equal" };
    let i = global(&c, &format!("Mpk.Go.Practical.Word.W{width}.{suffix}"));
    let DeclarationKind::Def {
        ty,
        value,
        reducibility,
    } = c.declarations[i].kind.clone()
    else {
        panic!("source definition")
    };
    let value = constant_body(
        &mut c,
        value,
        if not_equal {
            "Std.Bool.false"
        } else {
            "Std.Bool.true"
        },
    );
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    inspect(
        &format!(
            "Source{}Always{}-{width}",
            suffix,
            if not_equal { "False" } else { "True" }
        ),
        &encode(c),
        false,
    );
}
fn replace_result(c: &mut Certificate, id: u32, result: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = replace_result(c, body, result);
            term(c, TermNode::Pi { ty, body })
        }
        _ => result,
    }
}
fn false_reflect(width: usize) {
    let mut c = baseline();
    let i = global(
        &c,
        &format!("Mpk.Go.Practical.Word.SignedEquality.W{width}.EquivalentReflect"),
    );
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("reflection")
    };
    let nat = constant(&mut c, "Std.Nat");
    let zero = constant(&mut c, "Std.Nat.zero");
    let one = call(&mut c, "Std.Nat.succ", vec![zero]);
    let false_result = call(&mut c, "Std.Eq", vec![nat, zero, one]);
    let ty = replace_result(&mut c, ty, false_result);
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    inspect(
        &format!("FalseEquivalentReflect-{width}"),
        &encode(c),
        false,
    );
}
fn weaken_sign_coherence() {
    let mut c = baseline();
    let i = global(
        &c,
        "Mpk.Go.Practical.Word.SignedEquality.SignedEqualityScalar",
    );
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("scalar")
    };
    let mut cursor = ty;
    for _ in 0..7 {
        let TermNode::Pi { body, .. } = c.term_table[cursor as usize] else {
            panic!("scalar binder")
        };
        cursor = body;
    }
    let TermNode::Pi { ty: callback, .. } = c.term_table[cursor as usize] else {
        panic!("coherence binder")
    };
    let TermNode::Pi { ty: same_value, .. } = c.term_table[callback as usize] else {
        panic!("actual value equality premise")
    };
    let unrelated = unrelated_truth(&mut c);
    let weakened = term(
        &mut c,
        TermNode::Pi {
            ty: same_value,
            body: unrelated,
        },
    );
    let ty = replace_domain(&mut c, ty, 7, weakened);
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    inspect("WeakenSignCoherence", &encode(c), false);
}
pub(super) fn honest_different_component() -> Vec<u8> {
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Test.ExtraSignedEquality".into());
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
pub(super) fn semantic_controls() {
    for width in [8, 16, 32, 64] {
        for (member, value) in [
            ("SourceEqualPreservesSign", "Std.Bool.false"),
            ("ValueEqualPreservesSign", "Std.Bool.false"),
            ("EqualInteger", "Std.Bool.false"),
            ("NotEqualInteger", "Std.Bool.true"),
            ("EquivalentComplete", "Std.Bool.false"),
        ] {
            claim(
                &format!("False{member}-{width}"),
                &format!("W{width}.{member}"),
                value,
                None,
            );
        }
        source(width, false);
        source(width, true);
        claim(
            &format!("FakeValueEqualityPremise-{width}"),
            &format!("W{width}.ValueEqualPreservesSign"),
            "",
            Some(2),
        );
        false_reflect(width);
    }
    for (member, value) in [
        ("SignedEqualityScalar", "Std.Bool.false"),
        ("AndTrueRight", "Std.Bool.false"),
        ("AndFalseRight", "Std.Bool.true"),
        ("RefutationFalse", "Std.Bool.true"),
        ("StrictNotEqualLeft", "Std.Bool.true"),
    ] {
        claim(&format!("False{member}"), member, value, None);
    }
    for (label, premise) in [
        ("FakeABoundSignedEqualityScalar", 3),
        ("FakeBBoundSignedEqualityScalar", 4),
    ] {
        claim(label, "SignedEqualityScalar", "", Some(premise));
    }
    weaken_sign_coherence();
}
