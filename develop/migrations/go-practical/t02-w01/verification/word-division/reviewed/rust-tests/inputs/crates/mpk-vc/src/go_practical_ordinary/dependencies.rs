//! Untrusted closed ordinary assembly. This supplies no checked authority.
use super::*;
use std::collections::BTreeSet;

impl Builder {
    /// Retain complete type/proof/metadata dependencies of compiled-in roots.
    /// Primitive families remain complete. Original declaration and term order
    /// is preserved; every remaining reference is remapped, never redirected.
    pub(super) fn retain_ordinary_dependencies(&mut self, roots: &[String]) -> Result<()> {
        if !self.certificate.imports.is_empty()
            || !self.certificate.proof_node_table.is_empty()
            || !self.certificate.theory_certificates.is_empty()
            || self.certificate.source_manifest.is_some()
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let mut pending = roots
            .iter()
            .map(|name| {
                self.globals
                    .get(name)
                    .copied()
                    .ok_or(GoOrdinaryError::Linkage)
            })
            .collect::<Result<Vec<_>>>()?;
        for (i, declaration) in self.certificate.declarations.iter().enumerate() {
            if matches!(
                declaration.kind,
                DeclarationKind::Inductive { .. }
                    | DeclarationKind::Constructor { .. }
                    | DeclarationKind::Recursor { .. }
            ) {
                pending.push(i as u32);
            }
        }
        let (mut declarations, mut terms) = (BTreeSet::new(), BTreeSet::new());
        while let Some(index) = pending.pop() {
            if !declarations.insert(index) {
                continue;
            }
            let mut nodes = match &self.certificate.declarations[index as usize].kind {
                DeclarationKind::Def { ty, value, .. } => vec![*ty, *value],
                DeclarationKind::Theorem { ty, proof } => vec![*ty, *proof],
                DeclarationKind::Inductive { ty } => vec![*ty],
                DeclarationKind::Constructor { ty, inductive, .. }
                | DeclarationKind::Recursor { ty, inductive, .. } => {
                    pending.push(*inductive);
                    vec![*ty]
                }
                _ => return Err(GoOrdinaryError::Foundation),
            };
            while let Some(node) = nodes.pop() {
                if !terms.insert(node) {
                    continue;
                }
                match &self.certificate.term_table[node as usize] {
                    TermNode::Sort(_) | TermNode::Var(_) => {}
                    TermNode::Const { global, .. } => pending.push(*global),
                    TermNode::App {
                        function,
                        arguments,
                    } => {
                        nodes.push(*function);
                        nodes.extend(arguments.iter().copied());
                    }
                    TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                        nodes.extend([*ty, *body]);
                    }
                    TermNode::Let { ty, value, body } => nodes.extend([*ty, *value, *body]),
                }
            }
        }
        let ids = |old: &BTreeSet<u32>| {
            old.iter()
                .enumerate()
                .map(|(new, old)| (*old, new as u32))
                .collect::<BTreeMap<_, _>>()
        };
        let (globals, nodes) = (ids(&declarations), ids(&terms));
        let mut used_names = declarations
            .iter()
            .map(|i| self.certificate.declarations[*i as usize].name)
            .collect::<BTreeSet<_>>();
        for level in &self.certificate.level_table {
            if let LevelNode::Param(name) = level {
                used_names.insert(*name);
            }
        }
        let names = ids(&used_names);
        let name_table = used_names
            .iter()
            .map(|i| self.certificate.name_table[*i as usize].clone())
            .collect();
        let mut retained = Vec::with_capacity(declarations.len());
        for index in &declarations {
            let declaration = &self.certificate.declarations[*index as usize];
            let kind = match &declaration.kind {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => DeclarationKind::Def {
                    ty: nodes[ty],
                    value: nodes[value],
                    reducibility: *reducibility,
                },
                DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                    ty: nodes[ty],
                    proof: nodes[proof],
                },
                DeclarationKind::Inductive { ty } => DeclarationKind::Inductive { ty: nodes[ty] },
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Constructor {
                    ty: nodes[ty],
                    inductive: globals[inductive],
                    generated: *generated,
                },
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Recursor {
                    ty: nodes[ty],
                    inductive: globals[inductive],
                    generated: *generated,
                },
                _ => return Err(GoOrdinaryError::Foundation),
            };
            retained.push(Declaration {
                name: names[&declaration.name],
                kind,
            });
        }
        let term_table = terms
            .iter()
            .map(
                |index| match &self.certificate.term_table[*index as usize] {
                    TermNode::Sort(level) => TermNode::Sort(*level),
                    TermNode::Var(index) => TermNode::Var(*index),
                    TermNode::Const { global, levels } => TermNode::Const {
                        global: globals[global],
                        levels: levels.clone(),
                    },
                    TermNode::App {
                        function,
                        arguments,
                    } => TermNode::App {
                        function: nodes[function],
                        arguments: arguments.iter().map(|i| nodes[i]).collect(),
                    },
                    TermNode::Lam { ty, body } => TermNode::Lam {
                        ty: nodes[ty],
                        body: nodes[body],
                    },
                    TermNode::Pi { ty, body } => TermNode::Pi {
                        ty: nodes[ty],
                        body: nodes[body],
                    },
                    TermNode::Let { ty, value, body } => TermNode::Let {
                        ty: nodes[ty],
                        value: nodes[value],
                        body: nodes[body],
                    },
                },
            )
            .collect();
        for level in &mut self.certificate.level_table {
            if let LevelNode::Param(name) = level {
                *name = names[name];
            }
        }
        self.certificate.name_table = name_table;
        self.certificate.declarations = retained;
        self.certificate.term_table = term_table;
        self.sort = nodes[&self.sort];
        self.boolean = nodes[&self.boolean];
        self.globals = self
            .certificate
            .declarations
            .iter()
            .enumerate()
            .map(|(i, d)| {
                (
                    self.certificate.name_table[d.name as usize].clone(),
                    i as u32,
                )
            })
            .collect();
        self.terms = self
            .certificate
            .term_table
            .iter()
            .enumerate()
            .map(|(i, term)| (key(term), i as u32))
            .collect();
        Ok(())
    }
}
