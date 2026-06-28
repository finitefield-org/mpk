//! Core type inference and checking skeleton.

use crate::{
    lift, substitute_top, whnf, CoreError, CoreErrorCode, CoreLocation, Environment, GlobalId,
    LevelArena, LevelId, LocalContext, TermArena, TermId, TermNode,
};

pub fn infer(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
) -> Result<TermId, CoreError> {
    match terms.node(term).clone() {
        TermNode::Sort(level) => Ok(infer_sort(levels, terms, level)),
        TermNode::Var(index) => infer_var(terms, context, term, index),
        TermNode::Const { global, .. } => infer_const(env, term, global),
        TermNode::Lam { ty, body } => infer_lam(levels, terms, context, env, term, ty, body),
        TermNode::App {
            function,
            arguments,
        } => infer_app(levels, terms, context, env, term, function, arguments),
        TermNode::Pi { ty, body } => infer_pi(levels, terms, context, env, term, ty, body),
        TermNode::Let { ty, value, body } => infer_let(
            levels,
            terms,
            context,
            env,
            LetInference {
                term,
                ty,
                value,
                body,
            },
        ),
    }
}

pub fn check(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
    expected: TermId,
) -> Result<(), CoreError> {
    match (terms.node(term).clone(), terms.node(expected).clone()) {
        (
            TermNode::Lam { ty, body },
            TermNode::Pi {
                ty: expected_ty,
                body: expected_body,
            },
        ) => check_lam(
            levels,
            terms,
            context,
            env,
            LambdaCheck {
                term,
                ty,
                body,
                expected,
                expected_ty,
                expected_body,
            },
        ),
        _ => check_by_inference(levels, terms, context, env, term, expected),
    }
}

pub fn infer_sort(levels: &mut LevelArena, terms: &mut TermArena, level: LevelId) -> TermId {
    let ty_level = levels.succ(level);
    terms.sort(ty_level)
}

fn infer_var(
    terms: &mut TermArena,
    context: &LocalContext,
    term: TermId,
    index: u32,
) -> Result<TermId, CoreError> {
    let ty = context.lookup_var_type(index).ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::UnboundVariable,
            CoreLocation::root().with_field("infer"),
        )
        .with_detail("kind", "unbound_variable")
        .with_detail("index", index.to_string())
        .with_detail("term_index", term.index().to_string())
    })?;
    let amount = index.checked_add(1).ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::InternalInvariant,
            CoreLocation::root().with_field("infer"),
        )
        .with_detail("kind", "var_lift_amount_overflow")
        .with_detail("index", index.to_string())
        .with_detail("term_index", term.index().to_string())
    })?;

    lift(terms, ty, amount)
        .map_err(|error| {
            CoreError::from_subst_error(CoreLocation::root().with_field("infer"), error)
        })
        .map_err(|error| {
            error
                .with_detail("operation", "var_type_lift")
                .with_detail("index", index.to_string())
                .with_detail("term_index", term.index().to_string())
                .with_detail("type_index", ty.index().to_string())
        })
}

fn infer_const(env: &Environment, term: TermId, global: GlobalId) -> Result<TermId, CoreError> {
    env.lookup(global)
        .map(|declaration| declaration.ty())
        .ok_or_else(|| {
            CoreError::new(
                CoreErrorCode::UnknownGlobal,
                CoreLocation::root().with_field("infer"),
            )
            .with_detail("kind", "unknown_global")
            .with_detail("global", global.as_u32().to_string())
            .with_detail("term_index", term.index().to_string())
        })
}

fn infer_lam(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
    ty: TermId,
    body: TermId,
) -> Result<TermId, CoreError> {
    let domain_type = infer(levels, terms, context, env, ty)?;
    expect_sort(terms, term, "lambda", "domain", ty, domain_type)?;

    let body_context = extend_context_with_binder(context, ty);
    let body_type = infer(levels, terms, &body_context, env, body)?;
    let body_type_type = infer(levels, terms, &body_context, env, body_type)?;
    expect_sort(
        terms,
        term,
        "lambda",
        "body_type",
        body_type,
        body_type_type,
    )?;

    Ok(terms.pi(ty, body_type))
}

