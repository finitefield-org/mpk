use super::inspect;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_order_foundation;

fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_word_signed_order_foundation()
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
    let i = global(&c, &format!("Mpk.Go.Practical.Word.SignedOrder.{name}"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
        panic!("theorem")
    };
    let ty = if let Some(index) = premise {
        let bt = constant(&mut c, "Std.Bool");
        let yes = constant(&mut c, "Std.Bool.true");
        let unrelated = call(&mut c, "Std.Eq", vec![bt, yes, yes]);
        replace_domain(&mut c, ty, index, unrelated)
    } else {
        conclusion(&mut c, ty, value)
    };
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    inspect(label, &encode(c), false);
}
fn source(label: &str, width: usize, equal: bool) {
    let mut c = baseline();
    let suffix = if equal { "LessEqual" } else { "Less" };
    let i = global(
        &c,
        &format!("Mpk.Go.Practical.Word.W{width}.Signed{suffix}"),
    );
    let DeclarationKind::Def {
        ty, reducibility, ..
    } = c.declarations[i].kind.clone()
    else {
        panic!("source definition")
    };
    let value = constant(
        &mut c,
        &format!("Mpk.Go.Practical.Word.W{width}.Unsigned{suffix}"),
    );
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    inspect(label, &encode(c), false);
}
fn wrong_top(width: usize) {
    fn walk(
        c: &mut Certificate,
        id: u32,
        depth: usize,
        no: u32,
        hits: &mut usize,
        memo: &mut std::collections::BTreeMap<u32, u32>,
    ) -> u32 {
        if let Some(id) = memo.get(&id) {
            return *id;
        }
        let old = c.term_table[id as usize].clone();
        let node = match old {
            TermNode::App {
                function,
                arguments,
            } => {
                let top = matches!(c.term_table[function as usize], TermNode::Var(0 | 1)) && arguments.len() == depth && arguments.iter().all(|i| matches!(&c.term_table[*i as usize], TermNode::Const { global:g, .. } if c.name_table[c.declarations[*g as usize].name as usize] == "Std.Bool.true"));
                if top {
                    *hits += 1;
                    TermNode::App {
                        function,
                        arguments: vec![no; depth],
                    }
                } else {
                    let function = walk(c, function, depth, no, hits, memo);
                    let arguments = arguments
                        .into_iter()
                        .map(|a| walk(c, a, depth, no, hits, memo))
                        .collect();
                    TermNode::App {
                        function,
                        arguments,
                    }
                }
            }
            TermNode::Lam { ty, body } => {
                let ty = walk(c, ty, depth, no, hits, memo);
                let body = walk(c, body, depth, no, hits, memo);
                TermNode::Lam { ty, body }
            }
            other => other,
        };
        let new = term(c, node);
        memo.insert(id, new);
        new
    }
    let mut c = baseline();
    let i = global(&c, &format!("Mpk.Go.Practical.Word.W{width}.SignedLess"));
    let DeclarationKind::Def {
        ty,
        value,
        reducibility,
    } = c.declarations[i].kind.clone()
    else {
        panic!("source comparison")
    };
    let no = constant(&mut c, "Std.Bool.false");
    let mut hits = 0;
    let value = walk(
        &mut c,
        value,
        width.trailing_zeros() as usize,
        no,
        &mut hits,
        &mut std::collections::BTreeMap::new(),
    );
    assert_eq!(hits, 2, "both actual source sign addresses");
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    inspect(&format!("WrongTopAddress-{width}"), &encode(c), false);
}
pub(super) fn honest_different_component() -> Vec<u8> {
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Test.ExtraSignedOrder".into());
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
    for width in [8_usize, 16, 32, 64] {
        claim(
            &format!("FalseSignedLess-{width}"),
            &format!("W{width}.SignedLessInteger"),
            "Std.Bool.false",
            None,
        );
        claim(
            &format!("FalseSignedLessEqual-{width}"),
            &format!("W{width}.SignedLessEqualInteger"),
            "Std.Bool.true",
            None,
        );
        source(&format!("UnsignedForSignedLess-{width}"), width, false);
        source(&format!("UnsignedForSignedLessEqual-{width}"), width, true);
        wrong_top(width);
    }
    for (name, value) in [
        ("BelowExtensionBalance", "Std.Nat.zero"),
        ("BelowExtendedRight", "Std.Bool.false"),
        ("ExtendedRightNotBelow", "Std.Bool.true"),
        ("SignedLessScalar", "Std.Bool.false"),
    ] {
        claim(&format!("False{name}"), name, value, None);
    }
    for (label, name, premise) in [
        ("FakeBoundBelowExtensionBalance", "BelowExtensionBalance", 3),
        ("FakeABoundSignedLessScalar", "SignedLessScalar", 3),
        ("FakeBBoundSignedLessScalar", "SignedLessScalar", 4),
    ] {
        claim(label, name, "", Some(premise));
    }
}
