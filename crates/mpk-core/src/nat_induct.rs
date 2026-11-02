//! Canonical, dependent natural-number elimination into Sort0.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId, TermArena,
    TermId, TermNode,
};

pub(crate) struct NatInductSignature {
    pub ty: TermId,
    pub constructors: [GlobalId; 2],
}

pub(crate) fn check_nat_induct_constant_levels(
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
        DeclarationKind::Inductive { .. } => has_nat_induct(env, global),
        DeclarationKind::Constructor { inductive, .. } => has_nat_induct(env, inductive),
        DeclarationKind::Recursor { inductive, .. } => {
            env.lookup(inductive).is_some_and(|family| {
                declaration.name().as_str() == format!("{}.induct", family.name().as_str())
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
        .with_detail("kind", "invalid_nat_induct_interface")
        .with_detail("reason", detail)
}

pub(crate) fn nat_induct_signature(
    terms: &mut TermArena,
    env: &Environment,
    family: GlobalId,
) -> Result<NatInductSignature, CoreError> {
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
    let zero = terms.constant(no_global, []);
    let successor = terms.constant(yes_global, []);
    let motive_type = terms.pi(natural, sort);
    let p = terms.var(0);
    let zero_type = terms.app(p, [zero]);
    let p = terms.var(2);
    let n = terms.var(0);
    let induction_hypothesis = terms.app(p, [n]);
    let p = terms.var(3);
    let n = terms.var(1);
    let next_n = terms.app(successor, [n]);
    let step_result = terms.app(p, [next_n]);
    let step_result = terms.pi(induction_hypothesis, step_result);
    let step_type = terms.pi(natural, step_result);
    let p = terms.var(3);
    let n = terms.var(0);
    let result = terms.app(p, [n]);
    let major_to_result = terms.pi(natural, result);
    let step_to_major = terms.pi(step_type, major_to_result);
    let zero_to_step = terms.pi(zero_type, step_to_major);
    let ty = terms.pi(motive_type, zero_to_step);
    Ok(NatInductSignature {
        ty,
        constructors: [no_global, yes_global],
    })
}

/// Validate the reserved `<family>.induct` interface before registering it.
/// Other recursor names retain their existing declaration checking behavior.
pub fn check_nat_induct_declaration(
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
    if name != format!("{}.induct", declaration.name().as_str()) {
        if name.ends_with(".induct") {
            return Err(invalid("reserved_name_for_different_family"));
        }
        return Ok(());
    }
    if !generated {
        return Err(invalid("nongenerated_induction"));
    }
    let signature = nat_induct_signature(terms, env, family)?;
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

pub(crate) fn has_nat_induct(env: &Environment, family: GlobalId) -> bool {
    let Some(declaration) = env.lookup(family) else {
        return false;
    };
    let name = format!("{}.induct", declaration.name().as_str());
    env.lookup_by_name(name).ok().flatten().is_some_and(
        |d| matches!(d.kind(), DeclarationKind::Recursor { inductive, .. } if inductive == family),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        definitionally_equal, generate_mvp_inductive_declarations, infer, InductiveGenerationInput,
        LevelArena, LocalContext, MvpInductiveShape,
    };
    fn setup() -> (LevelArena, TermArena, Environment, GlobalId, [GlobalId; 2]) {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let sort = terms.sort(levels.zero());
        let g = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Nat, "Test.Nat", vec![], sort),
        )
        .unwrap();
        (
            levels,
            terms,
            env,
            g.family.global,
            [g.constructors[0].global, g.constructors[1].global],
        )
    }
    fn install(terms: &mut TermArena, env: &mut Environment, family: GlobalId) -> GlobalId {
        let ty = nat_induct_signature(terms, env, family).unwrap().ty;
        check_nat_induct_declaration(terms, env, "Test.Nat.induct", ty, family, true).unwrap();
        env.register_generated_recursor("Test.Nat.induct", ty, family)
            .unwrap()
    }
    #[test]
    fn equations_keep_the_old_recursor_and_infer_nested_successors() {
        let (mut l, mut t, mut e, f, cs) = setup();
        let old = e
            .iter()
            .map(|d| (d.name().clone(), d.kind()))
            .collect::<Vec<_>>();
        let g = install(&mut t, &mut e, f);
        assert_eq!(
            old,
            e.iter()
                .take(old.len())
                .map(|d| (d.name().clone(), d.kind()))
                .collect::<Vec<_>>()
        );
        let nat = t.constant(f, []);
        let zero = t.constant(cs[0], []);
        let succ = t.constant(cs[1], []);
        let rec = t.constant(g, []);
        let motive = t.lam(nat, nat);
        let v = t.var(0);
        let body = t.app(succ, [v]);
        let body = t.lam(nat, body);
        let step = t.lam(nat, body);
        let mut n = zero;
        for _ in 0..4 {
            let application = t.app(rec, [motive, zero, step, n]);
            let inferred = infer(&mut l, &mut t, &LocalContext::new(), &e, application).unwrap();
            assert!(definitionally_equal(&e, &mut t, inferred, nat).unwrap());
            assert!(definitionally_equal(&e, &mut t, application, n).unwrap());
            n = t.app(succ, [n]);
        }
    }
    #[test]
    fn malformed_signature_flags_and_constructor_sets_reject() {
        let (_, mut t, mut e, f, _) = setup();
        let ty = nat_induct_signature(&mut t, &e, f).unwrap().ty;
        let nat = t.constant(f, []);
        assert!(check_nat_induct_declaration(&mut t, &e, "Test.Nat.induct", nat, f, true).is_err());
        assert!(check_nat_induct_declaration(&mut t, &e, "Other.Nat.induct", ty, f, true).is_err());
        assert!(check_nat_induct_declaration(&mut t, &e, "Test.Nat.induct", ty, f, false).is_err());
        e.register_generated_constructor("Test.Nat.extra", nat, f)
            .unwrap();
        assert!(nat_induct_signature(&mut t, &e, f).is_err());
    }
    #[test]
    fn later_constructors_reject_without_environment_changes() {
        let (_, mut t, mut e, f, _) = setup();
        install(&mut t, &mut e, f);
        let nat = t.constant(f, []);
        let n = e.iter().count();
        assert!(e.register_constructor("Test.Nat.extra", nat, f).is_err());
        assert!(e
            .register_generated_constructor("Test.Nat.extra", nat, f)
            .is_err());
        assert_eq!(e.iter().count(), n);
    }
    #[test]
    fn nonempty_universe_arguments_reject_even_without_reduction() {
        let (mut l, mut t, mut e, f, cs) = setup();
        let g = install(&mut t, &mut e, f);
        for global in [f, cs[0], cs[1], g] {
            let x = t.constant(global, [l.zero()]);
            assert!(infer(&mut l, &mut t, &LocalContext::new(), &e, x).is_err());
        }
    }
    #[test]
    fn prior_opaque_and_theorem_uses_do_not_evade_monomorphism() {
        for theorem in [false, true] {
            let (l, mut t, mut e, f, cs) = setup();
            let nat = t.constant(f, []);
            let zero = t.constant(cs[0], [l.zero()]);
            if theorem {
                e.register_theorem("Test.Prior", nat, zero).unwrap();
            } else {
                e.register_definition(
                    "Test.Prior",
                    nat,
                    zero,
                    crate::DefinitionReducibility::Opaque,
                )
                .unwrap();
            }
            let ty = nat_induct_signature(&mut t, &e, f).unwrap().ty;
            assert!(
                check_nat_induct_declaration(&mut t, &e, "Test.Nat.induct", ty, f, true).is_err()
            );
        }
    }
    #[test]
    fn higher_universe_family_rejects() {
        let (mut l, mut t, mut e, _, _) = setup();
        let u = l.succ(l.zero());
        let sort = t.sort(u);
        let g = generate_mvp_inductive_declarations(
            &mut l,
            &mut t,
            &mut e,
            InductiveGenerationInput::new(MvpInductiveShape::Nat, "Test.Higher", vec![], sort),
        )
        .unwrap();
        assert!(nat_induct_signature(&mut t, &e, g.family.global).is_err());
    }
    #[test]
    fn partial_and_neutral_applications_remain_dependent() {
        let (mut l, mut t, mut e, f, cs) = setup();
        let g = install(&mut t, &mut e, f);
        let nat = t.constant(f, []);
        let sort = t.sort(l.zero());
        let z = t.constant(cs[0], []);
        let succ = t.constant(cs[1], []);
        let mut context = LocalContext::new();
        context.push_binder(t.pi(nat, sort));
        let p = t.var(0);
        context.push_binder(t.app(p, [z]));
        let p = t.var(2);
        let n = t.var(0);
        let ih = t.app(p, [n]);
        let p = t.var(3);
        let n = t.var(1);
        let sn = t.app(succ, [n]);
        let out = t.app(p, [sn]);
        let out = t.pi(ih, out);
        context.push_binder(t.pi(nat, out));
        context.push_binder(nat);
        let rec = t.constant(g, []);
        let p = t.var(3);
        let zero = t.var(2);
        let step = t.var(1);
        let n = t.var(0);
        let partial = t.app(rec, [p, zero, step]);
        infer(&mut l, &mut t, &context, &e, partial).unwrap();
        let a = t.app(rec, [p, zero, step, n]);
        let want = t.app(p, [n]);
        assert_eq!(infer(&mut l, &mut t, &context, &e, a).unwrap(), want);
        assert!(!definitionally_equal(&e, &mut t, a, zero).unwrap());
    }
}