fn infer_app(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
    function: TermId,
    arguments: Vec<TermId>,
) -> Result<TermId, CoreError> {
    let mut function_type = infer(levels, terms, context, env, function)?;

    for (argument_index, argument) in arguments.into_iter().enumerate() {
        let argument_index = app_argument_index(term, argument_index)?;
        let whnf_type = whnf(terms, function_type).map_err(|error| {
            CoreError::from_reduce_error(app_location(argument_index), error)
                .with_detail("operation", "app_function_type_whnf")
                .with_detail("term_index", term.index().to_string())
                .with_detail("function_term_index", function.index().to_string())
                .with_detail("function_type_index", function_type.index().to_string())
                .with_detail("argument_index", argument_index.to_string())
                .with_detail("argument_term_index", argument.index().to_string())
        })?;

        let TermNode::Pi { ty, body } = terms.node(whnf_type).clone() else {
            return Err(app_not_function_error(
                terms,
                term,
                function,
                function_type,
                whnf_type,
                argument_index,
                argument,
            ));
        };

        check(levels, terms, context, env, argument, ty)?;
        function_type = substitute_top(terms, body, argument).map_err(|error| {
            CoreError::from_subst_error(app_location(argument_index), error)
                .with_detail("operation", "app_result_type_substitution")
                .with_detail("term_index", term.index().to_string())
                .with_detail("function_term_index", function.index().to_string())
                .with_detail("body_type_index", body.index().to_string())
                .with_detail("argument_index", argument_index.to_string())
                .with_detail("argument_term_index", argument.index().to_string())
        })?;
    }

    Ok(function_type)
}

fn infer_pi(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
    ty: TermId,
    body: TermId,
) -> Result<TermId, CoreError> {
    let domain_type = infer(levels, terms, context, env, ty)?;
    let domain_level = expect_sort(terms, term, "pi", "domain", ty, domain_type)?;

    let body_context = extend_context_with_binder(context, ty);
    let body_type = infer(levels, terms, &body_context, env, body)?;
    let body_level = expect_sort(terms, term, "pi", "body", body, body_type)?;

    let pi_level = levels.max(domain_level, body_level);
    Ok(terms.sort(pi_level))
}

struct LetInference {
    term: TermId,
    ty: TermId,
    value: TermId,
    body: TermId,
}

fn infer_let(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    let_term: LetInference,
) -> Result<TermId, CoreError> {
    let ty_type = infer(levels, terms, context, env, let_term.ty)?;
    expect_sort(terms, let_term.term, "let", "type", let_term.ty, ty_type)?;
    check(levels, terms, context, env, let_term.value, let_term.ty)?;

    let body_context = extend_context_with_definition(context, let_term.ty, let_term.value);
    let body_type = infer(levels, terms, &body_context, env, let_term.body)?;
    substitute_top(terms, body_type, let_term.value)
        .map_err(|error| CoreError::from_subst_error(let_body_location(), error))
        .map_err(|error| {
            error
                .with_detail("operation", "let_body_type_substitution")
                .with_detail("term_index", let_term.term.index().to_string())
                .with_detail("type_index", let_term.ty.index().to_string())
                .with_detail("value_term_index", let_term.value.index().to_string())
                .with_detail("body_term_index", let_term.body.index().to_string())
                .with_detail("body_type_index", body_type.index().to_string())
        })
}

struct LambdaCheck {
    term: TermId,
    ty: TermId,
    body: TermId,
    expected: TermId,
    expected_ty: TermId,
    expected_body: TermId,
}

