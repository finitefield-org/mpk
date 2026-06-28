//! Conservative positivity checks for MVP inductive declarations.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, Environment, ExportedInductive,
    ExportedInductiveDeclaration, GlobalId, MvpInductiveShape, RegisteredInductive, TermArena,
    TermId, TermNode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PositivityErrorKind {
    ConstructorCountMismatch,
    ConstructorResultNotFamily,
    ConstructorShapeMismatch,
    NegativeRecursiveOccurrence,
    NestedRecursiveOccurrence,
    UnknownFunctorOccurrence,
    UnsupportedTerm,
}

impl PositivityErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConstructorCountMismatch => "constructor_count_mismatch",
            Self::ConstructorResultNotFamily => "constructor_result_not_family",
            Self::ConstructorShapeMismatch => "constructor_shape_mismatch",
            Self::NegativeRecursiveOccurrence => "negative_recursive_occurrence",
            Self::NestedRecursiveOccurrence => "nested_recursive_occurrence",
            Self::UnknownFunctorOccurrence => "unknown_functor_occurrence",
            Self::UnsupportedTerm => "unsupported_positivity_term",
        }
    }
}

pub fn check_mvp_positivity(
    terms: &TermArena,
    env: &Environment,
    registered: &RegisteredInductive,
) -> Result<(), CoreError> {
    let exported = registered.export(env)?;
    validate_constructor_count(&exported)?;

    for (index, constructor) in exported.constructors.iter().enumerate() {
        check_constructor_type(terms, &exported, constructor, index)?;
    }

    Ok(())
}

fn validate_constructor_count(exported: &ExportedInductive) -> Result<(), CoreError> {
    let expected = constructor_count(exported.shape);
    if exported.constructors.len() == expected {
        return Ok(());
    }

    Err(CoreError::new(
        CoreErrorCode::InvalidDeclaration,
        positivity_location().with_field("constructors"),
    )
    .with_detail(
        "kind",
        PositivityErrorKind::ConstructorCountMismatch.as_str(),
    )
    .with_detail("shape", exported.shape.as_str())
    .with_detail("expected", expected.to_string())
    .with_detail("actual", exported.constructors.len().to_string()))
}

fn constructor_count(shape: MvpInductiveShape) -> usize {
    match shape {
        MvpInductiveShape::Bool | MvpInductiveShape::Nat => 2,
        MvpInductiveShape::Eq => 1,
    }
}

fn check_constructor_type(
    terms: &TermArena,
    exported: &ExportedInductive,
    constructor: &ExportedInductiveDeclaration,
    constructor_index: usize,
) -> Result<(), CoreError> {
    let context = PositivityContext {
        shape: exported.shape,
        family: exported.family.global,
        family_name: exported.family.name.as_str(),
        constructor_name: constructor.name.as_str(),
        constructor_index,
    };
    let view = peel_constructor_type(terms, constructor.ty, constructor_index)?;

    for (argument_index, argument) in view.arguments.iter().copied().enumerate() {
        check_constructor_argument(
            terms,
            argument,
            &context,
            constructor_type_location(constructor_index)?
                .with_field("arguments")
                .with_index(index_as_u32(argument_index)?),
        )?;
    }

    check_constructor_result(
        terms,
        view.result,
        &context,
        constructor_type_location(constructor_index)?.with_field("result"),
    )?;
    validate_documented_shape(terms, &context, &view)
}

fn peel_constructor_type(
    terms: &TermArena,
    constructor_type: TermId,
    constructor_index: usize,
) -> Result<ConstructorTypeView, CoreError> {
    let mut current = constructor_type;
    let mut arguments = Vec::new();

    loop {
        validate_term_reference(
            terms,
            current,
            constructor_type_location(constructor_index)?,
        )?;
        match terms.node(current).clone() {
            TermNode::Pi { ty, body } => {
                arguments.push(ty);
                current = body;
            }
            _ => {
                return Ok(ConstructorTypeView {
                    constructor_type,
                    arguments,
                    result: current,
                });
            }
        }
    }
}

