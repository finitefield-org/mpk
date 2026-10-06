//! Complete eligible actual-default goals on the original foundation context.
//! Ineligible source-use obligations remain explicit until native proofs exist.
use super::super::super::super::super::super::ownership_proofs::{
    logic, prove_closed_boolean_definitions, publish_theorem,
};
use super::*;
use mpk_cert::encode::Certificate;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDefaultGoalProof {
    pub goal: ContractTerm,
    pub definition: String,
    pub theorem: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryActualDefaultProof {
    pub sequent: BindingSequent,
    pub goals: Vec<OrdinaryDefaultGoalProof>,
    pub proposition_definition: String,
    pub theorem: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDefaultProofProgram {
    schema: String,
    identity: OrdinaryIdentityProofProgram,
    original_default_program_sha256: String,
    original_default_certificate_sha256: String,
    proofs: Vec<OrdinaryActualDefaultProof>,
    pending_defaults: Vec<OrdinaryBindingDefaultPending>,
    supplied_binding_sequent_ids: Vec<String>,
    remaining_binding_sequent_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    imported_definitions: Vec<String>,
    compatible_shared_declarations: Vec<String>,
    static_transformers: usize,
    proof_check_pending: bool,
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    definition_certificate: Vec<u8>,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDefaultProofProgram {
    pub fn identity(&self) -> &OrdinaryIdentityProofProgram {
        &self.identity
    }
    pub fn proofs(&self) -> &[OrdinaryActualDefaultProof] {
        &self.proofs
    }
    pub fn pending_defaults(&self) -> &[OrdinaryBindingDefaultPending] {
        &self.pending_defaults
    }
    pub fn supplied_binding_sequent_ids(&self) -> &[String] {
        &self.supplied_binding_sequent_ids
    }
    pub fn remaining_binding_sequent_ids(&self) -> &[String] {
        &self.remaining_binding_sequent_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn definition_certificate_bytes(&self) -> &[u8] {
        &self.definition_certificate
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("original actual-default proof candidates")
    }
}

// Intern the existing DAG structurally without changing its original nodes or
// declaration bodies. Frozen foundations can contain duplicate term indices.
fn canonical(b: &mut Builder, t: u32, memo: &mut BTreeMap<u32, u32>) -> R<u32> {
    if let Some(&v) = memo.get(&t) {
        return Ok(v);
    }
    let node = match b.c.term_table[t as usize].clone() {
        TermNode::App {
            function,
            arguments,
        } => TermNode::App {
            function: canonical(b, function, memo)?,
            arguments: arguments
                .into_iter()
                .map(|t| canonical(b, t, memo))
                .collect::<R<_>>()?,
        },
        TermNode::Lam { ty, body } => TermNode::Lam {
            ty: canonical(b, ty, memo)?,
            body: canonical(b, body, memo)?,
        },
        TermNode::Pi { ty, body } => TermNode::Pi {
            ty: canonical(b, ty, memo)?,
            body: canonical(b, body, memo)?,
        },
        TermNode::Let { ty, value, body } => TermNode::Let {
            ty: canonical(b, ty, memo)?,
            value: canonical(b, value, memo)?,
            body: canonical(b, body, memo)?,
        },
        other => other,
    };
    let v = b.term(node)?;
    memo.insert(t, v);
    Ok(v)
}
fn roots(kind: &DeclarationKind) -> R<Vec<u32>> {
    Ok(match kind {
        DeclarationKind::Def { ty, value, .. } => vec![*ty, *value],
        DeclarationKind::Theorem { ty, proof } => vec![*ty, *proof],
        DeclarationKind::Inductive { ty }
        | DeclarationKind::Constructor { ty, .. }
        | DeclarationKind::Recursor { ty, .. } => vec![*ty],
        _ => return Err(OrdinaryCarrierError::Linkage),
    })
}
fn remap(
    b: &mut Builder,
    c: &Certificate,
    t: u32,
    globals: &BTreeMap<u32, u32>,
    levels: &[u32],
    memo: &mut BTreeMap<u32, u32>,
) -> R<u32> {
    if let Some(&v) = memo.get(&t) {
        return Ok(v);
    }
    let node = match &c.term_table[t as usize] {
        TermNode::Sort(l) => TermNode::Sort(levels[*l as usize]),
        TermNode::Var(i) => TermNode::Var(*i),
        TermNode::Const { global, levels: ls } => TermNode::Const {
            global: *globals.get(global).ok_or(OrdinaryCarrierError::Linkage)?,
            levels: ls.iter().map(|l| levels[*l as usize]).collect(),
        },
        TermNode::App {
            function,
            arguments,
        } => TermNode::App {
            function: remap(b, c, *function, globals, levels, memo)?,
            arguments: arguments
                .iter()
                .map(|t| remap(b, c, *t, globals, levels, memo))
                .collect::<R<_>>()?,
        },
        TermNode::Lam { ty, body } => TermNode::Lam {
            ty: remap(b, c, *ty, globals, levels, memo)?,
            body: remap(b, c, *body, globals, levels, memo)?,
        },
        TermNode::Pi { ty, body } => TermNode::Pi {
            ty: remap(b, c, *ty, globals, levels, memo)?,
            body: remap(b, c, *body, globals, levels, memo)?,
        },
        TermNode::Let { ty, value, body } => TermNode::Let {
            ty: remap(b, c, *ty, globals, levels, memo)?,
            value: remap(b, c, *value, globals, levels, memo)?,
            body: remap(b, c, *body, globals, levels, memo)?,
        },
    };
    let v = b.term(node)?;
    memo.insert(t, v);
    Ok(v)
}
// Private linkage for freshly source-generated definitions. Every shared
// declaration must have the same complete kind, type and body after remapping.
// Conflicts are never overwritten, normalized by evaluation, or assumed equal.
fn append(b: &mut Builder, c: &Certificate, names: &[String]) -> R<(Vec<String>, Vec<String>)> {
    if !c.proof_node_table.is_empty()
        || !c.theory_certificates.is_empty()
        || c.module != b.c.module
        || c.imports != b.c.imports
        || c.source_manifest != b.c.source_manifest
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    for d in &c.declarations {
        roots(&d.kind)?;
    }
    let index = c
        .declarations
        .iter()
        .enumerate()
        .map(|(i, d)| (c.name_table[d.name as usize].as_str(), i as u32))
        .collect::<BTreeMap<_, _>>();
    let mut wanted = names
        .iter()
        .map(|n| {
            index
                .get(n.as_str())
                .copied()
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let mut closure = BTreeSet::new();
    while let Some(g) = wanted.pop() {
        if !closure.insert(g) {
            continue;
        }
        let d = &c.declarations[g as usize];
        let mut terms = roots(&d.kind)?;
        let mut seen = BTreeSet::new();
        if let DeclarationKind::Constructor { inductive, .. }
        | DeclarationKind::Recursor { inductive, .. } = d.kind
        {
            wanted.push(inductive)
        }
        while let Some(t) = terms.pop() {
            if !seen.insert(t) {
                continue;
            }
            match &c.term_table[t as usize] {
                TermNode::Const { global, .. } => wanted.push(*global),
                TermNode::App {
                    function,
                    arguments,
                } => {
                    terms.push(*function);
                    terms.extend(arguments)
                }
                TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => terms.extend([ty, body]),
                TermNode::Let { ty, value, body } => terms.extend([ty, value, body]),
                _ => {}
            }
        }
    }
    let mut levels = vec![];
    for level in &c.level_table {
        let node = match level {
            LevelNode::Zero => LevelNode::Zero,
            LevelNode::Succ(l) => LevelNode::Succ(levels[*l as usize]),
            LevelNode::Max(a, z) => LevelNode::Max(levels[*a as usize], levels[*z as usize]),
            LevelNode::Param(_) => return Err(OrdinaryCarrierError::Linkage),
        };
        let l =
            b.c.level_table
                .iter()
                .position(|n| n == &node)
                .unwrap_or_else(|| {
                    b.c.level_table.push(node);
                    b.c.level_table.len() - 1
                });
        levels.push(l as u32);
    }
    let mut globals = BTreeMap::new();
    let mut memo = BTreeMap::new();
    let mut old_memo = BTreeMap::new();
    let mut imported = vec![];
    let mut shared = vec![];
    for g in closure {
        let d = &c.declarations[g as usize];
        let name = &c.name_table[d.name as usize];
        let old = b.globals.get(name).copied();
        let target = old.unwrap_or(b.c.declarations.len() as u32);
        globals.insert(g, target);
        let mut lower = |t| remap(b, c, t, &globals, &levels, &mut memo);
        let kind = match d.kind {
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => DeclarationKind::Def {
                ty: lower(ty)?,
                value: lower(value)?,
                reducibility,
            },
            DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                ty: lower(ty)?,
                proof: lower(proof)?,
            },
            DeclarationKind::Inductive { ty } => DeclarationKind::Inductive { ty: lower(ty)? },
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Constructor {
                ty: lower(ty)?,
                inductive: *globals
                    .get(&inductive)
                    .ok_or(OrdinaryCarrierError::Linkage)?,
                generated,
            },
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Recursor {
                ty: lower(ty)?,
                inductive: *globals
                    .get(&inductive)
                    .ok_or(OrdinaryCarrierError::Linkage)?,
                generated,
            },
            _ => return Err(OrdinaryCarrierError::Linkage),
        };
        if let Some(old) = old {
            let actual = b.c.declarations[old as usize].kind.clone();
            let actual = match actual {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => DeclarationKind::Def {
                    ty: canonical(b, ty, &mut old_memo)?,
                    value: canonical(b, value, &mut old_memo)?,
                    reducibility,
                },
                DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                    ty: canonical(b, ty, &mut old_memo)?,
                    proof: canonical(b, proof, &mut old_memo)?,
                },
                DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
                    ty: canonical(b, ty, &mut old_memo)?,
                },
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Constructor {
                    ty: canonical(b, ty, &mut old_memo)?,
                    inductive,
                    generated,
                },
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => DeclarationKind::Recursor {
                    ty: canonical(b, ty, &mut old_memo)?,
                    inductive,
                    generated,
                },
                _ => return Err(OrdinaryCarrierError::Linkage),
            };
            if actual != kind {
                return Err(OrdinaryCarrierError::Linkage);
            }
            shared.push(name.clone());
        } else {
            let DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } = kind
            else {
                return Err(OrdinaryCarrierError::Linkage);
            };
            if reducibility != mpk_cert::encode::DefinitionReducibility::Reducible {
                return Err(OrdinaryCarrierError::Linkage);
            }
            b.define(name, ty, value)?;
            imported.push(name.clone());
        }
    }
    Ok((imported, shared))
}

pub fn generate_csharp_practical_ordinary_default_proofs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDefaultProofProgram> {
    let identity = generate_csharp_practical_ordinary_identity_proofs(vir)?;
    let defaults = generate_csharp_practical_ordinary_binding_defaults(vir)?;
    if defaults.pending_proof_ids() != identity.pending_proof_ids()
        || defaults.public_domains() != identity.foundation().definition_program().public_domains()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut b = Builder::resume(identity.certificate_bytes())?;
    let names = defaults
        .conditions()
        .iter()
        .flat_map(|c| c.goal_definitions.iter().cloned())
        .collect::<Vec<_>>();
    let (imported_definitions, compatible_shared_declarations) = if names.is_empty() {
        (vec![], vec![])
    } else {
        append(
            &mut b,
            &decode_canonical_certificate(defaults.certificate_bytes())
                .map_err(|_| OrdinaryCarrierError::Linkage)?,
            &names,
        )?
    };
    let definition_certificate = b.clone().finish()?;
    let goal_theorems = prove_closed_boolean_definitions(&mut b, &names)?;
    if !names.is_empty() {
        logic(&mut b)?
    }
    let goal_map = names
        .into_iter()
        .zip(goal_theorems)
        .collect::<BTreeMap<_, _>>();
    let mut proofs = vec![];
    let original_default_program_sha256 =
        format!("{:x}", Sha256::digest(defaults.canonical_bytes()));
    for condition in defaults.conditions() {
        let sequent = &condition.sequent;
        if sequent.kind != "actual_default"
            || !sequent.subjects.is_empty()
            || !sequent.assumptions.is_empty()
            || sequent.goals.len() != 2
            || condition.goal_definitions.len() != 2
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let mut statements = vec![];
        let mut goals = vec![];
        for (goal, definition) in sequent.goals.iter().zip(&condition.goal_definitions) {
            let value = b.constant(definition)?;
            let yes = bit(&mut b, true)?;
            let boolean = b.boolean;
            let ty = call(&mut b, "Std.Eq", vec![boolean, value, yes])?;
            let theorem = goal_map
                .get(definition)
                .ok_or(OrdinaryCarrierError::Linkage)?
                .clone();
            let proof = b.constant(&theorem)?;
            statements.push((ty, proof));
            goals.push(OrdinaryDefaultGoalProof {
                goal: goal.clone(),
                definition: definition.clone(),
                theorem,
            });
        }
        let ty = call(
            &mut b,
            "Std.Logic.And",
            vec![statements[0].0, statements[1].0],
        )?;
        let proof = call(
            &mut b,
            "Std.Logic.And.intro",
            vec![
                statements[0].0,
                statements[1].0,
                statements[0].1,
                statements[1].1,
            ],
        )?;
        let hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&original_default_program_sha256, sequent))
                    .map_err(|_| OrdinaryCarrierError::Linkage)?
            )
        );
        let proposition_definition = format!("{PREFIX}.ActualDefaultProof.Type.H{hash}");
        b.define(&proposition_definition, b.sort, ty)?;
        let theorem = format!("{PREFIX}.ActualDefaultProof.Theorem.H{hash}");
        let ty = b.constant(&proposition_definition)?;
        publish_theorem(&mut b, &theorem, ty, proof)?;
        proofs.push(OrdinaryActualDefaultProof {
            sequent: sequent.clone(),
            goals,
            proposition_definition,
            theorem,
        });
    }
    let mut supplied = identity
        .supplied_binding_sequent_ids()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for proof in &proofs {
        if !supplied.insert(proof.sequent.id.clone()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    let pending_proof_ids = identity.pending_proof_ids().to_vec();
    // Count the entire source-generated input's work conservatively, even when
    // only its reachable goal definitions need to be appended to the DAG.
    let added = if proofs.is_empty() {
        0
    } else {
        defaults.static_transformers()
    };
    let static_transformers = identity
        .static_transformers()
        .checked_add(added)
        .and_then(|n| n.checked_add(b.static_transformers))
        .ok_or(OrdinaryCarrierError::Limit)?;
    if static_transformers > crate::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX as usize {
        return Err(OrdinaryCarrierError::Limit);
    }
    let certificate = b.finish()?;
    let result = OrdinaryDefaultProofProgram {
        schema: "mpk.csharp.ordinary_default_proofs.v1".into(),
        original_default_certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(
            defaults.certificate_bytes(),
        )),
        original_default_program_sha256,
        proofs,
        pending_defaults: defaults.pending_defaults().to_vec(),
        supplied_binding_sequent_ids: pending_proof_ids
            .iter()
            .filter(|id| supplied.contains(*id))
            .cloned()
            .collect(),
        remaining_binding_sequent_ids: pending_proof_ids
            .iter()
            .filter(|id| !supplied.contains(*id))
            .cloned()
            .collect(),
        pending_proof_ids,
        identity,
        imported_definitions,
        compatible_shared_declarations,
        static_transformers,
        proof_check_pending: true,
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        definition_certificate,
        certificate,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}
pub fn import_csharp_practical_ordinary_default_proofs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDefaultProofProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_default_proofs(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csharp_03_t06_w09_default_linkage_rejects_typed_shared_conflict() {
        let mut original = Builder::new().unwrap();
        let names = vec!["Std.Bool.and".to_owned()];
        let certificate = original.c.clone();
        let (imported, shared) = append(&mut original, &certificate, &names).unwrap();
        assert!(imported.is_empty());
        assert!(shared.contains(&names[0]));

        let mut changed = Builder::new().unwrap();
        let global = changed.globals[&names[0]] as usize;
        let DeclarationKind::Def {
            ty,
            mut value,
            reducibility,
        } = changed.c.declarations[global].kind
        else {
            panic!("missing shared Boolean function")
        };
        let mut binders = vec![];
        while let TermNode::Lam { ty, body } = changed.c.term_table[value as usize] {
            binders.push(ty);
            value = body;
        }
        let mut value = bit(&mut changed, false).unwrap();
        for ty in binders.into_iter().rev() {
            value = changed.lam(ty, value).unwrap();
        }
        changed.c.declarations[global].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let bytes = changed.finish().unwrap();
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(&bytes)
                .unwrap()
                .axiom_count,
            0
        );
        let changed = decode_canonical_certificate(&bytes).unwrap();
        assert_eq!(
            append(&mut original, &changed, &names).unwrap_err(),
            OrdinaryCarrierError::Linkage
        );
    }
}
