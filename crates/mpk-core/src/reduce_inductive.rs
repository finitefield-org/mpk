//! Iota reduction for generated MVP inductive recursors.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId,
    MvpInductiveShape, TermArena, TermId, TermNode, DEFAULT_WHNF_FUEL,
};

pub const DEFAULT_IOTA_FUEL: u32 = DEFAULT_WHNF_FUEL;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum IotaReductionErrorKind {
    NoIotaRedex,
    NonGeneratedRecursor,
    MalformedGeneratedRecursor,
    UnknownRecursorEquation,
}

impl IotaReductionErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoIotaRedex => "no_iota_redex",
            Self::NonGeneratedRecursor => "non_generated_recursor_iota",
            Self::MalformedGeneratedRecursor => "malformed_generated_recursor",
            Self::UnknownRecursorEquation => "unknown_recursor_equation",
        }
    }
}

pub fn reduce_generated_recursor_iota(
    env: &Environment,
    terms: &mut TermArena,
    term: TermId,
) -> Result<TermId, CoreError> {
    reduce_generated_recursor_iota_with_fuel(env, terms, term, DEFAULT_IOTA_FUEL)
}

pub fn reduce_generated_recursor_iota_with_fuel(
    env: &Environment,
    terms: &mut TermArena,
    term: TermId,
    fuel: u32,
) -> Result<TermId, CoreError> {
    let mut fuel = fuel;
    reduce_recursor_iota_with_budget(env, terms, term, &mut fuel, MissingEquationMode::Reject)?
        .ok_or_else(|| iota_error(IotaReductionErrorKind::NoIotaRedex, term, None, None))
}

pub(crate) fn try_reduce_generated_recursor_iota_with_budget(
    env: &Environment,
    terms: &mut TermArena,
    term: TermId,
    fuel: &mut u32,
) -> Result<Option<TermId>, CoreError> {
    reduce_recursor_iota_with_budget(env, terms, term, fuel, MissingEquationMode::Neutral)
}

fn reduce_recursor_iota_with_budget(
    env: &Environment,
    terms: &mut TermArena,
    term: TermId,
    fuel: &mut u32,
    missing_mode: MissingEquationMode,
) -> Result<Option<TermId>, CoreError> {
    let Some(recursor_spine) = const_spine(terms, term) else {
        return missing_iota(term, missing_mode, None, None);
    };
    let Some(recursor_decl) = env.lookup(recursor_spine.head) else {
        return missing_iota(term, missing_mode, Some(recursor_spine.head), None);
    };
    let DeclarationKind::Recursor {
        inductive,
        generated,
        ..
    } = recursor_decl.kind()
    else {
        return missing_iota(term, missing_mode, Some(recursor_spine.head), None);
    };
    if !generated {
        return match missing_mode {
            MissingEquationMode::Neutral => Ok(None),
            MissingEquationMode::Reject => Err(iota_error(
                IotaReductionErrorKind::NonGeneratedRecursor,
                term,
                Some(recursor_spine.head),
                Some(inductive),
            )),
        };
    }

    let info = generated_recursor_info(env, recursor_spine.head, inductive, term)?;
    let arity = recursor_arity(info.shape);
    if recursor_spine.arguments.len() < arity {
        return missing_iota(
            term,
            missing_mode,
            Some(recursor_spine.head),
            Some(inductive),
        );
    }

    let major = recursor_spine.arguments[arity - 1];
    let Some(major_spine) = const_spine(terms, major) else {
        return missing_iota(
            term,
            missing_mode,
            Some(recursor_spine.head),
            Some(inductive),
        );
    };
    let Some(major_decl) = env.lookup(major_spine.head) else {
        return unknown_equation(term, recursor_spine.head, inductive, major_spine.head);
    };
    let DeclarationKind::Constructor {
        inductive: constructor_inductive,
        generated,
        ..
    } = major_decl.kind()
    else {
        return missing_iota(
            term,
            missing_mode,
            Some(recursor_spine.head),
            Some(inductive),
        );
    };
    if constructor_inductive != inductive || !generated {
        return unknown_equation(term, recursor_spine.head, inductive, major_spine.head);
    }

    let reduced = match info.shape {
        MvpInductiveShape::Bool => reduce_bool_iota(
            &info,
            &recursor_spine.arguments,
            &major_spine,
            term,
            recursor_spine.head,
            inductive,
        )?,
        MvpInductiveShape::Nat => {
            reduce_nat_iota(terms, &info, &recursor_spine, &major_spine, term, inductive)?
        }
        MvpInductiveShape::Eq => reduce_eq_iota(
            &info,
            &recursor_spine.arguments,
            &major_spine,
            term,
            recursor_spine.head,
            inductive,
        )?,
    };

    consume_iota_fuel(fuel, term, recursor_spine.head, inductive)?;
    Ok(Some(apply_trailing_arguments(
        terms,
        reduced,
        &recursor_spine.arguments[arity..],
    )))
}