fn validate_documented_shape(
    terms: &TermArena,
    context: &PositivityContext<'_>,
    view: &ConstructorTypeView,
) -> Result<(), CoreError> {
    match context.shape {
        MvpInductiveShape::Bool if !view.arguments.is_empty() => Err(shape_mismatch_error(
            context,
            view.constructor_type,
            "nullary_bool_constructor",
            format!("{} top_level_arguments", view.arguments.len()),
        )),
        MvpInductiveShape::Bool => Ok(()),
        MvpInductiveShape::Nat if context.constructor_index == 0 && !view.arguments.is_empty() => {
            Err(shape_mismatch_error(
                context,
                view.constructor_type,
                "nullary_nat_zero_constructor",
                format!("{} top_level_arguments", view.arguments.len()),
            ))
        }
        MvpInductiveShape::Nat if context.constructor_index == 1 => {
            if view.arguments.len() != 1 {
                return Err(shape_mismatch_error(
                    context,
                    view.constructor_type,
                    "one_direct_recursive_nat_succ_argument",
                    format!("{} top_level_arguments", view.arguments.len()),
                ));
            }
            let argument = view.arguments[0];
            let location = constructor_type_location(context.constructor_index)?
                .with_field("arguments")
                .with_index(0)
                .with_field("documented_shape");
            if is_direct_family_application(terms, argument, context, location)? {
                return Ok(());
            }

            Err(shape_mismatch_error(
                context,
                argument,
                "one_direct_recursive_nat_succ_argument",
                "non_direct_recursive_argument",
            ))
        }
        MvpInductiveShape::Nat => Ok(()),
        MvpInductiveShape::Eq => {
            for (argument_index, argument) in view.arguments.iter().copied().enumerate() {
                let location = constructor_type_location(context.constructor_index)?
                    .with_field("arguments")
                    .with_index(index_as_u32(argument_index)?)
                    .with_field("documented_shape");
                if contains_family(terms, argument, context.family, location)? {
                    return Err(shape_mismatch_error(
                        context,
                        argument,
                        "non_recursive_eq_refl_argument",
                        "recursive_argument",
                    ));
                }
            }
            Ok(())
        }
    }
}

fn check_constructor_argument(
    terms: &TermArena,
    term: TermId,
    context: &PositivityContext<'_>,
    location: CoreLocation,
) -> Result<(), CoreError> {
    if !contains_family(terms, term, context.family, location.clone())? {
        return Ok(());
    }

    check_positive_position(terms, term, context, location)
}

fn check_constructor_result(
    terms: &TermArena,
    term: TermId,
    context: &PositivityContext<'_>,
    location: CoreLocation,
) -> Result<(), CoreError> {
    if is_direct_family_application(terms, term, context, location.clone())? {
        return Ok(());
    }

    Err(positivity_error(
        PositivityErrorKind::ConstructorResultNotFamily,
        location,
        context,
        term,
    ))
}

