//! Canonical, type-level natural-number recursion returning Sort0.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId, TermArena,
    TermId, TermNode,
};

pub(crate) struct NatRecTypeSignature {
    pub ty: TermId,
    pub constructors: [GlobalId; 2],
}

pub(crate) fn check_nat_rec_type_constant_levels(
    env: &Environment,
    global: GlobalId,
    has_levels: bool,
) -> Result<(), CoreError> {
    if !has_levels {
        return Ok(());
    }
    let Some(declaration) = env.lookup(global) else {
        return Ok(());
    };
    let monomorphic = match declaration.kind() {
        DeclarationKind::Inductive { .. } => has_nat_rec_type(env, global),
        DeclarationKind::Constructor { inductive, .. } => has_nat_rec_type(env, inductive),
        DeclarationKind::Recursor { inductive, .. } => {
            env.lookup(inductive).is_some_and(|family| {
                declaration.name().as_str() == format!("{}.recType", family.name().as_str())
            })
        }
        _ => false,
    };
    if monomorphic {
        return Err(invalid("nonempty_universe_arguments"));
    }
    Ok(())
}

fn invalid(detail: &str) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidDeclaration, CoreLocation::root())
        .with_detail("kind", "invalid_nat_rec_type_interface")
        .with_detail("reason", detail)
}

pub(crate) fn nat_rec_type_signature(
    terms: &mut TermArena,
    env: &Environment,
    family: GlobalId,
) -> Result<NatRecTypeSignature, CoreError> {
    let declaration = env
        .lookup(family)
        .ok_or_else(|| invalid("missing_family"))?;
    let DeclarationKind::Inductive { ty: sort } = declaration.kind() else {
        return Err(invalid("not_an_inductive_family"));
    };
    // Every LevelArena reserves index zero for the normalized zero universe.
    if !matches!(terms.node(sort), TermNode::Sort(level) if level.index() == 0) {
        return Err(invalid("family_is_not_sort_zero"));
    }
    let family_name = declaration.name().as_str();
    let constructors = env
        .iter()
        .filter(|declaration| {
            matches!(declaration.kind(), DeclarationKind::Constructor { inductive, .. } if inductive == family)
        })
        .collect::<Vec<_>>();
    let [no, yes] = constructors.as_slice() else {
        return Err(invalid("constructor_count"));
    };
    let natural = terms.constant(family, []);
    let successor_type = terms.pi(natural, natural);
    for (constructor, suffix, ty) in [(*no, "zero", natural), (*yes, "succ", successor_type)] {
        if constructor.name().as_str() != format!("{family_name}.{suffix}")
            || !constructor.kind().is_generated_constructor()
            || constructor.ty() != ty
        {
            return Err(invalid("noncanonical_constructor"));
        }
    }
    let no_global = no.global();
    let yes_global = yes.global();
    // The result is a type in Sort0; this is a fixed large eliminator,
    // not a universe-polymorphic rule or a change to legacy Nat.rec/induct.
    let step_result = terms.pi(sort, sort);
    let step_type = terms.pi(natural, step_result);
    let result = terms.pi(natural, sort);
    let result = terms.pi(step_type, result);
    let ty = terms.pi(sort, result);
    Ok(NatRecTypeSignature {
        ty,
        constructors: [no_global, yes_global],
    })
}

/// Validate the reserved `<family>.recType` interface before registering it.
/// Other recursor names retain their existing declaration checking behavior.
pub fn check_nat_rec_type_declaration(
    terms: &mut TermArena,
    env: &Environment,
    name: &str,
    ty: TermId,
    family: GlobalId,
    generated: bool,
) -> Result<(), CoreError> {
    let declaration = env
        .lookup(family)
        .ok_or_else(|| invalid("missing_family"))?;
    if name != format!("{}.recType", declaration.name().as_str()) {
        return Ok(());
    }
    if !generated {
        return Err(invalid("nongenerated_type_recursion"));
    }
    let signature = nat_rec_type_signature(terms, env, family)?;
    if ty != signature.ty {
        return Err(invalid("noncanonical_eliminator_type"));
    }
    // Earlier declarations were checked before this family became monomorphic.
    // Inspect their actual types, values and proofs so declaration order cannot
    // hide a universe argument in a reducible or opaque definition.
    let interface = [family, signature.constructors[0], signature.constructors[1]];
    let mut pending = Vec::new();
    for declaration in env.iter() {
        pending.push(declaration.ty());
        pending.extend(declaration.kind().definition_value());
        pending.extend(declaration.kind().theorem_proof());
    }
    let mut seen = std::collections::HashSet::new();
    while let Some(term) = pending.pop() {
        if !seen.insert(term) {
            continue;
        }
        match terms.node(term) {
            TermNode::Const { global, levels }
                if interface.contains(global) && !levels.is_empty() =>
            {
                return Err(invalid("prior_nonempty_universe_arguments"));
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                pending.extend([*ty, *body]);
            }
            TermNode::App {
                function,
                arguments,
            } => {
                pending.push(*function);
                pending.extend(arguments.iter().copied());
            }
            TermNode::Let { ty, value, body } => pending.extend([*ty, *value, *body]),
            TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => {}
        }
    }
    Ok(())
}

pub(crate) fn has_nat_rec_type(env: &Environment, family: GlobalId) -> bool {
    let Some(declaration) = env.lookup(family) else {
        return false;
    };
    let name = format!("{}.recType", declaration.name().as_str());
    env.lookup_by_name(name).ok().flatten().is_some_and(
        |d| matches!(d.kind(), DeclarationKind::Recursor { inductive, .. } if inductive == family),
    )
}