fn reduce_bool_iota(
    info: &GeneratedRecursorInfo,
    recursor_arguments: &[TermId],
    major_spine: &ConstSpine,
    term: TermId,
    recursor: GlobalId,
    inductive: GlobalId,
) -> Result<TermId, CoreError> {
    if !major_spine.arguments.is_empty() {
        return unknown_equation(term, recursor, inductive, major_spine.head);
    }
    if major_spine.head == info.constructors[0] {
        Ok(recursor_arguments[0])
    } else if major_spine.head == info.constructors[1] {
        Ok(recursor_arguments[1])
    } else {
        unknown_equation(term, recursor, inductive, major_spine.head)
    }
}

fn reduce_nat_iota(
    terms: &mut TermArena,
    info: &GeneratedRecursorInfo,
    recursor_spine: &ConstSpine,
    major_spine: &ConstSpine,
    term: TermId,
    inductive: GlobalId,
) -> Result<TermId, CoreError> {
    if major_spine.head == info.constructors[0] {
        if major_spine.arguments.is_empty() {
            return Ok(recursor_spine.arguments[0]);
        }
        return unknown_equation(term, recursor_spine.head, inductive, major_spine.head);
    }

    if major_spine.head != info.constructors[1] || major_spine.arguments.len() != 1 {
        return unknown_equation(term, recursor_spine.head, inductive, major_spine.head);
    }

    let predecessor = major_spine.arguments[0];
    let recursor_const = terms.constant(recursor_spine.head, recursor_spine.levels.iter().copied());
    let recursive_result = terms.app(
        recursor_const,
        [
            recursor_spine.arguments[0],
            recursor_spine.arguments[1],
            predecessor,
        ],
    );
    Ok(terms.app(recursor_spine.arguments[1], [predecessor, recursive_result]))
}

fn reduce_eq_iota(
    info: &GeneratedRecursorInfo,
    recursor_arguments: &[TermId],
    major_spine: &ConstSpine,
    term: TermId,
    recursor: GlobalId,
    inductive: GlobalId,
) -> Result<TermId, CoreError> {
    if major_spine.head == info.constructors[0] && major_spine.arguments.is_empty() {
        return Ok(recursor_arguments[0]);
    }

    unknown_equation(term, recursor, inductive, major_spine.head)
}

fn apply_trailing_arguments(terms: &mut TermArena, reduced: TermId, trailing: &[TermId]) -> TermId {
    if trailing.is_empty() {
        reduced
    } else {
        terms.app(reduced, trailing.iter().copied())
    }
}