fn check_positive_position(
    terms: &TermArena,
    term: TermId,
    context: &PositivityContext<'_>,
    location: CoreLocation,
) -> Result<(), CoreError> {
    validate_term_reference(terms, term, location.clone())?;
    if is_direct_family_application(terms, term, context, location.clone())? {
        return Ok(());
    }

    match terms.node(term).clone() {
        TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => Ok(()),
        TermNode::App {
            function,
            arguments,
        } => {
            if contains_family(
                terms,
                function,
                context.family,
                location.clone().with_field("function"),
            )? {
                return Err(positivity_error(
                    PositivityErrorKind::NestedRecursiveOccurrence,
                    location.with_field("function"),
                    context,
                    function,
                ));
            }

            for (index, argument) in arguments.iter().copied().enumerate() {
                let argument_location = location
                    .clone()
                    .with_field("arguments")
                    .with_index(index_as_u32(index)?);
                if contains_family(terms, argument, context.family, argument_location.clone())? {
                    return Err(positivity_error(
                        PositivityErrorKind::UnknownFunctorOccurrence,
                        argument_location,
                        context,
                        argument,
                    ));
                }
            }

            Ok(())
        }
        TermNode::Pi { ty, body } => {
            let domain_location = location.clone().with_field("domain");
            if contains_family(terms, ty, context.family, domain_location.clone())? {
                return Err(positivity_error(
                    PositivityErrorKind::NegativeRecursiveOccurrence,
                    domain_location,
                    context,
                    ty,
                ));
            }

            check_positive_position(terms, body, context, location.with_field("body"))
        }
        TermNode::Lam { .. } | TermNode::Let { .. } => Err(positivity_error(
            PositivityErrorKind::UnsupportedTerm,
            location,
            context,
            term,
        )),
    }
}

fn is_direct_family_application(
    terms: &TermArena,
    term: TermId,
    context: &PositivityContext<'_>,
    location: CoreLocation,
) -> Result<bool, CoreError> {
    validate_term_reference(terms, term, location.clone())?;
    match terms.node(term).clone() {
        TermNode::Const { global, .. } if global == context.family => Ok(true),
        TermNode::App {
            function,
            arguments,
        } if is_family_head(terms, function, context.family, location.clone())? => {
            for (index, argument) in arguments.iter().copied().enumerate() {
                let argument_location = location
                    .clone()
                    .with_field("family_arguments")
                    .with_index(index_as_u32(index)?);
                if contains_family(terms, argument, context.family, argument_location.clone())? {
                    return Err(positivity_error(
                        PositivityErrorKind::NestedRecursiveOccurrence,
                        argument_location,
                        context,
                        argument,
                    ));
                }
            }

            Ok(true)
        }
        _ => Ok(false),
    }
}

fn is_family_head(
    terms: &TermArena,
    term: TermId,
    family: GlobalId,
    location: CoreLocation,
) -> Result<bool, CoreError> {
    validate_term_reference(terms, term, location)?;
    Ok(matches!(
        terms.node(term),
        TermNode::Const { global, .. } if *global == family
    ))
}

fn contains_family(
    terms: &TermArena,
    term: TermId,
    family: GlobalId,
    location: CoreLocation,
) -> Result<bool, CoreError> {
    validate_term_reference(terms, term, location.clone())?;
    match terms.node(term).clone() {
        TermNode::Sort(_) | TermNode::Var(_) => Ok(false),
        TermNode::Const { global, .. } => Ok(global == family),
        TermNode::App {
            function,
            arguments,
        } => {
            if contains_family(
                terms,
                function,
                family,
                location.clone().with_field("function"),
            )? {
                return Ok(true);
            }
            for (index, argument) in arguments.into_iter().enumerate() {
                if contains_family(
                    terms,
                    argument,
                    family,
                    location
                        .clone()
                        .with_field("arguments")
                        .with_index(index_as_u32(index)?),
                )? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            Ok(
                contains_family(terms, ty, family, location.clone().with_field("type"))?
                    || contains_family(terms, body, family, location.with_field("body"))?,
            )
        }
        TermNode::Let { ty, value, body } => {
            Ok(
                contains_family(terms, ty, family, location.clone().with_field("type"))?
                    || contains_family(terms, value, family, location.clone().with_field("value"))?
                    || contains_family(terms, body, family, location.with_field("body"))?,
            )
        }
    }
}

fn validate_term_reference(
    terms: &TermArena,
    term: TermId,
    location: CoreLocation,
) -> Result<(), CoreError> {
    if term.index() < terms.len() {
        return Ok(());
    }

    Err(
        CoreError::new(CoreErrorCode::InvalidTermReference, location)
            .with_detail("kind", "unknown_term")
            .with_detail("term", term.index().to_string())
            .with_detail("arena_len", terms.len().to_string()),
    )
}

struct PositivityContext<'a> {
    shape: MvpInductiveShape,
    family: GlobalId,
    family_name: &'a str,
    constructor_name: &'a str,
    constructor_index: usize,
}

