//! Canonical, dependent Boolean elimination into Sort0.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId, TermArena,
    TermId, TermNode,
};

pub(crate) struct BoolCasesSignature {
    pub ty: TermId,
    pub constructors: [GlobalId; 2],
}

fn invalid(detail: &str) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidDeclaration, CoreLocation::root())
        .with_detail("kind", "invalid_bool_cases_interface")
        .with_detail("reason", detail)
}

pub(crate) fn bool_cases_signature(
    terms: &mut TermArena,
    env: &Environment,
    family: GlobalId,
) -> Result<BoolCasesSignature, CoreError> {
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
    let no = terms.constant(no_global, []);
    let yes = terms.constant(yes_global, []);
    let motive_type = terms.pi(boolean, sort);
    let motive = terms.var(0);
    let no_type = terms.app(motive, [no]);
    let motive = terms.var(1);
    let yes_type = terms.app(motive, [yes]);
    let motive = terms.var(3);
    let major = terms.var(0);
    let result = terms.app(motive, [major]);
    let major_to_result = terms.pi(boolean, result);
    let yes_to_major = terms.pi(yes_type, major_to_result);
    let no_to_yes = terms.pi(no_type, yes_to_major);
    let ty = terms.pi(motive_type, no_to_yes);
    Ok(BoolCasesSignature {
        ty,
        constructors: [no_global, yes_global],
    })
}

/// Validate the reserved `<family>.cases` interface before registering it.
/// Other recursor names retain their existing declaration checking behavior.
pub fn check_bool_cases_declaration(
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
    if name != format!("{}.cases", declaration.name().as_str()) {
        return Ok(());
    }
    if !generated {
        return Err(invalid("nongenerated_cases"));
    }
    if ty != bool_cases_signature(terms, env, family)?.ty {
        return Err(invalid("noncanonical_eliminator_type"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        definitionally_equal, generate_bool_cases_declaration, generate_mvp_inductive_declarations,
        infer, InductiveGenerationInput, LevelArena, LocalContext, MvpInductiveShape,
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
        let constructors = [
            generated.constructors[0].global,
            generated.constructors[1].global,
        ];
        (levels, terms, env, generated.family.global, constructors)
    }

    #[test]
    fn bool_cases_checks_both_equations_and_preserves_legacy_recursor() {
        let (mut levels, mut terms, mut env, family, constructors) = setup();
        let previous = env
            .iter()
            .map(|d| (d.name().clone(), d.kind()))
            .collect::<Vec<_>>();
        let cases =
            generate_bool_cases_declaration(&mut levels, &mut terms, &mut env, family).unwrap();
        assert_eq!(
            previous,
            env.iter()
                .take(previous.len())
                .map(|d| (d.name().clone(), d.kind()))
                .collect::<Vec<_>>()
        );
        let boolean = terms.constant(family, []);
        let no = terms.constant(constructors[0], []);
        let yes = terms.constant(constructors[1], []);
        let motive = terms.lam(boolean, boolean);
        let function = terms.constant(cases.global, []);
        for (major, wanted) in [(no, no), (yes, yes)] {
            let application = terms.app(function, [motive, no, yes, major]);
            let inferred = infer(
                &mut levels,
                &mut terms,
                &LocalContext::new(),
                &env,
                application,
            )
            .unwrap();
            assert!(definitionally_equal(&env, &mut terms, inferred, boolean).unwrap());
            assert!(definitionally_equal(&env, &mut terms, application, wanted).unwrap());
        }
    }

    #[test]
    fn bool_cases_infers_open_dependent_result_without_reducing_neutral_major() {
        let (mut levels, mut terms, mut env, family, constructors) = setup();
        let cases =
            generate_bool_cases_declaration(&mut levels, &mut terms, &mut env, family).unwrap();
        let boolean = terms.constant(family, []);
        let sort = terms.sort(levels.zero());
        let mut context = LocalContext::new();
        context.push_binder(terms.pi(boolean, sort));
        let p = terms.var(0);
        let no = terms.constant(constructors[0], []);
        context.push_binder(terms.app(p, [no]));
        let p = terms.var(1);
        let yes = terms.constant(constructors[1], []);
        context.push_binder(terms.app(p, [yes]));
        context.push_binder(boolean);
        let p = terms.var(3);
        let no_proof = terms.var(2);
        let yes_proof = terms.var(1);
        let b = terms.var(0);
        let function = terms.constant(cases.global, []);
        let application = terms.app(function, [p, no_proof, yes_proof, b]);
        let wanted = terms.app(p, [b]);
        assert_eq!(
            infer(&mut levels, &mut terms, &context, &env, application).unwrap(),
            wanted
        );
        assert!(!definitionally_equal(&env, &mut terms, application, no_proof).unwrap());
        assert!(!definitionally_equal(&env, &mut terms, application, yes_proof).unwrap());
    }

    #[test]
    fn bool_cases_rejects_wrong_type_nongenerated_and_extra_constructor() {
        let (_, mut terms, mut env, family, _) = setup();
        let ty = bool_cases_signature(&mut terms, &env, family).unwrap().ty;
        check_bool_cases_declaration(&mut terms, &env, "Test.Bool.cases", ty, family, true)
            .unwrap();
        assert!(check_bool_cases_declaration(
            &mut terms,
            &env,
            "Test.Bool.cases",
            ty,
            family,
            false
        )
        .is_err());
        let boolean = terms.constant(family, []);
        assert!(check_bool_cases_declaration(
            &mut terms,
            &env,
            "Test.Bool.cases",
            boolean,
            family,
            true
        )
        .is_err());
        env.register_constructor("Test.Bool.extra", boolean, family)
            .unwrap();
        assert!(check_bool_cases_declaration(
            &mut terms,
            &env,
            "Test.Bool.cases",
            ty,
            family,
            true
        )
        .is_err());
    }

    #[test]
    fn bool_cases_rejects_higher_universe_family() {
        let (mut levels, mut terms, mut env, _, _) = setup();
        let upper = levels.succ(levels.zero());
        let sort = terms.sort(upper);
        let family = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Test.Higher", vec![], sort),
        )
        .unwrap()
        .family
        .global;
        assert!(
            generate_bool_cases_declaration(&mut levels, &mut terms, &mut env, family).is_err()
        );
    }
}