fn check_lam(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    lambda: LambdaCheck,
) -> Result<(), CoreError> {
    infer(levels, terms, context, env, lambda.expected)?;
    if lambda.ty != lambda.expected_ty {
        return Err(lambda_domain_mismatch_error(
            terms,
            lambda.term,
            lambda.ty,
            lambda.expected_ty,
        ));
    }

    let body_context = extend_context_with_binder(context, lambda.expected_ty);
    check(
        levels,
        terms,
        &body_context,
        env,
        lambda.body,
        lambda.expected_body,
    )
}

fn check_by_inference(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    context: &LocalContext,
    env: &Environment,
    term: TermId,
    expected: TermId,
) -> Result<(), CoreError> {
    let inferred = infer(levels, terms, context, env, term)?;
    if inferred == expected {
        return Ok(());
    }

    Err(check_type_mismatch_error(terms, term, expected, inferred))
}

fn extend_context_with_binder(context: &LocalContext, ty: TermId) -> LocalContext {
    let mut extended = context.clone();
    extended.push_binder(ty);
    extended
}

fn extend_context_with_definition(
    context: &LocalContext,
    ty: TermId,
    value: TermId,
) -> LocalContext {
    let mut extended = context.clone();
    extended.push_definition(ty, value);
    extended
}

fn app_argument_index(term: TermId, index: usize) -> Result<u32, CoreError> {
    u32::try_from(index).map_err(|_| {
        CoreError::new(
            CoreErrorCode::InternalInvariant,
            CoreLocation::root().with_field("infer").with_field("app"),
        )
        .with_detail("kind", "app_argument_index_overflow")
        .with_detail("term_index", term.index().to_string())
        .with_detail("argument_index", index.to_string())
    })
}

fn app_location(argument_index: u32) -> CoreLocation {
    CoreLocation::root()
        .with_field("infer")
        .with_field("app")
        .with_index(argument_index)
}

fn let_body_location() -> CoreLocation {
    CoreLocation::root()
        .with_field("infer")
        .with_field("let")
        .with_field("body")
}

fn expect_sort(
    terms: &TermArena,
    term: TermId,
    construct: &'static str,
    component: &'static str,
    subject: TermId,
    inferred: TermId,
) -> Result<LevelId, CoreError> {
    match terms.node(inferred) {
        TermNode::Sort(level) => Ok(*level),
        node => {
            let kind = format!("{construct}_component_not_sort");
            Err(CoreError::new(
                CoreErrorCode::TypeMismatch,
                CoreLocation::root()
                    .with_field("infer")
                    .with_field(construct)
                    .with_field(component),
            )
            .with_detail("kind", kind)
            .with_detail("term_index", term.index().to_string())
            .with_detail("subject_term_index", subject.index().to_string())
            .with_detail("inferred_term_index", inferred.index().to_string())
            .with_detail("expected", "sort")
            .with_detail("actual", term_kind(node)))
        }
    }
}

fn check_type_mismatch_error(
    terms: &TermArena,
    term: TermId,
    expected: TermId,
    inferred: TermId,
) -> CoreError {
    CoreError::new(
        CoreErrorCode::TypeMismatch,
        CoreLocation::root().with_field("check"),
    )
    .with_detail("kind", "check_type_mismatch")
    .with_detail("term_index", term.index().to_string())
    .with_detail("expected_term_index", expected.index().to_string())
    .with_detail("expected_kind", term_kind(terms.node(expected)))
    .with_detail("inferred_term_index", inferred.index().to_string())
    .with_detail("inferred_kind", term_kind(terms.node(inferred)))
}