struct ConstructorTypeView {
    constructor_type: TermId,
    arguments: Vec<TermId>,
    result: TermId,
}

fn positivity_error(
    kind: PositivityErrorKind,
    location: CoreLocation,
    context: &PositivityContext<'_>,
    term: TermId,
) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidDeclaration, location)
        .with_detail("kind", kind.as_str())
        .with_detail("shape", context.shape.as_str())
        .with_detail("family", context.family_name)
        .with_detail("constructor", context.constructor_name)
        .with_detail("constructor_index", context.constructor_index.to_string())
        .with_detail("term", term.index().to_string())
}

fn shape_mismatch_error(
    context: &PositivityContext<'_>,
    term: TermId,
    expected: &'static str,
    actual: impl Into<String>,
) -> CoreError {
    let location = match constructor_type_location(context.constructor_index) {
        Ok(location) => location,
        Err(error) => return error,
    };

    positivity_error(
        PositivityErrorKind::ConstructorShapeMismatch,
        location,
        context,
        term,
    )
    .with_detail("expected", expected)
    .with_detail("actual", actual)
}

fn positivity_location() -> CoreLocation {
    CoreLocation::root().with_field("positivity")
}

fn constructor_type_location(constructor_index: usize) -> Result<CoreLocation, CoreError> {
    Ok(positivity_location()
        .with_field("constructors")
        .with_index(index_as_u32(constructor_index)?)
        .with_field("type"))
}

