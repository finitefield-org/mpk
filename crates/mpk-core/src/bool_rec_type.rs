//! Canonical, type-level Boolean recursion returning Sort0.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId, TermArena,
    TermId, TermNode,
};

pub(crate) struct BoolRecTypeSignature {
    pub ty: TermId,
    pub constructors: [GlobalId; 2],
}

pub(crate) fn check_bool_rec_type_constant_levels(
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
        DeclarationKind::Inductive { .. } => has_bool_rec_type(env, global),
        DeclarationKind::Constructor { inductive, .. } => has_bool_rec_type(env, inductive),
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
        .with_detail("kind", "invalid_bool_rec_type_interface")
        .with_detail("reason", detail)
}

pub(crate) fn bool_rec_type_signature(
    terms: &mut TermArena,
    env: &Environment,
    family: GlobalId,
) -> Result<BoolRecTypeSignature, CoreError> {
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
    let boolean = terms.constant(family, []);
    for (constructor, suffix) in [(*no, "false"), (*yes, "true")] {
        if constructor.name().as_str() != format!("{family_name}.{suffix}")
            || !constructor.kind().is_generated_constructor()
            || constructor.ty() != boolean
        {
            return Err(invalid("noncanonical_constructor"));
        }
    }
    let no_global = no.global();
    let yes_global = yes.global();
    // Fixed Sort0 type selection; no universe-polymorphic rule or legacy change.
    let result = terms.pi(boolean, sort);
    let result = terms.pi(sort, result);
    let ty = terms.pi(sort, result);
    Ok(BoolRecTypeSignature {
        ty,
        constructors: [no_global, yes_global],
    })
}

/// Validate the reserved `<family>.recType` interface before registering it.
/// Other recursor names retain their existing declaration checking behavior.
pub fn check_bool_rec_type_declaration(
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
        if name.ends_with(".recType") {
            return Err(invalid("reserved_name_for_different_family"));
        }
        return Ok(());
    }
    if !generated {
        return Err(invalid("nongenerated_type_recursion"));
    }
    let signature = bool_rec_type_signature(terms, env, family)?;
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

