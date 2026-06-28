//! Core type inference skeleton.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, Environment, GlobalId, LevelArena, LevelId,
    LocalContext, TermArena, TermId, TermNode,
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
        TermNode::Var(index) => infer_var(context, term, index),
        TermNode::Const { global, .. } => infer_const(env, term, global),
        TermNode::Pi { ty, body } => infer_pi(levels, terms, context, env, term, ty, body),
        node => Err(unsupported_inference_error(term, &node)),
    }
}

pub fn infer_sort(levels: &mut LevelArena, terms: &mut TermArena, level: LevelId) -> TermId {
    let ty_level = levels.succ(level);
    terms.sort(ty_level)
}

fn infer_var(context: &LocalContext, term: TermId, index: u32) -> Result<TermId, CoreError> {
    context.lookup_var_type(index).ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::UnboundVariable,
            CoreLocation::root().with_field("infer"),
        )
        .with_detail("kind", "unbound_variable")
        .with_detail("index", index.to_string())
        .with_detail("term_index", term.index().to_string())
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
    let domain_level = expect_sort(terms, term, "domain", ty, domain_type)?;

    let mut body_context = context.clone();
    body_context.push_binder(ty);
    let body_type = infer(levels, terms, &body_context, env, body)?;
    let body_level = expect_sort(terms, term, "body", body, body_type)?;

    let pi_level = levels.max(domain_level, body_level);
    Ok(terms.sort(pi_level))
}

fn expect_sort(
    terms: &TermArena,
    term: TermId,
    component: &'static str,
    subject: TermId,
    inferred: TermId,
) -> Result<LevelId, CoreError> {
    match terms.node(inferred) {
        TermNode::Sort(level) => Ok(*level),
        node => Err(CoreError::new(
            CoreErrorCode::TypeMismatch,
            CoreLocation::root()
                .with_field("infer")
                .with_field("pi")
                .with_field(component),
        )
        .with_detail("kind", "pi_component_not_sort")
        .with_detail("term_index", term.index().to_string())
        .with_detail("subject_term_index", subject.index().to_string())
        .with_detail("inferred_term_index", inferred.index().to_string())
        .with_detail("expected", "sort")
        .with_detail("actual", term_kind(node))),
    }
}

fn unsupported_inference_error(term: TermId, node: &TermNode) -> CoreError {
    let mut error = CoreError::new(
        CoreErrorCode::UnsupportedFeature,
        CoreLocation::root().with_field("infer"),
    )
    .with_detail("kind", "unsupported_term_inference")
    .with_detail("term_index", term.index().to_string())
    .with_detail("term_kind", term_kind(node));

    match node {
        TermNode::Var(index) => {
            error = error.with_detail("var_index", index.to_string());
        }
        TermNode::Const { global, .. } => {
            error = error.with_detail("global", global.as_u32().to_string());
        }
        TermNode::App { arguments, .. } => {
            error = error.with_detail("argument_count", arguments.len().to_string());
        }
        TermNode::Lam { .. } | TermNode::Pi { .. } | TermNode::Let { .. } | TermNode::Sort(_) => {}
    }

    error
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
    use crate::{
        infer, infer_sort, Environment, LevelArena, LevelNode, LocalContext, TermArena, TermNode,
    };

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

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{\"field\":\"infer\"},{\"field\":\"pi\"},{\"field\":\"domain\"}],\"details\":{\"actual\":\"var\",\"expected\":\"sort\",\"inferred_term_index\":\"0\",\"kind\":\"pi_component_not_sort\",\"subject_term_index\":\"1\",\"term_index\":\"3\"}}"
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

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{\"field\":\"infer\"},{\"field\":\"pi\"},{\"field\":\"body\"}],\"details\":{\"actual\":\"var\",\"expected\":\"sort\",\"inferred_term_index\":\"0\",\"kind\":\"pi_component_not_sort\",\"subject_term_index\":\"2\",\"term_index\":\"3\"}}"
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

    #[test]
    fn unsupported_app_inference_rejects_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let function = terms.var(0);
        let argument = terms.var(1);
        let app = terms.app(function, [argument]);

        let error = infer(&mut levels, &mut terms, &context, &env, app).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_UNSUPPORTED_FEATURE\",\"location\":[{\"field\":\"infer\"}],\"details\":{\"argument_count\":\"1\",\"kind\":\"unsupported_term_inference\",\"term_index\":\"2\",\"term_kind\":\"app\"}}"
        );
    }
}