fn index_as_u32(index: usize) -> Result<u32, CoreError> {
    u32::try_from(index).map_err(|_| {
        CoreError::new(CoreErrorCode::InternalInvariant, positivity_location())
            .with_detail("kind", "index_overflow")
            .with_detail("index", index.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::{check_mvp_positivity, PositivityErrorKind};

    use std::fs;
    use std::path::Path;

    use crate::{
        CoreError, Environment, LevelArena, MvpInductiveShape, RegisteredInductive, TermArena,
        TermId,
    };

    const CORE_INDUCTIVE_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/core-inductive");

    #[derive(Debug)]
    struct PositivityFixture {
        id: String,
        expected: String,
        expected_error_kind: Option<String>,
    }

    fn read_core_inductive_fixtures() -> Vec<PositivityFixture> {
        let mut entries = fs::read_dir(CORE_INDUCTIVE_FIXTURE_DIR)
            .expect("core-inductive fixture directory exists")
            .map(|entry| entry.expect("fixture dir entry is readable").path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "fixture")
            })
            .collect::<Vec<_>>();
        entries.sort();

        entries
            .into_iter()
            .map(|path| parse_core_inductive_fixture(&path))
            .collect()
    }

    fn parse_core_inductive_fixture(path: &Path) -> PositivityFixture {
        let contents = fs::read_to_string(path).expect("fixture is readable");
        let mut id = None;
        let mut expected = None;
        let mut expected_error_kind = None;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once(':').unwrap_or_else(|| {
                panic!("fixture line must be `key: value` in {}", path.display())
            });
            let value = value.trim().to_owned();
            match key.trim() {
                "id" => id = Some(value),
                "expected" => expected = Some(value),
                "expected_error_kind" => expected_error_kind = Some(value),
                _ => {}
            }
        }

        PositivityFixture {
            id: id.unwrap_or_else(|| panic!("fixture id missing in {}", path.display())),
            expected: expected.unwrap_or_else(|| {
                panic!("fixture expected verdict missing in {}", path.display())
            }),
            expected_error_kind,
        }
    }

    fn run_core_inductive_fixture(fixture: &PositivityFixture) -> Result<(), CoreError> {
        match fixture.id.as_str() {
            "bool-positive" => run_bool_positive_fixture(),
            "nat-positive" => run_nat_positive_fixture(),
            "eq-positive" => run_eq_positive_fixture(),
            "bool-recursive-argument" => run_bool_recursive_argument_fixture(),
            "nat-negative-function" => run_nat_negative_function_fixture(),
            "nat-nested-unknown-functor" => run_nat_nested_unknown_functor_fixture(),
            "nat-bad-result" => run_nat_bad_result_fixture(),
            id => panic!("unknown core-inductive fixture id `{id}`"),
        }
    }

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param");
        terms.sort(level)
    }

    fn register_family(
        env: &mut Environment,
        terms: &mut TermArena,
        ty: TermId,
        name: &str,
        levels: impl IntoIterator<Item = crate::LevelId>,
    ) -> (crate::GlobalId, TermId) {
        let family = env.register_inductive(name, ty).expect("valid family");
        let family_term = terms.constant(family, levels);
        (family, family_term)
    }

    fn register_constructor(
        env: &mut Environment,
        name: &str,
        ty: TermId,
        family: crate::GlobalId,
    ) -> crate::GlobalId {
        env.register_constructor(name, ty, family)
            .expect("valid constructor")
    }

    fn register_recursor(
        env: &mut Environment,
        name: &str,
        ty: TermId,
        family: crate::GlobalId,
    ) -> crate::GlobalId {
        env.register_recursor(name, ty, family)
            .expect("valid recursor")
    }

    fn run_bool_positive_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.Bool", []);
        let false_ctor = register_constructor(&mut env, "Std.Bool.false", family_term, family);
        let true_ctor = register_constructor(&mut env, "Std.Bool.true", family_term, family);
        let recursor = register_recursor(&mut env, "Std.Bool.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Bool,
            universe_params: Vec::new(),
            family,
            constructors: vec![false_ctor, true_ctor],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_nat_positive_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.Nat", []);
        let zero = register_constructor(&mut env, "Std.Nat.zero", family_term, family);
        let succ_ty = terms.pi(family_term, family_term);
        let succ = register_constructor(&mut env, "Std.Nat.succ", succ_ty, family);
        let recursor = register_recursor(&mut env, "Std.Nat.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Nat,
            universe_params: Vec::new(),
            family,
            constructors: vec![zero, succ],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_eq_positive_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let universe = levels.parse_param("u").expect("valid level param");
        let ty = terms.sort(universe);
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.Eq", [universe]);
        let refl = register_constructor(&mut env, "Std.Eq.refl", family_term, family);
        let recursor = register_recursor(&mut env, "Std.Eq.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Eq,
            universe_params: vec![universe],
            family,
            constructors: vec![refl],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_bool_recursive_argument_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.BadBool", []);
        let bad_false_ty = terms.pi(family_term, family_term);
        let false_ctor = register_constructor(&mut env, "Std.BadBool.false", bad_false_ty, family);
        let true_ctor = register_constructor(&mut env, "Std.BadBool.true", family_term, family);
        let recursor = register_recursor(&mut env, "Std.BadBool.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Bool,
            universe_params: Vec::new(),
            family,
            constructors: vec![false_ctor, true_ctor],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_nat_negative_function_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.BadNat", []);
        let zero = register_constructor(&mut env, "Std.BadNat.zero", family_term, family);
        let non_family = terms.var(0);
        let negative_argument = terms.pi(family_term, non_family);
        let bad_succ_ty = terms.pi(negative_argument, family_term);
        let bad_succ = register_constructor(&mut env, "Std.BadNat.bad", bad_succ_ty, family);
        let recursor = register_recursor(&mut env, "Std.BadNat.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Nat,
            universe_params: Vec::new(),
            family,
            constructors: vec![zero, bad_succ],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_nat_nested_unknown_functor_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let wrapper = env
            .register_axiom("Std.Wrapper", ty)
            .expect("valid wrapper axiom");
        let wrapper_term = terms.constant(wrapper, []);
        let (family, family_term) = register_family(&mut env, &mut terms, ty, "Std.NestedNat", []);
        let zero = register_constructor(&mut env, "Std.NestedNat.zero", family_term, family);
        let nested_argument = terms.app(wrapper_term, [family_term]);
        let bad_succ_ty = terms.pi(nested_argument, family_term);
        let bad_succ = register_constructor(&mut env, "Std.NestedNat.bad", bad_succ_ty, family);
        let recursor = register_recursor(&mut env, "Std.NestedNat.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Nat,
            universe_params: Vec::new(),
            family,
            constructors: vec![zero, bad_succ],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    fn run_nat_bad_result_fixture() -> Result<(), CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let (family, family_term) =
            register_family(&mut env, &mut terms, ty, "Std.NoResultNat", []);
        let zero = register_constructor(&mut env, "Std.NoResultNat.zero", family_term, family);
        let bad_result = terms.var(0);
        let bad_succ = register_constructor(&mut env, "Std.NoResultNat.bad", bad_result, family);
        let recursor = register_recursor(&mut env, "Std.NoResultNat.rec", family_term, family);
        let registered = RegisteredInductive {
            shape: MvpInductiveShape::Nat,
            universe_params: Vec::new(),
            family,
            constructors: vec![zero, bad_succ],
            recursor,
        };

        check_mvp_positivity(&terms, &env, &registered)
    }

    #[test]
    fn mvp_inductive_positivity_accepts_documented_shapes() {
        run_bool_positive_fixture().expect("Bool is accepted");
        run_nat_positive_fixture().expect("Nat is accepted");
        run_eq_positive_fixture().expect("Eq is accepted");
    }

    #[test]
    fn mvp_inductive_positivity_rejects_negative_occurrences() {
        let error = run_nat_negative_function_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(PositivityErrorKind::NegativeRecursiveOccurrence.as_str())
        );
    }

    #[test]
    fn mvp_inductive_positivity_rejects_unknown_nested_functors() {
        let error = run_nat_nested_unknown_functor_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(PositivityErrorKind::UnknownFunctorOccurrence.as_str())
        );
    }

    #[test]
    fn mvp_inductive_positivity_rejects_non_family_results() {
        let error = run_nat_bad_result_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(PositivityErrorKind::ConstructorResultNotFamily.as_str())
        );
    }

    #[test]
    fn mvp_inductive_positivity_rejects_undocumented_shape_patterns() {
        let error = run_bool_recursive_argument_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(PositivityErrorKind::ConstructorShapeMismatch.as_str())
        );
    }

    #[test]
    fn core_inductive_fixtures_are_deterministic() {
        let fixtures = read_core_inductive_fixtures();
        assert!(!fixtures.is_empty());

        for fixture in fixtures {
            let result = run_core_inductive_fixture(&fixture);
            match fixture.expected.as_str() {
                "accept" => result.unwrap_or_else(|error| {
                    panic!(
                        "fixture `{}` should accept but rejected with {}",
                        fixture.id,
                        error.to_deterministic_json()
                    )
                }),
                "reject" => {
                    let error = match result {
                        Ok(()) => panic!("fixture `{}` should reject", fixture.id),
                        Err(error) => error,
                    };
                    let expected_error_kind =
                        fixture.expected_error_kind.as_deref().unwrap_or_else(|| {
                            panic!("fixture `{}` missing expected_error_kind", fixture.id)
                        });
                    assert_eq!(
                        error.details().get("kind").map(String::as_str),
                        Some(expected_error_kind),
                        "fixture `{}` rejected with {}",
                        fixture.id,
                        error.to_deterministic_json()
                    );
                }
                other => panic!("fixture `{}` has unknown expected `{other}`", fixture.id),
            }
        }
    }
}