pub(crate) fn has_bool_rec_type(env: &Environment, family: GlobalId) -> bool {
    let Some(declaration) = env.lookup(family) else {
        return false;
    };
    let name = format!("{}.recType", declaration.name().as_str());
    env.lookup_by_name(name).ok().flatten().is_some_and(
        |d| matches!(d.kind(), DeclarationKind::Recursor { inductive, .. } if inductive == family),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        definitionally_equal, generate_mvp_inductive_declarations, infer,
        reduce_generated_recursor_iota_with_fuel, InductiveGenerationInput, LevelArena,
        LocalContext, MvpInductiveShape,
    };

    fn setup() -> (LevelArena, TermArena, Environment, GlobalId, [GlobalId; 2]) {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let sort = terms.sort(levels.zero());
        let generated = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Test.Bool", vec![], sort),
        )
        .unwrap();
        (
            levels,
            terms,
            env,
            generated.family.global,
            [
                generated.constructors[0].global,
                generated.constructors[1].global,
            ],
        )
    }

    fn install(terms: &mut TermArena, env: &mut Environment, family: GlobalId) -> GlobalId {
        let ty = bool_rec_type_signature(terms, env, family).unwrap().ty;
        check_bool_rec_type_declaration(terms, env, "Test.Bool.recType", ty, family, true).unwrap();
        env.register_generated_recursor("Test.Bool.recType", ty, family)
            .unwrap()
    }

    #[test]
    fn selects_distinct_sort_zero_types_and_retains_legacy_declarations() {
        let (mut levels, mut terms, mut env, family, constructors) = setup();
        let previous = env
            .iter()
            .map(|d| (d.name().clone(), d.kind()))
            .collect::<Vec<_>>();
        let recursor = install(&mut terms, &mut env, family);
        assert_eq!(
            previous,
            env.iter()
                .take(previous.len())
                .map(|d| (d.name().clone(), d.kind()))
                .collect::<Vec<_>>()
        );
        let boolean = terms.constant(family, []);
        let function = terms.pi(boolean, boolean);
        let recursor = terms.constant(recursor, []);
        let sort = terms.sort(levels.zero());
        for (constructor, expected) in [(constructors[0], boolean), (constructors[1], function)] {
            let major = terms.constant(constructor, []);
            let value = terms.app(recursor, [boolean, function, major]);
            assert_eq!(
                infer(&mut levels, &mut terms, &LocalContext::new(), &env, value).unwrap(),
                sort
            );
            assert!(definitionally_equal(&env, &mut terms, value, expected).unwrap());
            assert_eq!(
                reduce_generated_recursor_iota_with_fuel(&env, &mut terms, value, 0)
                    .unwrap_err()
                    .code(),
                CoreErrorCode::FuelExhausted
            );
        }
    }

    #[test]
    fn partial_and_neutral_type_selection_does_not_choose_a_branch() {
        let (mut levels, mut terms, mut env, family, _) = setup();
        let recursor = install(&mut terms, &mut env, family);
        let boolean = terms.constant(family, []);
        let sort = terms.sort(levels.zero());
        let mut context = LocalContext::new();
        context.push_binder(sort);
        context.push_binder(sort);
        context.push_binder(boolean);
        let function = terms.constant(recursor, []);
        let a = terms.var(2);
        let b = terms.var(1);
        let tag = terms.var(0);
        let partial = terms.app(function, [a]);
        infer(&mut levels, &mut terms, &context, &env, partial).unwrap();
        let result = terms.app(function, [a, b, tag]);
        assert_eq!(
            infer(&mut levels, &mut terms, &context, &env, result).unwrap(),
            sort
        );
        assert!(!definitionally_equal(&env, &mut terms, result, a).unwrap());
        assert!(!definitionally_equal(&env, &mut terms, result, b).unwrap());
    }

    #[test]
    fn rejects_wrong_or_reserved_aliases_and_later_constructors_before_mutation() {
        let (_, mut terms, mut env, family, _) = setup();
        let ty = bool_rec_type_signature(&mut terms, &env, family)
            .unwrap()
            .ty;
        let boolean = terms.constant(family, []);
        for (name, candidate, generated) in [
            ("Test.Bool.recType", boolean, true),
            ("Test.Bool.recType", ty, false),
            ("Other.Bool.recType", ty, true),
        ] {
            assert!(check_bool_rec_type_declaration(
                &mut terms, &env, name, candidate, family, generated
            )
            .is_err());
        }
        install(&mut terms, &mut env, family);
        let before = env.iter().count();
        assert!(env
            .register_constructor("Test.Bool.extra", boolean, family)
            .is_err());
        assert!(env
            .register_generated_constructor("Test.Bool.extra", boolean, family)
            .is_err());
        assert_eq!(before, env.iter().count());
        assert!(env.resolve("Test.Bool.extra").unwrap().is_none());
    }

    #[test]
    fn rejects_prior_actual_universe_uses_but_ignores_unused_term_nodes() {
        for prior in ["unused", "type", "opaque", "proof"] {
            let (levels, mut terms, mut env, family, constructors) = setup();
            let boolean = terms.constant(family, []);
            let value = terms.constant(constructors[0], [levels.zero()]);
            match prior {
                "type" => {
                    let ty = terms.constant(family, [levels.zero()]);
                    env.register_theorem("Test.Prior", ty, value).unwrap();
                }
                "opaque" => {
                    env.register_definition(
                        "Test.Prior",
                        boolean,
                        value,
                        crate::DefinitionReducibility::Opaque,
                    )
                    .unwrap();
                }
                "proof" => {
                    env.register_theorem("Test.Prior", boolean, value).unwrap();
                }
                _ => {}
            }
            let ty = bool_rec_type_signature(&mut terms, &env, family)
                .unwrap()
                .ty;
            assert_eq!(
                check_bool_rec_type_declaration(
                    &mut terms,
                    &env,
                    "Test.Bool.recType",
                    ty,
                    family,
                    true
                )
                .is_ok(),
                prior == "unused"
            );
        }
    }

    #[test]
    fn rejects_higher_family_and_every_actual_interface_universe_argument() {
        let (mut levels, mut terms, mut env, family, constructors) = setup();
        let recursor = install(&mut terms, &mut env, family);
        for global in [family, constructors[0], constructors[1], recursor] {
            let value = terms.constant(global, [levels.zero()]);
            assert!(infer(&mut levels, &mut terms, &LocalContext::new(), &env, value).is_err());
        }
        let upper = levels.succ(levels.zero());
        let sort = terms.sort(upper);
        let higher = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Test.Higher", vec![], sort),
        )
        .unwrap();
        assert!(bool_rec_type_signature(&mut terms, &env, higher.family.global).is_err());
    }

    #[test]
    fn rejects_noncanonical_constructor_metadata_without_needing_cases() {
        for mutation in ["order", "extra", "nongenerated"] {
            let levels = LevelArena::new();
            let mut terms = TermArena::new();
            let mut env = Environment::new();
            let sort = terms.sort(levels.zero());
            let family = env.register_inductive("Test.Bool", sort).unwrap();
            let boolean = terms.constant(family, []);
            for suffix in if mutation == "order" {
                ["true", "false"]
            } else {
                ["false", "true"]
            } {
                if mutation == "nongenerated" && suffix == "true" {
                    env.register_constructor(format!("Test.Bool.{suffix}"), boolean, family)
                        .unwrap();
                } else {
                    env.register_generated_constructor(
                        format!("Test.Bool.{suffix}"),
                        boolean,
                        family,
                    )
                    .unwrap();
                }
            }
            if mutation == "extra" {
                env.register_generated_constructor("Test.Bool.extra", boolean, family)
                    .unwrap();
            }
            assert!(
                bool_rec_type_signature(&mut terms, &env, family).is_err(),
                "{mutation}"
            );
        }
    }
}