fn generated_recursor_info(
    env: &Environment,
    recursor: GlobalId,
    inductive: GlobalId,
    term: TermId,
) -> Result<GeneratedRecursorInfo, CoreError> {
    let family = env.lookup(inductive).ok_or_else(|| {
        iota_error(
            IotaReductionErrorKind::MalformedGeneratedRecursor,
            term,
            Some(recursor),
            Some(inductive),
        )
    })?;
    let family_name = family.name().as_str();
    let recursor_name = env
        .lookup(recursor)
        .map(|declaration| declaration.name().as_str())
        .unwrap_or_default();
    if recursor_name != format!("{family_name}.rec") {
        return Err(iota_error(
            IotaReductionErrorKind::MalformedGeneratedRecursor,
            term,
            Some(recursor),
            Some(inductive),
        ));
    }

    let constructors = env
        .iter()
        .filter_map(|declaration| match declaration.kind() {
            DeclarationKind::Constructor {
                inductive: actual,
                generated: true,
                ..
            } if actual == inductive => Some((declaration.global(), declaration.name().as_str())),
            _ => None,
        })
        .collect::<Vec<_>>();

    for shape in [
        MvpInductiveShape::Bool,
        MvpInductiveShape::Nat,
        MvpInductiveShape::Eq,
    ] {
        let expected = constructor_suffixes(shape)
            .iter()
            .map(|suffix| format!("{family_name}.{suffix}"))
            .collect::<Vec<_>>();
        if constructors.len() == expected.len()
            && constructors
                .iter()
                .zip(expected.iter())
                .all(|((_, actual), expected)| actual == expected)
        {
            return Ok(GeneratedRecursorInfo {
                shape,
                constructors: constructors.iter().map(|(global, _)| *global).collect(),
            });
        }
    }

    Err(iota_error(
        IotaReductionErrorKind::MalformedGeneratedRecursor,
        term,
        Some(recursor),
        Some(inductive),
    ))
}

fn recursor_arity(shape: MvpInductiveShape) -> usize {
    match shape {
        MvpInductiveShape::Bool | MvpInductiveShape::Nat => 3,
        MvpInductiveShape::Eq => 2,
    }
}

fn constructor_suffixes(shape: MvpInductiveShape) -> &'static [&'static str] {
    match shape {
        MvpInductiveShape::Bool => &["false", "true"],
        MvpInductiveShape::Nat => &["zero", "succ"],
        MvpInductiveShape::Eq => &["refl"],
    }
}

fn const_spine(terms: &TermArena, term: TermId) -> Option<ConstSpine> {
    match terms.node(term).clone() {
        TermNode::Const { global, levels } => Some(ConstSpine {
            head: global,
            levels,
            arguments: Vec::new(),
        }),
        TermNode::App {
            function,
            arguments,
        } => match terms.node(function).clone() {
            TermNode::Const { global, levels } => Some(ConstSpine {
                head: global,
                levels,
                arguments,
            }),
            _ => None,
        },
        _ => None,
    }
}

fn consume_iota_fuel(
    fuel: &mut u32,
    term: TermId,
    recursor: GlobalId,
    inductive: GlobalId,
) -> Result<(), CoreError> {
    if *fuel == 0 {
        return Err(
            CoreError::new(CoreErrorCode::FuelExhausted, iota_location())
                .with_detail("kind", "iota_fuel_exhausted")
                .with_detail("term_index", term.index().to_string())
                .with_detail("recursor", recursor.as_u32().to_string())
                .with_detail("inductive", inductive.as_u32().to_string()),
        );
    }

    *fuel -= 1;
    Ok(())
}

fn missing_iota(
    term: TermId,
    mode: MissingEquationMode,
    recursor: Option<GlobalId>,
    inductive: Option<GlobalId>,
) -> Result<Option<TermId>, CoreError> {
    match mode {
        MissingEquationMode::Neutral => Ok(None),
        MissingEquationMode::Reject => Err(iota_error(
            IotaReductionErrorKind::NoIotaRedex,
            term,
            recursor,
            inductive,
        )),
    }
}

