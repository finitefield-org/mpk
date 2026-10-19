//! Compare named ordinary declarations independently of numeric table indices.
use mpk_cert::encode::{Certificate, DeclarationKind, LevelNode, TermNode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn fingerprint(value: Value) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(&value).unwrap()))
}
pub(super) fn declarations(c: &Certificate) -> BTreeMap<String, Value> {
    let name = |i: u32| c.name_table[c.declarations[i as usize].name as usize].clone();
    let mut levels: Vec<String> = vec![];
    for node in &c.level_table {
        let value = match node {
            LevelNode::Zero => json!(["zero"]),
            LevelNode::Succ(n) => json!(["succ", levels[*n as usize]]),
            LevelNode::Max(a, b) => json!(["max", levels[*a as usize], levels[*b as usize]]),
            LevelNode::Param(n) => json!(["param", c.name_table[*n as usize]]),
        };
        levels.push(fingerprint(value));
    }
    let mut terms: Vec<String> = vec![];
    for node in &c.term_table {
        let value = match node {
            TermNode::Sort(level) => json!(["sort", levels[*level as usize]]),
            TermNode::Var(i) => json!(["var", i]),
            TermNode::Const { global, levels: ls } => json!([
                "const",
                name(*global),
                ls.iter().map(|i| &levels[*i as usize]).collect::<Vec<_>>()
            ]),
            TermNode::App {
                function,
                arguments,
            } => json!([
                "app",
                terms[*function as usize],
                arguments
                    .iter()
                    .map(|i| &terms[*i as usize])
                    .collect::<Vec<_>>()
            ]),
            TermNode::Lam { ty, body } => {
                json!(["lam", terms[*ty as usize], terms[*body as usize]])
            }
            TermNode::Pi { ty, body } => json!(["pi", terms[*ty as usize], terms[*body as usize]]),
            TermNode::Let { ty, value, body } => json!([
                "let",
                terms[*ty as usize],
                terms[*value as usize],
                terms[*body as usize]
            ]),
        };
        terms.push(fingerprint(value));
    }
    c.declarations
        .iter()
        .map(|d| {
            let value = match &d.kind {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => json!([
                    "definition",
                    terms[*ty as usize],
                    terms[*value as usize],
                    format!("{reducibility:?}")
                ]),
                DeclarationKind::Theorem { ty, proof } => {
                    json!(["theorem", terms[*ty as usize], terms[*proof as usize]])
                }
                DeclarationKind::Inductive { ty } => json!(["inductive", terms[*ty as usize]]),
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => json!([
                    "constructor",
                    terms[*ty as usize],
                    name(*inductive),
                    generated
                ]),
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => json!(["recursor", terms[*ty as usize], name(*inductive), generated]),
                _ => panic!("ordinary zero-axiom declaration"),
            };
            (c.name_table[d.name as usize].clone(), value)
        })
        .collect()
}
