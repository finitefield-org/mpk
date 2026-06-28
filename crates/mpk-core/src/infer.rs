//! Core type inference skeleton.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, Environment, LevelArena, LevelId, LocalContext,
    TermArena, TermId, TermNode,
};

pub fn infer(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    _context: &LocalContext,
    _env: &Environment,
    term: TermId,
) -> Result<TermId, CoreError> {
    match terms.node(term).clone() {
        TermNode::Sort(level) => Ok(infer_sort(levels, terms, level)),
        node => Err(unsupported_inference_error(term, &node)),
    }
}

pub fn infer_sort(levels: &mut LevelArena, terms: &mut TermArena, level: LevelId) -> TermId {
    let ty_level = levels.succ(level);
    terms.sort(ty_level)
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
    fn unsupported_non_sort_inference_rejects_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let context = LocalContext::new();
        let env = Environment::new();
        let var = terms.var(3);

        let error = infer(&mut levels, &mut terms, &context, &env, var).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_UNSUPPORTED_FEATURE\",\"location\":[{\"field\":\"infer\"}],\"details\":{\"kind\":\"unsupported_term_inference\",\"term_index\":\"0\",\"term_kind\":\"var\",\"var_index\":\"3\"}}"
        );
    }
}