fn unknown_equation<T>(
    term: TermId,
    recursor: GlobalId,
    inductive: GlobalId,
    major: GlobalId,
) -> Result<T, CoreError> {
    Err(iota_error(
        IotaReductionErrorKind::UnknownRecursorEquation,
        term,
        Some(recursor),
        Some(inductive),
    )
    .with_detail("major_constructor", major.as_u32().to_string()))
}

fn iota_error(
    kind: IotaReductionErrorKind,
    term: TermId,
    recursor: Option<GlobalId>,
    inductive: Option<GlobalId>,
) -> CoreError {
    let mut error = CoreError::new(CoreErrorCode::UnsupportedFeature, iota_location())
        .with_detail("kind", kind.as_str())
        .with_detail("term_index", term.index().to_string());
    if let Some(recursor) = recursor {
        error = error.with_detail("recursor", recursor.as_u32().to_string());
    }
    if let Some(inductive) = inductive {
        error = error.with_detail("inductive", inductive.as_u32().to_string());
    }
    error
}

fn iota_location() -> CoreLocation {
    CoreLocation::root().with_field("reduce_inductive")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MissingEquationMode {
    Neutral,
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ConstSpine {
    head: GlobalId,
    levels: Vec<crate::LevelId>,
    arguments: Vec<TermId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedRecursorInfo {
    shape: MvpInductiveShape,
    constructors: Vec<GlobalId>,
}

#[cfg(test)]
mod tests {
    use super::{
        reduce_generated_recursor_iota, reduce_generated_recursor_iota_with_fuel,
        IotaReductionErrorKind,
    };

    use std::fs;
    use std::path::Path;

    use crate::{
        definitionally_equal, generate_mvp_inductive_declarations, CoreError, CoreErrorCode,
        Environment, InductiveGenerationInput, LevelArena, MvpInductiveShape, TermArena, TermId,
    };

    const CORE_RECURSOR_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/core-recursor");

    #[derive(Debug)]
    struct RecursorFixture {
        id: String,
        expected: String,
        expected_error_kind: Option<String>,
    }

    struct GeneratedFixtureContext {
        env: Environment,
        terms: TermArena,
        family: crate::GlobalId,
        recursor: crate::GlobalId,
        constructors: Vec<crate::GlobalId>,
        ty: TermId,
    }

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn generated_context(shape: MvpInductiveShape, family_name: &str) -> GeneratedFixtureContext {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let generated = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(shape, family_name, Vec::new(), ty),
        )
        .expect("generation succeeds");

        GeneratedFixtureContext {
            env,
            terms,
            family: generated.family.global,
            recursor: generated.recursor.global,
            constructors: generated
                .constructors
                .iter()
                .map(|artifact| artifact.global)
                .collect(),
            ty,
        }
    }

    fn run_bool_false_fixture() -> Result<(TermId, TermId), CoreError> {
        let mut context = generated_context(MvpInductiveShape::Bool, "Std.Bool");
        let false_case = context.terms.var(0);
        let true_case = context.terms.var(1);
        let false_constructor = context.terms.constant(context.constructors[0], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [false_case, true_case, false_constructor]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
            .map(|reduced| (reduced, false_case))
    }

    fn run_bool_true_fixture() -> Result<(TermId, TermId), CoreError> {
        let mut context = generated_context(MvpInductiveShape::Bool, "Std.Bool");
        let false_case = context.terms.var(0);
        let true_case = context.terms.var(1);
        let true_constructor = context.terms.constant(context.constructors[1], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [false_case, true_case, true_constructor]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
            .map(|reduced| (reduced, true_case))
    }

    fn run_nat_zero_fixture() -> Result<(TermId, TermId), CoreError> {
        let mut context = generated_context(MvpInductiveShape::Nat, "Std.Nat");
        let zero_case = context.terms.var(0);
        let step_case = context.terms.var(1);
        let zero_constructor = context.terms.constant(context.constructors[0], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [zero_case, step_case, zero_constructor]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
            .map(|reduced| (reduced, zero_case))
    }

    fn run_nat_succ_fixture() -> Result<(TermId, TermId), CoreError> {
        let mut context = generated_context(MvpInductiveShape::Nat, "Std.Nat");
        let zero_case = context.terms.var(0);
        let step_case = context.terms.var(1);
        let predecessor = context.terms.var(2);
        let succ_constructor = context.terms.constant(context.constructors[1], []);
        let succ_major = context.terms.app(succ_constructor, [predecessor]);
        let recursor = context.terms.constant(context.recursor, []);
        let recursive_result = context
            .terms
            .app(recursor, [zero_case, step_case, predecessor]);
        let expected = context
            .terms
            .app(step_case, [predecessor, recursive_result]);
        let redex = context
            .terms
            .app(recursor, [zero_case, step_case, succ_major]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
            .map(|reduced| (reduced, expected))
    }

    fn run_eq_refl_fixture() -> Result<(TermId, TermId), CoreError> {
        let mut context = generated_context(MvpInductiveShape::Eq, "Std.Eq");
        let refl_case = context.terms.var(0);
        let refl_constructor = context.terms.constant(context.constructors[0], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context.terms.app(recursor, [refl_case, refl_constructor]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
            .map(|reduced| (reduced, refl_case))
    }

    fn run_unknown_recursor_fixture() -> Result<TermId, CoreError> {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let family = env
            .register_inductive("Std.ManualBool", ty)
            .expect("manual family");
        let false_ctor = env
            .register_constructor("Std.ManualBool.false", ty, family)
            .expect("manual false");
        let true_ctor = env
            .register_constructor("Std.ManualBool.true", ty, family)
            .expect("manual true");
        let recursor = env
            .register_recursor("Std.ManualBool.rec", ty, family)
            .expect("manual recursor");
        let false_case = terms.var(0);
        let true_case = terms.var(1);
        let false_constructor = terms.constant(false_ctor, []);
        let recursor_term = terms.constant(recursor, []);
        let redex = terms.app(recursor_term, [false_case, true_case, false_constructor]);

        assert!(env.lookup(true_ctor).is_some());
        reduce_generated_recursor_iota(&env, &mut terms, redex)
    }

    fn run_unknown_constructor_equation_fixture() -> Result<TermId, CoreError> {
        let mut context = generated_context(MvpInductiveShape::Bool, "Std.Bool");
        let other_constructor = context
            .env
            .register_constructor("Std.Bool.other", context.ty, context.family)
            .expect("manual constructor on generated family");
        let false_case = context.terms.var(0);
        let true_case = context.terms.var(1);
        let other_major = context.terms.constant(other_constructor, []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [false_case, true_case, other_major]);

        reduce_generated_recursor_iota(&context.env, &mut context.terms, redex)
    }

    fn read_core_recursor_fixtures() -> Vec<RecursorFixture> {
        let mut entries = fs::read_dir(CORE_RECURSOR_FIXTURE_DIR)
            .expect("core-recursor fixture directory exists")
            .map(|entry| entry.expect("fixture dir entry is readable").path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "fixture")
            })
            .collect::<Vec<_>>();
        entries.sort();

        entries
            .into_iter()
            .map(|path| parse_core_recursor_fixture(&path))
            .collect()
    }

    fn parse_core_recursor_fixture(path: &Path) -> RecursorFixture {
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

        RecursorFixture {
            id: id.unwrap_or_else(|| panic!("fixture id missing in {}", path.display())),
            expected: expected.unwrap_or_else(|| {
                panic!("fixture expected verdict missing in {}", path.display())
            }),
            expected_error_kind,
        }
    }

    fn run_core_recursor_fixture(fixture: &RecursorFixture) -> Result<(), CoreError> {
        match fixture.id.as_str() {
            "bool-false" => run_bool_false_fixture().map(|_| ()),
            "bool-true" => run_bool_true_fixture().map(|_| ()),
            "nat-zero" => run_nat_zero_fixture().map(|_| ()),
            "nat-succ" => run_nat_succ_fixture().map(|_| ()),
            "eq-refl" => run_eq_refl_fixture().map(|_| ()),
            "unknown-recursor" => run_unknown_recursor_fixture().map(|_| ()),
            "unknown-constructor-equation" => {
                run_unknown_constructor_equation_fixture().map(|_| ())
            }
            id => panic!("unknown core-recursor fixture id `{id}`"),
        }
    }

    #[test]
    fn generated_bool_recursor_iota_reduces_by_constructor() {
        let (false_result, false_case) = run_bool_false_fixture().expect("false iota reduces");
        let (true_result, true_case) = run_bool_true_fixture().expect("true iota reduces");

        assert_eq!(false_result, false_case);
        assert_eq!(true_result, true_case);
    }

    #[test]
    fn generated_nat_recursor_iota_reduces_zero_and_succ() {
        let (zero_result, zero_case) = run_nat_zero_fixture().expect("zero iota reduces");
        let (succ_result, succ_expected) = run_nat_succ_fixture().expect("succ iota reduces");

        assert_eq!(zero_result, zero_case);
        assert_eq!(succ_result, succ_expected);
    }

    #[test]
    fn generated_eq_recursor_iota_reduces_refl() {
        let (result, refl_case) = run_eq_refl_fixture().expect("refl iota reduces");

        assert_eq!(result, refl_case);
    }

    #[test]
    fn non_generated_recursor_equation_rejects() {
        let error = run_unknown_recursor_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(IotaReductionErrorKind::NonGeneratedRecursor.as_str())
        );
    }

    #[test]
    fn unknown_generated_recursor_equation_rejects() {
        let error = run_unknown_constructor_equation_fixture().unwrap_err();

        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some(IotaReductionErrorKind::UnknownRecursorEquation.as_str())
        );
    }

    #[test]
    fn generated_iota_participates_in_definitional_equality() {
        let mut context = generated_context(MvpInductiveShape::Bool, "Std.Bool");
        let false_case = context.terms.var(0);
        let true_case = context.terms.var(1);
        let false_constructor = context.terms.constant(context.constructors[0], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [false_case, true_case, false_constructor]);

        assert!(
            definitionally_equal(&context.env, &mut context.terms, redex, false_case)
                .expect("defeq iota succeeds")
        );
    }

    #[test]
    fn iota_fuel_exhaustion_rejects_deterministically() {
        let mut context = generated_context(MvpInductiveShape::Bool, "Std.Bool");
        let false_case = context.terms.var(0);
        let true_case = context.terms.var(1);
        let false_constructor = context.terms.constant(context.constructors[0], []);
        let recursor = context.terms.constant(context.recursor, []);
        let redex = context
            .terms
            .app(recursor, [false_case, true_case, false_constructor]);

        let error =
            reduce_generated_recursor_iota_with_fuel(&context.env, &mut context.terms, redex, 0)
                .unwrap_err();

        assert_eq!(error.code(), CoreErrorCode::FuelExhausted);
        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some("iota_fuel_exhausted")
        );
    }

    #[test]
    fn core_recursor_fixtures_are_deterministic() {
        let fixtures = read_core_recursor_fixtures();
        assert!(!fixtures.is_empty());

        for fixture in fixtures {
            let result = run_core_recursor_fixture(&fixture);
            match fixture.expected.as_str() {
                "reduce" => {
                    result.unwrap_or_else(|error| {
                        panic!(
                            "fixture `{}` should reduce but rejected with {}",
                            fixture.id,
                            error.to_deterministic_json()
                        )
                    });
                }
                "reject" => {
                    let error = match result {
                        Ok(()) => panic!("fixture `{}` should reject but reduced", fixture.id),
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