fn lambda_domain_mismatch_error(
    terms: &TermArena,
    term: TermId,
    actual: TermId,
    expected: TermId,
) -> CoreError {
    CoreError::new(
        CoreErrorCode::TypeMismatch,
        CoreLocation::root()
            .with_field("check")
            .with_field("lambda")
            .with_field("domain"),
    )
    .with_detail("kind", "lambda_domain_mismatch")
    .with_detail("term_index", term.index().to_string())
    .with_detail("actual_domain_index", actual.index().to_string())
    .with_detail("actual_kind", term_kind(terms.node(actual)))
    .with_detail("expected_domain_index", expected.index().to_string())
    .with_detail("expected_kind", term_kind(terms.node(expected)))
}

fn app_not_function_error(
    terms: &TermArena,
    term: TermId,
    function: TermId,
    function_type: TermId,
    whnf_type: TermId,
    argument_index: u32,
    argument: TermId,
) -> CoreError {
    CoreError::new(CoreErrorCode::NotAFunction, app_location(argument_index))
        .with_detail("kind", "app_function_type_not_pi")
        .with_detail("term_index", term.index().to_string())
        .with_detail("function_term_index", function.index().to_string())
        .with_detail("function_type_index", function_type.index().to_string())
        .with_detail("whnf_type_index", whnf_type.index().to_string())
        .with_detail("actual", term_kind(terms.node(whnf_type)))
        .with_detail("argument_index", argument_index.to_string())
        .with_detail("argument_term_index", argument.index().to_string())
}

fn term_kind(node: &TermNode) -> &'static str {
    match node {
        TermNode::Sort(_) => "sort",
        TermNode::Var(_) => "var",
        TermNode::Const { .. } => "const",
        TermNode::App { .. } => "app",
        TermNode::Lam { .. } => "lam",
        TermNode::Pi { .. } => "pi",
        TermNode::Let { .. } => "let",
    }
}

#[cfg(test)]
mod tests {
    use super::term_kind;

    use crate::{
        check, infer, infer_sort, Environment, LevelArena, LevelNode, LocalContext, TermArena,
        TermId, TermNode,
    };

    fn check_type_mismatch_json(
        terms: &TermArena,
        term: TermId,
        expected: TermId,
        inferred: TermId,
    ) -> String {
        format!(
            "{{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{{\"field\":\"check\"}}],\"details\":{{\"expected_kind\":\"{}\",\"expected_term_index\":\"{}\",\"inferred_kind\":\"{}\",\"inferred_term_index\":\"{}\",\"kind\":\"check_type_mismatch\",\"term_index\":\"{}\"}}}}",
            term_kind(terms.node(expected)),
            expected.index(),
            term_kind(terms.node(inferred)),
            inferred.index(),
            term.index()
        )
    }

    fn lambda_domain_mismatch_json(
        terms: &TermArena,
        term: TermId,
        actual: TermId,
        expected: TermId,
    ) -> String {
        format!(
            "{{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{{\"field\":\"check\"}},{{\"field\":\"lambda\"}},{{\"field\":\"domain\"}}],\"details\":{{\"actual_domain_index\":\"{}\",\"actual_kind\":\"{}\",\"expected_domain_index\":\"{}\",\"expected_kind\":\"{}\",\"kind\":\"lambda_domain_mismatch\",\"term_index\":\"{}\"}}}}",
            actual.index(),
            term_kind(terms.node(actual)),
            expected.index(),
            term_kind(terms.node(expected)),
            term.index()
        )
    }

    fn infer_component_not_sort_json(
        terms: &TermArena,
        construct: &str,
        component: &str,
        term: TermId,
        subject: TermId,
        inferred: TermId,
    ) -> String {
        format!(
            "{{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{{\"field\":\"infer\"}},{{\"field\":\"{}\"}},{{\"field\":\"{}\"}}],\"details\":{{\"actual\":\"{}\",\"expected\":\"sort\",\"inferred_term_index\":\"{}\",\"kind\":\"{}_component_not_sort\",\"subject_term_index\":\"{}\",\"term_index\":\"{}\"}}}}",
            construct,
            component,
            term_kind(terms.node(inferred)),
            inferred.index(),
            construct,
            subject.index(),
            term.index()
        )
    }

