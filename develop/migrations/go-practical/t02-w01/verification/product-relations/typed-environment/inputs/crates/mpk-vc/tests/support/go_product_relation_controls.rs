use super::inspect;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_product_relation_foundation,
    generate_go_practical_word_product_equality_foundation,
};

fn baseline(words: bool) -> Certificate {
    static RELATIONS: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    static WORDS: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    let cached = if words { &WORDS } else { &RELATIONS };
    cached
        .get_or_init(|| {
            let bytes = if words {
                generate_go_practical_word_product_equality_foundation()
                    .unwrap()
                    .certificate_bytes()
                    .to_vec()
            } else {
                generate_go_practical_product_relation_foundation()
                    .unwrap()
                    .certificate_bytes()
                    .to_vec()
            };
            mpk_cert::decode_canonical_certificate(&bytes).unwrap()
        })
        .clone()
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn term(c: &mut Certificate, t: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(t);
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
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131_072);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn boolean_claim(c: &mut Certificate, left: bool) -> u32 {
    let ty = constant(c, "Std.Bool");
    let left = constant(
        c,
        if left {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    );
    let right = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![ty, left, right])
}
fn change_result(c: &mut Certificate, id: u32, changed: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = change_result(c, body, changed);
            term(c, TermNode::Pi { ty, body })
        }
        _ => changed,
    }
}
fn replace_domain(c: &mut Certificate, id: u32, index: usize, changed: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("binder")
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: changed, body })
    } else {
        let body = replace_domain(c, body, index - 1, changed);
        term(c, TermNode::Pi { ty, body })
    }
}
fn altered_claim(label: &str, name: &str, premise: Option<usize>, words: bool) {
    let mut c = baseline(words);
    let index = global(&c, name);
    let DeclarationKind::Theorem { ty, proof } = c.declarations[index].kind.clone() else {
        panic!("theorem")
    };
    let altered = boolean_claim(&mut c, premise.is_some());
    let ty = if let Some(i) = premise {
        replace_domain(&mut c, ty, i, altered)
    } else {
        change_result(&mut c, ty, altered)
    };
    c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
    inspect(label, &encode(c), false);
}
fn replace_body(
    c: &mut Certificate,
    id: u32,
    replacement: &mut impl FnMut(&mut Certificate) -> u32,
) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = replace_body(c, body, replacement);
            term(c, TermNode::Lam { ty, body })
        }
        _ => replacement(c),
    }
}
fn changed_source(label: &str, member: &str, mut replacement: impl FnMut(&mut Certificate) -> u32) {
    let mut c = baseline(false);
    let index = global(&c, &format!("Mpk.Go.Practical.Product.Relation.{member}"));
    c.declarations[index].kind = match c.declarations[index].kind.clone() {
        DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } => DeclarationKind::Def {
            ty,
            value: replace_body(&mut c, value, &mut replacement),
            reducibility,
        },
        DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
            ty,
            proof: replace_body(&mut c, proof, &mut replacement),
        },
        _ => panic!("ordinary member"),
    };
    inspect(label, &encode(c), false);
}
pub(super) fn honest_different(words: bool) -> Vec<u8> {
    let mut c = baseline(words);
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    c.name_table
        .push("Mpk.Go.Consumer.HonestDifferentProductRelation".into());
    c.declarations.push(Declaration {
        name: (c.name_table.len() - 1) as u32,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    });
    encode(c)
}
pub(super) fn semantic_controls() {
    for member in [
        "RelatedMake",
        "RelatedLeft",
        "RelatedRight",
        "Reflexive",
        "Symmetric",
        "Transitive",
        "SetLeftRelated",
        "SetRightRelated",
        "MapRelated",
        "AndTrueLeft",
        "AndTrueRight",
        "AndTrueIntro",
        "EqualByReflect",
        "EqualByComplete",
    ] {
        altered_claim(
            &format!("false-{member}"),
            &format!("Mpk.Go.Practical.Product.Relation.{member}"),
            None,
            false,
        );
    }
    for (member, indices) in [
        ("RelatedMake", vec![6, 7]),
        ("RelatedLeft", vec![6]),
        ("RelatedRight", vec![6]),
        ("Reflexive", vec![5, 6]),
        ("Symmetric", vec![6, 7, 8]),
        ("Transitive", vec![6, 8, 9, 10]),
        ("SetLeftRelated", vec![6, 9]),
        ("SetRightRelated", vec![6, 9]),
        ("MapRelated", vec![6, 13, 14]),
        ("AndTrueLeft", vec![2]),
        ("AndTrueRight", vec![2]),
        ("AndTrueIntro", vec![2, 3]),
        ("EqualByReflect", vec![8, 9, 10]),
        ("EqualByComplete", vec![8, 9, 10]),
    ] {
        for index in indices {
            altered_claim(
                &format!("erased-{member}-{index}"),
                &format!("Mpk.Go.Practical.Product.Relation.{member}"),
                Some(index),
                false,
            );
        }
    }
    changed_source("drop-right-comparator", "EqualBy", |c| {
        // EqualBy binds A B p q cmpA cmpB. Return the actual left comparator.
        let a = term(c, TermNode::Var(5));
        let b = term(c, TermNode::Var(4));
        let p = term(c, TermNode::Var(3));
        let q = term(c, TermNode::Var(2));
        let cmp = term(c, TermNode::Var(1));
        let left_p = call(c, "Mpk.Go.Practical.Product.Left", vec![a, b, p]);
        let left_q = call(c, "Mpk.Go.Practical.Product.Left", vec![a, b, q]);
        term(
            c,
            TermNode::App {
                function: cmp,
                arguments: vec![left_p, left_q],
            },
        )
    });
    changed_source("related-requires-right-witness", "Related", |c| {
        constant(c, "Std.Bool")
    });
    changed_source("wrong-related-right-projection", "RelatedRight", |c| {
        let args = (0..7).rev().map(|i| term(c, TermNode::Var(i))).collect();
        call(c, "Mpk.Go.Practical.Product.Relation.RelatedLeft", args)
    });
    for left in [8, 16, 32, 64] {
        for right in [8, 16, 32, 64] {
            for member in ["EquivalentReflect", "EquivalentComplete"] {
                altered_claim(
                    &format!("false-W{left}W{right}-{member}"),
                    &format!("Mpk.Go.Practical.Word.ProductEquality.W{left}W{right}.{member}"),
                    None,
                    true,
                );
            }
        }
    }
    altered_claim(
        "false-integer-pair-representation-shift",
        "Mpk.Go.Practical.Int.ProductEquality.RepresentationShift",
        None,
        true,
    );
}
