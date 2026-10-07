include!("builder.rs");
use mpk_cert::encode::DefinitionReducibility;
use std::{collections::BTreeSet, fs, path::Path};

fn compact_terms(c: &mut Certificate) {
    let mut pending = Vec::new();
    for declaration in &c.declarations {
        match declaration.kind {
            DeclarationKind::Def { ty, value, .. } => pending.extend([ty, value]),
            DeclarationKind::Theorem { ty, proof } => pending.extend([ty, proof]),
            DeclarationKind::Axiom { ty }
            | DeclarationKind::Inductive { ty }
            | DeclarationKind::Constructor { ty, .. }
            | DeclarationKind::Recursor { ty, .. } => pending.push(ty),
            _ => panic!("unexpected declaration"),
        }
    }
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) { continue; }
        match &c.term_table[id as usize] {
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => pending.extend([*ty, *body]),
            TermNode::App { function, arguments } => {
                pending.push(*function); pending.extend(arguments);
            }
            TermNode::Let { ty, value, body } => pending.extend([*ty, *value, *body]),
            _ => {},
        }
    }
    let mapping = seen.iter().enumerate().map(|(new, old)| (*old, new as u32)).collect::<BTreeMap<_, _>>();
    c.term_table = seen.into_iter().map(|id| {
        let mut term = c.term_table[id as usize].clone();
        match &mut term {
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => { *ty = mapping[ty]; *body = mapping[body]; }
            TermNode::App { function, arguments } => { *function = mapping[function]; for id in arguments { *id = mapping[id]; } }
            TermNode::Let { ty, value, body } => { *ty = mapping[ty]; *value = mapping[value]; *body = mapping[body]; }
            _ => {},
        }
        term
    }).collect();
    for declaration in &mut c.declarations {
        match &mut declaration.kind {
            DeclarationKind::Def { ty, value, .. } => { *ty = mapping[ty]; *value = mapping[value]; }
            DeclarationKind::Theorem { ty, proof } => { *ty = mapping[ty]; *proof = mapping[proof]; }
            DeclarationKind::Axiom { ty }
            | DeclarationKind::Inductive { ty }
            | DeclarationKind::Constructor { ty, .. }
            | DeclarationKind::Recursor { ty, .. } => *ty = mapping[ty],
            _ => panic!("unexpected declaration"),
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let text = fs::read_to_string(&args[1]).unwrap();
    let original = mpk_cert::decode_canonical_certificate(&bytes(text.trim())).unwrap();
    let out = Path::new(&args[2]); fs::create_dir_all(out).unwrap();
    let names = [
        "prior-constructor-value-levels", "prior-opaque-value-levels", "prior-family-type-levels",
        "prior-theorem-proof-levels", "prior-let-value-levels", "prior-lambda-body-levels",
        "prior-pi-domain-levels", "later-constructor-value-levels",
        "legacy-prior-constructor-value-levels", "legacy-prior-theorem-proof-levels",
        "unused-constructor-levels", "prior-other-family-levels",
    ];
    for label in names {
        let mut b = Builder::from(original.clone());
        let cases = b.globals["Std.Bool.cases"] as usize;
        let legacy = label.starts_with("legacy-");
        if legacy { b.c.declarations.truncate(cases); compact_terms(&mut b.c); b = Builder::from(b.c); }
        let insert = if label.starts_with("later-") { b.c.declarations.len() } else { cases };
        let unused = label.starts_with("unused-");
        let other = label == "prior-other-family-levels";
        if !legacy && !unused {
            let shift = if other { 2 } else { 1 };
            for term in &mut b.c.term_table {
                if let TermNode::Const { global, .. } = term { if *global >= insert as u32 { *global += shift; } }
            }
            for declaration in &mut b.c.declarations {
                match &mut declaration.kind {
                    DeclarationKind::Constructor { inductive, .. } | DeclarationKind::Recursor { inductive, .. } => {
                        if *inductive >= insert as u32 { *inductive += shift; }
                    }
                    _ => {},
                }
            }
        }
        let false_global = b.globals["Std.Bool.false"];
        let family = b.globals["Std.Bool"];
        let bad_no = b.term(TermNode::Const { global: false_global, levels: vec![0] }).unwrap();
        let bad_boolean = b.term(TermNode::Const { global: family, levels: vec![0] }).unwrap();
        let ty = b.boolean;
        let name = b.c.name_table.len() as u32;
        b.c.name_table.push(format!("Probe.Bool.{}", label.replace('-', "_")));
        let kind = match label {
            "prior-family-type-levels" => {
                let ty = b.pi(bad_boolean, bad_boolean).unwrap();
                let body = b.var(0).unwrap();
                let value = b.lam(bad_boolean, body).unwrap();
                DeclarationKind::Theorem { ty, proof: value }
            },
            "prior-theorem-proof-levels" | "legacy-prior-theorem-proof-levels" => DeclarationKind::Theorem { ty, proof: bad_no },
            "prior-opaque-value-levels" => DeclarationKind::Def { ty, value: bad_no, reducibility: DefinitionReducibility::Opaque },
            "prior-let-value-levels" => {
                let body = b.var(0).unwrap();
                let value = b.term(TermNode::Let { ty, value: bad_no, body }).unwrap();
                DeclarationKind::Def { ty, value, reducibility: DefinitionReducibility::Opaque }
            }
            "prior-lambda-body-levels" => {
                let ty = b.pi(ty, ty).unwrap();
                let function = b.lam(b.boolean, bad_no).unwrap();
                let no = b.constant("Std.Bool.false").unwrap();
                let body = b.app(function, vec![no]).unwrap();
                let value = b.lam(b.boolean, body).unwrap();
                DeclarationKind::Theorem { ty, proof: value }
            }
            "prior-pi-domain-levels" => {
                let ty = b.pi(bad_boolean, b.boolean).unwrap();
                let body = b.constant("Std.Bool.false").unwrap();
                let value = b.lam(bad_boolean, body).unwrap();
                DeclarationKind::Def { ty, value, reducibility: DefinitionReducibility::Reducible }
            }
            _ => DeclarationKind::Def { ty, value: bad_no, reducibility: DefinitionReducibility::Reducible },
        };
        if other {
            let other_name = b.c.name_table.len() as u32; b.c.name_table.push("Probe.Other".into());
            let other_ty = b.term(TermNode::Const { global: insert as u32, levels: vec![0] }).unwrap();
            let body = b.var(0).unwrap();
            let ty = b.pi(other_ty, other_ty).unwrap();
            let value = b.lam(other_ty, body).unwrap();
            b.c.declarations.insert(insert, Declaration { name: other_name, kind: DeclarationKind::Inductive { ty: b.sort } });
            b.c.declarations.insert(insert + 1, Declaration { name, kind: DeclarationKind::Def { ty, value, reducibility: DefinitionReducibility::Reducible } });
        } else if !unused {
            b.c.declarations.insert(insert, Declaration { name, kind });
        }
        let data = Builder::from(b.c).finish();
        fs::write(out.join(format!("{label}.mpcert")), &data).unwrap();
        let text = data.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        fs::write(out.join(format!("{label}.hex")), text + "\n").unwrap();
        println!("{label}: {} bytes", data.len());
    }
}

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i+2], 16).unwrap()).collect()
}