    fn app_not_function_json(
        terms: &TermArena,
        app: TermId,
        function: TermId,
        function_type: TermId,
        whnf_type: TermId,
        argument_index: u32,
        argument: TermId,
    ) -> String {
        format!(
            "{{\"code\":\"CORE_NOT_A_FUNCTION\",\"location\":[{{\"field\":\"infer\"}},{{\"field\":\"app\"}},{{\"index\":{}}}],\"details\":{{\"actual\":\"{}\",\"argument_index\":\"{}\",\"argument_term_index\":\"{}\",\"function_term_index\":\"{}\",\"function_type_index\":\"{}\",\"kind\":\"app_function_type_not_pi\",\"term_index\":\"{}\",\"whnf_type_index\":\"{}\"}}}}",
            argument_index,
            term_kind(terms.node(whnf_type)),
            argument_index,
            argument.index(),
            function.index(),
            function_type.index(),
            app.index(),
            whnf_type.index()
        )
    }

    #[test]
    fn infers_sort_successor_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let sort_u = terms.sort(u);

        let inferred = infer(&mut levels, &mut terms, &context, &env, sort_u).expect("sort infers");
        let succ_u = levels.succ(u);

        assert_eq!(terms.node(inferred), &TermNode::Sort(succ_u));
        assert_eq!(levels.node(succ_u), &LevelNode::Succ(u));
    }

    #[test]
    fn infers_sort_zero_as_sort_succ_zero() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let sort_zero = terms.sort(zero);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, sort_zero).expect("sort infers");
        let succ_zero = levels.succ(zero);

        assert_eq!(terms.node(inferred), &TermNode::Sort(succ_zero));
    }

    #[test]
    fn infer_sort_reuses_interned_successor_sort() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let u = levels.parse_param("u").expect("valid level param");

        let first = infer_sort(&mut levels, &mut terms, u);
        let second = infer_sort(&mut levels, &mut terms, u);

        assert_eq!(first, second);
    }

    #[test]
    fn infers_var_from_local_context() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let v = levels.parse_param("v").expect("valid level param");
        let outer_ty = terms.sort(u);
        let inner_ty = terms.sort(v);
        let var_zero = terms.var(0);
        let var_one = terms.var(1);

        context.push_binder(outer_ty);
        context.push_binder(inner_ty);

        assert_eq!(
            infer(&mut levels, &mut terms, &context, &env, var_zero).expect("var 0 infers"),
            inner_ty
        );
        assert_eq!(
            infer(&mut levels, &mut terms, &context, &env, var_one).expect("var 1 infers"),
            outer_ty
        );
    }

    #[test]
    fn infers_var_from_local_definition_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let v = levels.parse_param("v").expect("valid level param");
        let outer_ty = terms.sort(u);
        let definition_ty = terms.sort(v);
        let definition_value = terms.var(0);
        let local_definition = terms.var(0);

        context.push_binder(outer_ty);
        context.push_definition(definition_ty, definition_value);

        assert_eq!(
            infer(&mut levels, &mut terms, &context, &env, local_definition)
                .expect("local definition var infers"),
            definition_ty
        );
    }

    #[test]
    fn infers_var_lifts_dependent_local_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let type_sort = terms.sort(u);
        let dependent_type = terms.var(0);
        let local_value = terms.var(0);

        context.push_binder(type_sort);
        context.push_binder(dependent_type);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, local_value).expect("var infers");
        let lifted_type = terms.var(1);

        assert_eq!(inferred, lifted_type);
    }

    #[test]
    fn unbound_var_inference_rejects_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let var = terms.var(2);

        let error = infer(&mut levels, &mut terms, &context, &env, var).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_UNBOUND_VARIABLE\",\"location\":[{\"field\":\"infer\"}],\"details\":{\"index\":\"2\",\"kind\":\"unbound_variable\",\"term_index\":\"0\"}}"
        );
    }

    #[test]
    fn infers_const_from_environment() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let mut env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let ty = terms.sort(u);
        let global = env
            .register_axiom("Core.Prop", ty)
            .expect("valid declaration");
        let constant = terms.constant(global, []);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, constant).expect("const infers");

        assert_eq!(inferred, ty);
    }

    #[test]
    fn infers_non_dependent_pi_sort_from_domain_and_body_sorts() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let v = levels.parse_param("v").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.sort(v);
        let pi = terms.pi(domain, body);

        let inferred = infer(&mut levels, &mut terms, &context, &env, pi).expect("pi infers");
        let domain_level = levels.succ(u);
        let body_level = levels.succ(v);
        let expected_level = levels.max(domain_level, body_level);

        assert_eq!(terms.node(inferred), &TermNode::Sort(expected_level));
    }

    #[test]
    fn infers_dependent_pi_body_in_extended_context() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.var(0);
        let pi = terms.pi(domain, body);

        let inferred = infer(&mut levels, &mut terms, &context, &env, pi).expect("pi infers");
        let domain_level = levels.succ(u);
        let expected_level = levels.max(domain_level, u);

        assert_eq!(terms.node(inferred), &TermNode::Sort(expected_level));
    }

    #[test]
    fn pi_domain_must_infer_to_sort_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let non_sort_type = terms.var(9);
        let domain = terms.var(0);
        let u = levels.parse_param("u").expect("valid level param");
        let body = terms.sort(u);
        let pi = terms.pi(domain, body);

        context.push_binder(non_sort_type);

        let error = infer(&mut levels, &mut terms, &context, &env, pi).unwrap_err();
        let inferred_domain_type = terms.var(10);

        assert_eq!(
            error.to_deterministic_json(),
            infer_component_not_sort_json(&terms, "pi", "domain", pi, domain, inferred_domain_type)
        );
    }

    #[test]
    fn pi_body_must_infer_to_sort_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let non_sort_type = terms.var(9);
        let u = levels.parse_param("u").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.var(1);
        let pi = terms.pi(domain, body);

        context.push_binder(non_sort_type);

        let error = infer(&mut levels, &mut terms, &context, &env, pi).unwrap_err();
        let inferred_body_type = terms.var(11);

        assert_eq!(
            error.to_deterministic_json(),
            infer_component_not_sort_json(&terms, "pi", "body", pi, body, inferred_body_type)
        );
    }

    #[test]
    fn infers_lambda_as_pi_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);

        let inferred = infer(&mut levels, &mut terms, &context, &env, lambda).expect("lam infers");
        let expected = terms.pi(domain, domain);

        assert_eq!(inferred, expected);
    }

    #[test]
    fn infers_lambda_with_dependent_domain_lifted_under_body_binder() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let type_sort = terms.sort(u);
        let domain = terms.var(0);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);

        context.push_binder(type_sort);

        let inferred = infer(&mut levels, &mut terms, &context, &env, lambda).expect("lam infers");
        let lifted_domain = terms.var(1);
        let expected = terms.pi(domain, lifted_domain);

        assert_eq!(inferred, expected);
    }

    #[test]
    fn checks_lambda_against_expected_pi() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);
        let expected = terms.pi(domain, domain);

        check(&mut levels, &mut terms, &context, &env, lambda, expected)
            .expect("lambda checks against pi");
    }

    #[test]
    fn lambda_check_rejects_non_pi_expected_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let expected = terms.sort(u);
        let body = terms.var(0);
        let lambda = terms.lam(expected, body);

        let error = check(&mut levels, &mut terms, &context, &env, lambda, expected).unwrap_err();
        let inferred = terms.pi(expected, expected);

        assert_eq!(
            error.to_deterministic_json(),
            check_type_mismatch_json(&terms, lambda, expected, inferred)
        );
    }

    #[test]
    fn lambda_check_rejects_domain_mismatch_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let v = levels.parse_param("v").expect("valid level param");
        let actual_domain = terms.sort(u);
        let body = terms.var(0);
        let expected_domain = terms.sort(v);
        let lambda = terms.lam(actual_domain, body);
        let expected = terms.pi(expected_domain, expected_domain);

        let error = check(&mut levels, &mut terms, &context, &env, lambda, expected).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            lambda_domain_mismatch_json(&terms, lambda, actual_domain, expected_domain)
        );
    }

    #[test]
    fn lambda_check_rejects_body_mismatch_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let v = levels.parse_param("v").expect("valid level param");
        let domain = terms.sort(u);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);
        let expected_body = terms.sort(v);
        let expected = terms.pi(domain, expected_body);

        let error = check(&mut levels, &mut terms, &context, &env, lambda, expected).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            check_type_mismatch_json(&terms, body, expected_body, domain)
        );
    }

    #[test]
    fn lambda_domain_must_infer_to_sort_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let non_sort_type = terms.var(9);
        let domain = terms.var(0);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);

        context.push_binder(non_sort_type);

        let error = infer(&mut levels, &mut terms, &context, &env, lambda).unwrap_err();
        let inferred_domain_type = terms.var(10);

        assert_eq!(
            error.to_deterministic_json(),
            infer_component_not_sort_json(
                &terms,
                "lambda",
                "domain",
                lambda,
                domain,
                inferred_domain_type
            )
        );
    }

    #[test]
    fn infers_application_by_checking_argument_against_pi_domain() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let domain_level = levels.succ(zero);
        let domain = terms.sort(domain_level);
        let argument = terms.sort(zero);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);
        let application = terms.app(lambda, [argument]);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, application).expect("app infers");

        assert_eq!(inferred, domain);
    }

    #[test]
    fn infers_application_substitutes_dependent_result_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let mut env = Environment::new();
        let zero = levels.zero();
        let domain_level = levels.succ(zero);
        let domain = terms.sort(domain_level);
        let argument = terms.sort(zero);
        let dependent_body = terms.var(0);
        let function_type = terms.pi(domain, dependent_body);
        let global = env
            .register_axiom("Core.DependentFn", function_type)
            .expect("valid declaration");
        let function = terms.constant(global, []);
        let application = terms.app(function, [argument]);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, application).expect("app infers");

        assert_eq!(inferred, argument);
    }

    #[test]
    fn infers_application_after_whnf_function_type_to_pi() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let mut env = Environment::new();
        let zero = levels.zero();
        let domain_level = levels.succ(zero);
        let domain = terms.sort(domain_level);
        let argument = terms.sort(zero);
        let pi_type = terms.pi(domain, domain);
        let wrapped_body = terms.var(0);
        let wrapped_type = terms.let_term(domain, pi_type, wrapped_body);
        let global = env
            .register_axiom("Core.WrappedFn", wrapped_type)
            .expect("valid declaration");
        let function = terms.constant(global, []);
        let application = terms.app(function, [argument]);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, application).expect("app infers");

        assert_eq!(inferred, domain);
    }

    #[test]
    fn infers_multi_argument_application_spine() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let domain_level = levels.succ(zero);
        let domain = terms.sort(domain_level);
        let first_argument = terms.sort(zero);
        let second_argument = terms.sort(zero);
        let inner_body = terms.var(0);
        let inner_lambda = terms.lam(domain, inner_body);
        let outer_lambda = terms.lam(domain, inner_lambda);
        let application = terms.app(outer_lambda, [first_argument, second_argument]);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, application).expect("app infers");

        assert_eq!(inferred, domain);
    }

    #[test]
    fn application_rejects_non_function_type_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let function = terms.sort(zero);
        let argument = terms.sort(zero);
        let application = terms.app(function, [argument]);

        let error = infer(&mut levels, &mut terms, &context, &env, application).unwrap_err();
        let function_type = terms.sort(levels.succ(zero));

        assert_eq!(
            error.to_deterministic_json(),
            app_not_function_json(
                &terms,
                application,
                function,
                function_type,
                function_type,
                0,
                argument
            )
        );
    }

    #[test]
    fn application_rejects_argument_type_mismatch_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let domain_level = levels.succ(zero);
        let wrong_argument_type_level = levels.succ(domain_level);
        let domain = terms.sort(domain_level);
        let wrong_argument = terms.sort(domain_level);
        let body = terms.var(0);
        let lambda = terms.lam(domain, body);
        let application = terms.app(lambda, [wrong_argument]);

        let error = infer(&mut levels, &mut terms, &context, &env, application).unwrap_err();
        let wrong_argument_type = terms.sort(wrong_argument_type_level);

        assert_eq!(
            error.to_deterministic_json(),
            check_type_mismatch_json(&terms, wrong_argument, domain, wrong_argument_type)
        );
    }

    #[test]
    fn infers_let_body_with_local_definition() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let type_level = levels.succ(zero);
        let ty = terms.sort(type_level);
        let value = terms.sort(zero);
        let body = terms.var(0);
        let let_term = terms.let_term(ty, value, body);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, let_term).expect("let infers");

        assert_eq!(inferred, ty);
    }

    #[test]
    fn infers_let_body_type_after_discharging_local_definition() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let type_sort = terms.sort(u);
        let outer_type_before_value = terms.var(0);
        context.push_binder(type_sort);
        context.push_binder(outer_type_before_value);
        let outer_type = terms.var(1);
        let value = terms.var(0);
        let body = terms.var(0);
        let let_term = terms.let_term(outer_type, value, body);

        let inferred =
            infer(&mut levels, &mut terms, &context, &env, let_term).expect("let infers");

        assert_eq!(inferred, outer_type);
    }

    #[test]
    fn let_type_must_infer_to_sort_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut context = LocalContext::new();
        let env = Environment::new();
        let non_sort_type = terms.var(9);
        let ty = terms.var(0);
        let value = terms.var(0);
        let body = terms.var(0);
        let let_term = terms.let_term(ty, value, body);

        context.push_binder(non_sort_type);

        let error = infer(&mut levels, &mut terms, &context, &env, let_term).unwrap_err();
        let inferred_type_type = terms.var(10);

        assert_eq!(
            error.to_deterministic_json(),
            infer_component_not_sort_json(&terms, "let", "type", let_term, ty, inferred_type_type)
        );
    }

    #[test]
    fn let_value_must_check_against_annotation_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let zero = levels.zero();
        let type_level = levels.succ(zero);
        let wrong_value_type_level = levels.succ(type_level);
        let ty = terms.sort(type_level);
        let wrong_value = terms.sort(type_level);
        let body = terms.var(0);
        let let_term = terms.let_term(ty, wrong_value, body);

        let error = infer(&mut levels, &mut terms, &context, &env, let_term).unwrap_err();
        let wrong_value_type = terms.sort(wrong_value_type_level);

        assert_eq!(
            error.to_deterministic_json(),
            check_type_mismatch_json(&terms, wrong_value, ty, wrong_value_type)
        );
    }

    #[test]
    fn unknown_const_inference_rejects_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let mut other_env = Environment::new();
        let u = levels.parse_param("u").expect("valid level param");
        let ty = terms.sort(u);
        let unknown = other_env
            .register_axiom("Other.Prop", ty)
            .expect("valid declaration");
        let constant = terms.constant(unknown, []);

        let error = infer(&mut levels, &mut terms, &context, &env, constant).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_UNKNOWN_GLOBAL\",\"location\":[{\"field\":\"infer\"}],\"details\":{\"global\":\"0\",\"kind\":\"unknown_global\",\"term_index\":\"1\"}}"
        );
    }
}
