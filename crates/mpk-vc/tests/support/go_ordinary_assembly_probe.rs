//! Diagnostic closed-table assembly: preserve every original declaration and term.
use mpk_cert::encode::{
    Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode, TermNode,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
enum LevelKey {
    Zero,
    Succ(u32),
    Max(u32, u32),
    Param(u32),
}
fn level_key(n: &LevelNode) -> LevelKey {
    match n {
        LevelNode::Zero => LevelKey::Zero,
        LevelNode::Succ(i) => LevelKey::Succ(*i),
        LevelNode::Max(a, b) => LevelKey::Max(*a, *b),
        LevelNode::Param(i) => LevelKey::Param(*i),
    }
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
enum TermKey {
    Sort(u32),
    Var(u32),
    Const(u32, Vec<u32>),
    App(u32, Vec<u32>),
    Lam(u32, u32),
    Pi(u32, u32),
    Let(u32, u32, u32),
}
fn term_key(n: &TermNode) -> TermKey {
    match n {
        TermNode::Sort(i) => TermKey::Sort(*i),
        TermNode::Var(i) => TermKey::Var(*i),
        TermNode::Const { global, levels } => TermKey::Const(*global, levels.clone()),
        TermNode::App {
            function,
            arguments,
        } => TermKey::App(*function, arguments.clone()),
        TermNode::Lam { ty, body } => TermKey::Lam(*ty, *body),
        TermNode::Pi { ty, body } => TermKey::Pi(*ty, *body),
        TermNode::Let { ty, value, body } => TermKey::Let(*ty, *value, *body),
    }
}

pub(super) fn encode(mut c: Certificate) -> Vec<u8> {
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes = CertificateHashes {
        export_hash: mpk_cert::export_block_hash(&c.export_block),
        axiom_report_hash: mpk_cert::axiom_report_hash_for_report(&c.axiom_report),
        certificate_hash: [0; 32],
    };
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}

pub(super) fn merge(inputs: &[Certificate]) -> Result<Certificate, String> {
    assert!(!inputs.is_empty());
    let mut names = BTreeSet::new();
    let mut globals = BTreeMap::new();
    let mut count = 0;
    for c in inputs {
        if !c.imports.is_empty()
            || !c.proof_node_table.is_empty()
            || !c.theory_certificates.is_empty()
            || c.source_manifest.is_some()
            || c.axiom_report.summary.total_axiom_count != 0
            || !c.axiom_report.entries.is_empty()
        {
            return Err("non-ordinary or imported assembly input".into());
        }
        names.extend(c.name_table.iter().cloned());
        for d in &c.declarations {
            let name = &c.name_table[d.name as usize];
            if !globals.contains_key(name) {
                globals.insert(name.clone(), count);
                count += 1;
            }
        }
    }
    let mut output = inputs[0].clone();
    output.module = "Mpk.Go.Practical.FullShiftProductAssemblyProbe".into();
    output.name_table = names.into_iter().collect();
    output.level_table.clear();
    output.term_table.clear();
    output.declarations.clear();
    output.export_block.clear();
    output.axiom_report = Default::default();
    output.hashes = Default::default();
    let name_id = output
        .name_table
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i as u32))
        .collect::<BTreeMap<_, _>>();
    let mut levels = BTreeMap::new();
    let mut terms = BTreeMap::new();
    for c in inputs {
        let gm = c
            .declarations
            .iter()
            .map(|d| globals[&c.name_table[d.name as usize]])
            .collect::<Vec<_>>();
        let mut lm = vec![];
        for n in &c.level_table {
            let node = match n {
                LevelNode::Zero => LevelNode::Zero,
                LevelNode::Succ(i) => LevelNode::Succ(lm[*i as usize]),
                LevelNode::Max(a, b) => LevelNode::Max(lm[*a as usize], lm[*b as usize]),
                LevelNode::Param(i) => LevelNode::Param(name_id[&c.name_table[*i as usize]]),
            };
            let key = level_key(&node);
            let id = if let Some(id) = levels.get(&key) {
                *id
            } else {
                let id = output.level_table.len() as u32;
                output.level_table.push(node);
                levels.insert(key, id);
                id
            };
            lm.push(id);
        }
        let mut tm = vec![];
        for n in &c.term_table {
            let node = match n {
                TermNode::Sort(i) => TermNode::Sort(lm[*i as usize]),
                TermNode::Var(i) => TermNode::Var(*i),
                TermNode::Const { global, levels } => TermNode::Const {
                    global: gm[*global as usize],
                    levels: levels.iter().map(|i| lm[*i as usize]).collect(),
                },
                TermNode::App {
                    function,
                    arguments,
                } => TermNode::App {
                    function: tm[*function as usize],
                    arguments: arguments.iter().map(|i| tm[*i as usize]).collect(),
                },
                TermNode::Lam { ty, body } => TermNode::Lam {
                    ty: tm[*ty as usize],
                    body: tm[*body as usize],
                },
                TermNode::Pi { ty, body } => TermNode::Pi {
                    ty: tm[*ty as usize],
                    body: tm[*body as usize],
                },
                TermNode::Let { ty, value, body } => TermNode::Let {
                    ty: tm[*ty as usize],
                    value: tm[*value as usize],
                    body: tm[*body as usize],
                },
            };
            let key = term_key(&node);
            let id = if let Some(id) = terms.get(&key) {
                *id
            } else {
                let id = output.term_table.len() as u32;
                output.term_table.push(node);
                terms.insert(key, id);
                id
            };
            tm.push(id);
        }
        for (i, d) in c.declarations.iter().enumerate() {
            let kind = match &d.kind {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => DeclarationKind::Def {
                    ty: tm[*ty as usize],
                    value: tm[*value as usize],
                    reducibility: *reducibility,
                },
                DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                    ty: tm[*ty as usize],
                    proof: tm[*proof as usize],
                },
                DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
                    ty: tm[*ty as usize],
                },
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Constructor {
                    ty: tm[*ty as usize],
                    inductive: gm[*inductive as usize],
                    generated: *generated,
                },
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Recursor {
                    ty: tm[*ty as usize],
                    inductive: gm[*inductive as usize],
                    generated: *generated,
                },
                _ => return Err("non-ordinary declaration in assembly".into()),
            };
            let declaration = Declaration {
                name: name_id[&c.name_table[d.name as usize]],
                kind,
            };
            let id = gm[i] as usize;
            if id < output.declarations.len() {
                if output.declarations[id] != declaration {
                    return Err(format!(
                        "nominal identity conflict: {}",
                        c.name_table[d.name as usize]
                    ));
                }
            } else {
                assert_eq!(id, output.declarations.len());
                output.declarations.push(declaration);
            }
        }
    }
    assert_eq!(output.declarations.len(), globals.len());
    Ok(output)
}
